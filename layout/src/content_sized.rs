use bevy::{
    ecs::{
        prelude::*,
        query::{ROQueryItem, ReadOnlyWorldQuery, WorldQuery},
        schedule::SystemSetConfig,
        system::{assert_is_system, StaticSystemParam, SystemParam},
    },
    prelude::{debug, error, trace, App, Parent, Update},
};
use bevy_mod_sysfail::{sysfail, FailureMode, LogLevel};
use thiserror::Error;

use crate::{
    direction::Axis, ComputeLayout, ComputeLayoutSet, Container, ContentSizedComputeSystem,
    LeafRule, Node, Size,
};

/// Extends [`App`] to support adding [`ComputeContentSize`].
pub trait AppContentSizeExt {
    /// Add support for content-sized for given `T` [`ComputeContentSize`].
    ///
    /// To add your own content-sized nodes you need:
    /// - A type implementing [`SystemParam`], most likely using `#[derive(SystemParam)]`.
    /// - `impl ComputeContentSize for MyComputeSystemParam<'_, '_>`
    /// - `impl ComputeContentParam for MyComputeSystemParam<'static, 'static>`
    /// - `app.add_content_sized::<MyComputeSystemParam>()`
    ///
    /// The [`ComputeContentParam::Components`] and [`ComputeContentSize::Components`]
    /// types should be identical.
    fn add_content_sized<S: ComputeContentParam>(&mut self) -> &mut Self
    where
        for<'w, 's> S::Item<'w, 's>: ComputeContentSize<Components = S::Components>;
}
impl AppContentSizeExt for App {
    fn add_content_sized<S: ComputeContentParam>(&mut self) -> &mut App
    where
        for<'w, 's> S::Item<'w, 's>: ComputeContentSize<Components = S::Components>,
    {
        let set = ContentSizedComputeSystem::<S>::default();
        self.add_systems(
            Update,
            compute_content_size::<S>
                .in_set(ComputeLayoutSet)
                .in_set(set),
        );
        self.configure_set(Update, S::condition(set));
        self.configure_set(Update, ComputeLayout.after(set));
        self
    }
}

/// The static version of whatever implements [`ComputeContentSize`].
///
/// Without this, it would be impossible to access [`ComputeContentSize::Components`]
/// in the implementation.
pub trait ComputeContentParam: SystemParam + 'static
where
    for<'w, 's> Self::Item<'w, 's>: ComputeContentSize<Components = Self::Components>,
{
    /// Same as [`ComputeContentSize::Components`]. Make sure to copy the type here!
    type Components: ReadOnlyWorldQuery + 'static;

    /// Run condition for when to re-compute content-sized values.
    ///
    /// I wish you could just do `-> impl Condition` but this isn't stable in rust.
    ///
    /// Note that you should consider adding `.or_else(require_layout_recompute)`
    /// to your condition, as update to node size might influence computed-size
    /// axis size.
    fn condition(label: ContentSizedComputeSystem<Self>) -> SystemSetConfig;
}

/// A [`SystemParam`] to compute the size of content-sized layout [`Node`]s.
pub trait ComputeContentSize: SystemParam {
    /// Components of the thing which content affect the node's size.
    ///
    /// This is passed to [`Self::compute_content`] in addition to the
    /// sizes.
    type Components: ReadOnlyWorldQuery + 'static;

    /// Given provided `set` bounds (`Some` is set, `None` if content-sized),
    /// return content-sized bounds.
    ///
    /// Note that non-content-sized axis will keep the pre-set size, even
    /// if a different value is returned for that axis.
    fn compute_content(
        &self,
        components: ROQueryItem<Self::Components>,
        set_size: Size<Option<f32>>,
    ) -> Size<f32>;
}

type ParentQuery<'w, 's, Wq> =
    Query<'w, 's, (Entity, Option<&'static Parent>, &'static mut Node, Wq)>;

// TODO(perf): instead of storing in `to_update` and inserting everything
// afterward, we should Split `to_set` in two. This would also fix the `Root`
// problem.
#[sysfail(log(level = "error"))]
fn compute_content_size<S: ComputeContentParam>(
    compute_param: StaticSystemParam<S>,
    mut to_set: ParentQuery<S::Components>,
    mut to_update: Local<Vec<(Entity, Size<Option<f32>>)>>,
) -> Result<(), BadRule>
where
    for<'w, 's> S::Item<'w, 's>: ComputeContentSize<Components = S::Components>,
{
    assert_is_system(compute_content_size::<S>);
    debug!(
        "Computing content-sized nodes for {}",
        bevy::utils::get_short_name(std::any::type_name::<S>())
    );

    for (entity, parent, node, components) in &to_set {
        if !node.content_sized() {
            continue;
        }
        trace!("Computing size of a node with constraints: {node:?}");
        let size = node_content_size(parent, node, &to_set)?;
        let computed = compute_param.compute_content(components, size);
        let computed = Size {
            width: size.width.is_none().then_some(computed.width),
            height: size.height.is_none().then_some(computed.height),
        };
        trace!("It is: {computed:?}");
        to_update.push((entity, computed));
    }
    for (node, computed) in to_update.drain(..) {
        // SAFETY: due to the above, this can only be valid
        let node = unsafe { to_set.get_component_mut::<Node>(node).unwrap_unchecked() };
        set_node_content_size(node, computed);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Error)]
#[error("Bad rule, couldn't compute content sizes")]
struct BadRule;

impl FailureMode for BadRule {
    type ID = ();

    fn log_level(&self) -> LogLevel {
        LogLevel::Warn
    }

    fn identify(&self) {}

    fn display(&self) -> Option<String> {
        Some(self.to_string())
    }
}

// TODO(bug): This breaks when the source of size is `Root`.
// (unrelated) Note: This is tail-recursive, but I'm not sure how much that maters.
fn parent_size<Wq: WorldQuery>(
    ratio: f32,
    axis: Axis,
    node: Option<&Parent>,
    parents: &ParentQuery<Wq>,
) -> Result<f32, BadRule> {
    let node = node.ok_or(BadRule)?.get();
    let (_, parent, node, _) = parents.get(node).map_err(|_| BadRule)?;
    let Node::Container(Container { rules, ..}) = node else {
        return Err(BadRule);
    };
    match axis.relative(rules.as_ref()).main {
        crate::Rule::Children(_) => Err(BadRule),
        crate::Rule::Fixed(value) => Ok(ratio * *value),
        crate::Rule::Parent(this_ratio) => parent_size(this_ratio * ratio, axis, parent, parents),
    }
}
fn leaf_size<Wq: WorldQuery>(
    axis: Axis,
    rule: LeafRule,
    parent: Option<&Parent>,
    parents: &ParentQuery<Wq>,
) -> Result<Option<f32>, BadRule> {
    match rule {
        LeafRule::Parent(ratio) => Ok(Some(parent_size(ratio, axis, parent, parents)?)),
        LeafRule::Fixed(value, false) => Ok(Some(value)),
        LeafRule::Fixed(_, true) => Ok(None),
    }
}
fn node_content_size<Wq: WorldQuery>(
    parent: Option<&Parent>,
    node: &Node,
    parents: &ParentQuery<Wq>,
) -> Result<Size<Option<f32>>, BadRule> {
    // TODO(bug)TODO(feat): Node::Axis
    if let Node::Box(size) = node {
        Ok(Size {
            width: leaf_size(Axis::Horizontal, size.width, parent, parents)?,
            height: leaf_size(Axis::Vertical, size.height, parent, parents)?,
        })
    } else {
        Err(BadRule)
    }
}
fn set_node_content_size(mut node: Mut<Node>, new: Size<Option<f32>>) {
    let Node::Box(size) = &mut *node else {
        unreachable!("bad");
    };
    if let (LeafRule::Fixed(to_update, true), Some(new)) = (&mut size.width, new.width) {
        if new.is_nan() {
            error!("Some computed width was NAN, this will break the layouting algo");
        }
        *to_update = new;
    }
    if let (LeafRule::Fixed(to_update, true), Some(new)) = (&mut size.height, new.height) {
        if new.is_nan() {
            error!("Some computed height was NAN, this will break the layouting algo");
        }
        *to_update = new;
    }
}
