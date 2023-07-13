//! Layouting system for bevy cuicui.
//!
//! The layouting system is very dumb. It is based on [`Container`]s.
//! A [`Container`] layouts its content in either a column or a row.
//!     
//! The individual items are positioned according to:
//! - [`Container::align`]: The container's children [`Alignment`].
//! - [`Container::distrib`]: The container's children [`Distribution`].
//! - [`Container::flow`]: The direction in which the container's children [`Flow`].
//!
//! By default, items are aligned at the center of the container, distributed
//! on the flow direction evenly within the container.
//!
//! All things in a cuicui layout has a known size. This is why
//! everything needs to live in a root container of a fixed size.
//!
//! That's it! Now make a nice UI using bevy.
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(
    clippy::match_bool,
    clippy::manual_range_contains,
    clippy::use_self,
    clippy::redundant_pub_crate,
    clippy::module_name_repetitions
)]

mod alignment;
pub mod bundles;
mod content_sized;
mod direction;
pub mod dsl;
mod error;
mod layout;

/// Functions to simplify using [`dsl::LayoutDsl`].
pub mod dsl_functions {
    pub use crate::dsl::{child, pct, px};
}
use std::marker::PhantomData;

use bevy::ecs::component::Tick;
use bevy::ecs::prelude::*;
use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::ecs::system::SystemChangeTick;
use bevy::prelude::{App, Children, Name, Parent, Plugin as BevyPlugin, Transform, Update, Vec2};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent};
use bevy_mod_sysfail::sysfail;

pub use alignment::{Alignment, Distribution};
pub use content_sized::{AppContentSizeExt, ComputeContentParam, ComputeContentSize};
pub use cuicui_dsl::{dsl, DslBundle};
pub use direction::{Flow, Oriented, Size};
pub use dsl::LayoutDsl;
pub use error::ComputeLayoutError;
pub use layout::{Container, LeafRule, Node, NodeQuery, Root, Rule};

use crate::layout::Layout;
use error::Computed;

/// Use this camera's logical size as the root fixed-size container for
/// `cuicui_layout`.
///
/// Note that it is an error to have more than a single camera with this
/// component.
#[derive(Component, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
pub struct LayoutRootCamera;

/// Set this [`Root`] to track the [`LayoutRootCamera`]'s size.
#[derive(Component, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
pub struct ScreenRoot;

/// Position and size of a [`Node`] as computed by the layouting algo.
///
/// Note that `Pos` will always be **relative to** the top left position of the
/// containing node.
#[derive(Component, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct PosRect {
    size: Size<f32>,
    pos: Size<f32>,
}
impl PosRect {
    /// The `(top, left)` position of the [`Node`].
    #[must_use]
    pub const fn pos(&self) -> Vec2 {
        Vec2::new(self.pos.width, self.pos.height)
    }
    /// The [`Size`] of the node.
    #[must_use]
    pub const fn size(&self) -> Size<f32> {
        self.size
    }
}

/// Stores the tick of the last time [`compute_layout::<F>`] ran.
#[derive(Resource)]
pub struct LastLayoutChange<F> {
    tick: Option<Tick>,
    _f: PhantomData<fn(F)>,
}
impl<F> LastLayoutChange<F> {
    /// The last time [`compute_layout<F>`] ran.
    #[must_use]
    pub const fn tick(&self) -> Option<Tick> {
        self.tick
    }
}
impl<F> Default for LastLayoutChange<F> {
    fn default() -> Self {
        Self { tick: None, _f: PhantomData }
    }
}

type LayoutRef = (
    Option<Ref<'static, Node>>,
    Option<Ref<'static, Root>>,
    Option<Ref<'static, Children>>,
    Option<Ref<'static, Parent>>,
);

// TODO(bug): We need to .or_else this with the updates on `ComputeContentSize`
/// A run condition to tell whether it's necessary to recompute layout.
#[allow(clippy::needless_pass_by_value, clippy::must_use_candidate)]
pub fn require_layout_recompute<F: ReadOnlyWorldQuery + 'static>(
    nodes: Query<NodeQuery, F>,
    anything_changed: Query<LayoutRef, (F, Or<(With<Node>, With<Root>)>)>,
    last_layout_change: Res<LastLayoutChange<F>>,
    system_tick: SystemChangeTick,
    mut children_removed: RemovedComponents<Children>,
    mut parent_removed: RemovedComponents<Parent>,
) -> bool {
    let Some(tick) = last_layout_change.tick else {
        return true;
    };
    let this_tick = system_tick.this_run();
    let anything_changed = anything_changed.iter().any(|q| {
        matches!(q.0, Some(r) if r.last_changed().is_newer_than(tick, this_tick))
            || matches!(q.1, Some(r) if r.last_changed().is_newer_than(tick, this_tick))
            || matches!(q.2, Some(r) if r.last_changed().is_newer_than(tick, this_tick))
            || matches!(q.3, Some(r) if r.last_changed().is_newer_than(tick, this_tick))
    });
    let mut children_removed = || children_removed.iter().any(|e| nodes.contains(e));
    let mut parent_removed = || parent_removed.iter().any(|e| nodes.contains(e));

    anything_changed || children_removed() || parent_removed()
}

/// Run the layout algorithm on entities with [`Node`] and [`PosRect`] components.
///
/// You may set `F` to any query filter in order to limit the layouting to a
/// subset of layout entities.
#[sysfail(log(level = "error"))]
pub fn compute_layout<F: ReadOnlyWorldQuery + 'static>(
    mut to_update: Query<&'static mut PosRect, F>,
    nodes: Query<NodeQuery, F>,
    names: Query<&'static Name>,
    roots: Query<(Entity, &'static Root, &'static Children), F>,
    mut last_layout_change: ResMut<LastLayoutChange<F>>,
    system_tick: SystemChangeTick,
) -> Result<(), ComputeLayoutError> {
    for (entity, root, children) in &roots {
        let root_container = *root.get();
        let bounds = root.get_size(entity, &names)?;
        if let Ok(mut to_update) = to_update.get_mut(entity) {
            to_update.size = bounds;
        }
        let mut layout = Layout::new(entity, &mut to_update, &nodes, &names);
        let mut bounds: Size<Computed> = bounds.into();
        bounds.set_margin(root_container.margin, &layout)?;
        layout.container(root_container, children, bounds)?;
    }
    last_layout_change.tick = Some(system_tick.this_run());
    Ok(())
}
/// Update transform of things that have a `PosRect` component.
pub fn update_transforms(mut positioned: Query<(&PosRect, &mut Transform), Changed<PosRect>>) {
    for (pos, mut transform) in &mut positioned {
        transform.translation.x = pos.pos.width;
        transform.translation.y = pos.pos.height;
    }
}

/// Systems added by [`Plugin`].
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub enum Systems {
    /// The layouting system, [`compute_layout`].
    ComputeLayout,
    /// When [`ComputeContentSize::compute_content`]  is evaulated.
    /// [`add_content_sized`] automatically adds the relevant systems to this set.
    ///
    /// It is part of the [`Self::ComputeLayout`] set, but this happens just
    /// before computing [`compute_layout`], setting the content-sized
    /// informations.
    ///
    /// [`add_content_sized`]: AppContentSizeExt::add_content_sized
    ContentSizedCompute,
}

/// Add the [`compute_layout`] system to the bevy `Update` set.
///
/// ## Features
///
/// When the `"reflect"` feature is enabled, also register all the layouting
/// types used by `cuicui_layout`.
pub struct Plugin<F = ()>(PhantomData<fn(F)>);
impl Plugin<()> {
    /// Layout all relevant entities, without filters.
    #[must_use]
    pub const fn new() -> Self {
        Plugin(PhantomData)
    }
    /// Layout entities with the provided filters.
    #[must_use]
    pub const fn filter<F: ReadOnlyWorldQuery + 'static>() -> Plugin<F> {
        Plugin(PhantomData)
    }
}

impl<F: ReadOnlyWorldQuery + 'static> BevyPlugin for Plugin<F> {
    fn build(&self, app: &mut App) {
        app.configure_set(
            Update,
            Systems::ComputeLayout.run_if(require_layout_recompute::<F>),
        )
        .add_systems(
            Update,
            compute_layout::<F>
                .in_set(Systems::ComputeLayout)
                .after(Systems::ContentSizedCompute),
        )
        .init_resource::<LastLayoutChange<F>>();

        #[cfg(feature = "reflect")]
        app.register_type::<Alignment>()
            .register_type::<Container>()
            .register_type::<Distribution>()
            .register_type::<Flow>()
            .register_type::<Node>()
            .register_type::<Oriented<LeafRule>>()
            .register_type::<PosRect>()
            .register_type::<Root>()
            .register_type::<Rule>()
            .register_type::<Size<f32>>()
            .register_type::<Size<LeafRule>>()
            .register_type::<Size<Rule>>();
    }
}
