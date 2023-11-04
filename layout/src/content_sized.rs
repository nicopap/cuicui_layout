/*!
[`AnyOf`]: AnyOf
[`AppContentSizeExt::add_content_sized`]: AppContentSizeExt::add_content_sized
[`ComputeContentParam`]: ComputeContentParam
[`ComputeContentParam::Components`]: ComputeContentParam::Components
[`ComputeContentParam::condition`]: ComputeContentParam::condition
[`ComputeContentSize::compute_content`]: ComputeContentSize::compute_content
[`Entity`]: Entity
[`SystemParam`]: bevy::ecs::system::SystemParam
*/
//! Define how leaf nodes should size based on arbitrary components.
//!
#![doc = include_str!("../content_sized.md")]
use std::{any::type_name, convert::Infallible, marker::PhantomData};

use bevy::app::{App, Update};
use bevy::ecs::prelude::*;
use bevy::ecs::query::{ROQueryItem, ReadOnlyWorldQuery};
use bevy::ecs::schedule::SystemSetConfigs;
use bevy::ecs::system::{assert_is_system, StaticSystemParam, SystemParam};
use bevy::log::{debug, error, trace};
use bevy::prelude::{Name, Parent};
use bevy_mod_sysfail::{sysfail, FailureMode, LogLevel};
use thiserror::Error;

use crate::direction::Axis;
use crate::error::Handle;
use crate::{
    ComputeLayout, ComputeLayoutSet, Container, LeafNode, LeafRule, Node, Root, Rule, Size,
};

pub use crate::labels::{ContentSizedComputeSystem, ContentSizedComputeSystemSet};

type Result<T> = std::result::Result<T, BadRule>;

#[derive(Debug, Clone, Error)]
enum Why<T> {
    #[error("{}.compute_content returned a Nan when computing {1}'s {0}. Size must be a number.", type_name::<T>())]
    Nan(Axis, Handle),
    #[error("When computing content of {}: {0} depends on its parent, but it has no parents :(",  type_name::<T>())]
    Orphan(Handle),
    #[error("Not shown, crate::error::Why::CyclicRule should do this job")]
    CyclicRule,
    #[error("This error never occurs")]
    _Ignore(PhantomData<fn(T)>, Infallible),
}

impl<T> FailureMode for Why<T> {
    type ID = ();
    fn identify(&self) {}
    fn log_level(&self) -> LogLevel {
        match self {
            Why::Nan(_, _) | Why::Orphan(_) => LogLevel::Error,
            Why::CyclicRule | Why::_Ignore(..) => LogLevel::Silent,
        }
    }
    fn display(&self) -> Option<String> {
        Some(self.to_string())
    }
}

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
    ///
    /// [`SystemParam`]: bevy::ecs::system::SystemParam
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
                .in_set(ContentSizedComputeSystemSet)
                .in_set(set),
        );
        self.configure_sets(Update, S::condition(set));
        self.configure_sets(Update, ComputeLayout.after(set));
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
    fn condition(label: ContentSizedComputeSystem<Self>) -> SystemSetConfigs;
}

/// A [`SystemParam`] to compute the size of content-sized layout [`Node`]s.
///
/// In order to compute the size of content-sized nodes,
/// you should also define a [`ComputeContentParam`] and add it to the app
/// using [`AppContentSizeExt::add_content_sized`].
///
/// [`SystemParam`]: bevy::ecs::system::SystemParam
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

type BasicQuery<'w, 's, C, F> =
    Query<'w, 's, (Entity, Option<&'static Name>, Option<&'static Parent>, C), F>;

type NodeQuery<'w, 's> =
    BasicQuery<'w, 's, AnyOf<(&'static Node, &'static Root)>, Without<LeafNode>>;

#[sysfail(log(level = "error"))]
fn compute_content_size<S: ComputeContentParam>(
    compute_param: StaticSystemParam<S>,
    mut content_sized: BasicQuery<(&mut Node, S::Components), With<LeafNode>>,
    nodes: NodeQuery,
) -> std::result::Result<(), Why<S>>
where
    for<'w, 's> S::Item<'w, 's>: ComputeContentSize<Components = S::Components>,
{
    let mut errs: Option<(Why<S>, usize)> = None;
    assert_is_system(compute_content_size::<S>);
    debug!(
        "Computing content-sized nodes for {}",
        bevy::utils::get_short_name(std::any::type_name::<S>())
    );
    for (e, name, parent, (node, components)) in &mut content_sized {
        if !node.content_sized() {
            continue;
        }
        trace!("Computing size of a node with constraints: {node:?}");
        let size = match node_content_size(parent, &node, &nodes) {
            Ok(size) => size,
            Err(err) => {
                let errs = errs.get_or_insert((err.into_why(e, name), 0));
                errs.1 += 1;
                continue;
            }
        };
        let computed = compute_param.compute_content(components, size);
        let computed = Size {
            width: size.width.is_none().then_some(computed.width),
            height: size.height.is_none().then_some(computed.height),
        };
        trace!("It is: {computed:?}");
        if let Err(err) = set_node_content_size(node, computed) {
            let errs = errs.get_or_insert((err.into_why(e, name), 0));
            errs.1 += 1;
        };
    }
    if let Some((err, _)) = errs.take() {
        Err(err)
    } else {
        Ok(())
    }
}

enum BadRule {
    OrphanUnnamed,
    Orphan(Handle),
    Nan(Axis),
    Cyclic,
}
impl BadRule {
    fn into_why<T>(self, e: Entity, name: Option<&Name>) -> Why<T> {
        use Handle::{Named, Unnamed};
        let handle = || name.map_or(Unnamed(e), |n| Named(n.clone()));
        match self {
            BadRule::OrphanUnnamed => Why::Orphan(handle()),
            BadRule::Orphan(handle) => Why::Orphan(handle),
            BadRule::Nan(axis) => Why::Nan(axis, handle()),
            BadRule::Cyclic => Why::CyclicRule,
        }
    }

    fn name(self, e: Entity, name: Option<&Name>) -> Self {
        use Handle::{Named, Unnamed};
        let handle = || name.map_or(Unnamed(e), |n| Named(n.clone()));
        match self {
            BadRule::OrphanUnnamed => BadRule::Orphan(handle()),
            BadRule::Orphan(_) | BadRule::Nan(_) | BadRule::Cyclic => self,
        }
    }
}

const fn get_rules<'a>(node: (Option<&'a Node>, Option<&'a Root>)) -> Result<&'a Size<Rule>> {
    match node {
        (Some(Node::Container(Container { rules, .. })), _)
        | (None, Some(Root { node: Container { rules, .. }, .. })) => Ok(rules),
        _ => Err(BadRule::OrphanUnnamed),
    }
}

fn node_content_size(
    parent: Option<&Parent>,
    node: &Node,
    nodes: &NodeQuery,
) -> Result<Size<Option<f32>>> {
    let leaf_size = |axis, rule| match rule {
        LeafRule::Parent(ratio) => Ok(Some(parent_size(ratio, axis, parent, nodes)?)),
        LeafRule::Fixed(value) => Ok(Some(value)),
        LeafRule::Content(_) => Ok(None),
    };
    // TODO(bug)TODO(feat): Node::Axis
    if let Node::Box(size) = node {
        Ok(Size {
            width: leaf_size(Axis::Horizontal, size.width)?,
            height: leaf_size(Axis::Vertical, size.height)?,
        })
    } else {
        unreachable!(
            "node_content_size is only called on node.is_content_sized() \
            meaning this branch should never be reached"
        );
    }
}
fn parent_size(ratio: f32, axis: Axis, this: Option<&Parent>, nodes: &NodeQuery) -> Result<f32> {
    use BadRule::OrphanUnnamed as Orphan;
    let this = this.ok_or(Orphan)?.get();
    let (e, n, parent, node) = nodes.get(this).map_err(|_| Orphan)?;
    let rules = get_rules(node)?;
    match axis.relative(rules.as_ref()).main {
        Rule::Children(_) => Err(BadRule::Cyclic),
        &Rule::Fixed(value) => Ok(ratio * value),
        Rule::Parent(this_ratio) => {
            parent_size(ratio * this_ratio, axis, parent, nodes).map_err(|err| err.name(e, n))
        }
    }
}
fn set_node_content_size(mut node: Mut<Node>, new: Size<Option<f32>>) -> Result<()> {
    let Node::Box(size) = &mut *node else {
        unreachable!(
            "set_node_content_size is only called on node.is_content_sized() \
            meaning this branch should never be reached"
        );
    };
    if let (LeafRule::Content(to_update), Some(new)) = (&mut size.width, new.width) {
        if new.is_nan() {
            return Err(BadRule::Nan(Axis::Horizontal));
        }
        *to_update = new;
    }
    if let (LeafRule::Content(to_update), Some(new)) = (&mut size.height, new.height) {
        if new.is_nan() {
            return Err(BadRule::Nan(Axis::Vertical));
        }
        *to_update = new;
    }
    Ok(())
}
