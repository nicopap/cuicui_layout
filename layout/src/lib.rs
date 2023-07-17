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

use bevy::ecs::{component::Tick, prelude::*, system::SystemChangeTick};
use bevy::prelude::{debug, App, Children, Name, Parent, Plugin as BevyPlugin, Update, Vec2};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent};
use bevy_mod_sysfail::sysfail;

use crate::layout::Layout;
use error::Computed;

mod alignment;
pub mod bundles;
mod content_sized;
#[cfg(feature = "debug")]
pub mod debug;
mod direction;
pub mod dsl;
mod error;
mod labels;
mod layout;

/// Functions to simplify using [`dsl::LayoutDsl`].
pub mod dsl_functions {
    pub use crate::dsl::{child, pct, px};
}

pub use alignment::{Alignment, Distribution};
pub use content_sized::{AppContentSizeExt, ComputeContentParam, ComputeContentSize};
pub use cuicui_dsl::{dsl, DslBundle, IntoEntityCommands};
pub use direction::{Flow, Oriented, Size};
pub use dsl::LayoutDsl;
pub use error::ComputeLayoutError;
pub use labels::{ComputeLayout, ComputeLayoutSet, ContentSizedComputeSystem};
pub use layout::{Container, LeafRule, Node, NodeQuery, Root, Rule};

/// Use this camera's logical size as the root container size.
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
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
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

/// Stores the tick of the last time [`compute_layout`] ran.
#[derive(Resource, Default)]
pub struct LastLayoutChange {
    tick: Option<Tick>,
}
impl LastLayoutChange {
    /// The last time [`compute_layout`] ran.
    #[must_use]
    pub const fn tick(&self) -> Option<Tick> {
        self.tick
    }
}

type LayoutRef = (
    Option<Ref<'static, Node>>,
    Option<Ref<'static, Root>>,
    Option<Ref<'static, Children>>,
    Option<Ref<'static, Parent>>,
);

/// A run condition to tell whether it's necessary to recompute layout.
#[allow(clippy::needless_pass_by_value, clippy::must_use_candidate)]
pub fn require_layout_recompute(
    nodes: Query<NodeQuery>,
    anything_changed: Query<LayoutRef, Or<(With<Node>, With<Root>)>>,
    last_layout_change: Res<LastLayoutChange>,
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

/// Run the layout algorithm.
#[sysfail(log(level = "error"))]
pub fn compute_layout(
    mut to_update: Query<&'static mut PosRect>,
    nodes: Query<NodeQuery>,
    names: Query<&'static Name>,
    roots: Query<(Entity, &'static Root, &'static Children)>,
    mut last_layout_change: ResMut<LastLayoutChange>,
    system_tick: SystemChangeTick,
) -> Result<(), ComputeLayoutError> {
    debug!("Computing layout");
    last_layout_change.tick = Some(system_tick.this_run());
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
    Ok(())
}

/// Add layout-related sets and systems to the `Update` schedule.
///
/// This adds:
/// - [`compute_layout`] system as member of [`ComputeLayout`] and
///   [`ComputeLayoutSet`].
/// - [`ComputeLayout`]: this set only contains `compute_layout`.
/// - [`ComputeLayoutSet`]: contains `compute_layout` and
///   [content-sized](ComputeContentSize) systems.
///
/// ## Features
///
/// When the `"reflect"` feature is enabled, also register all the layouting
/// types used by `cuicui_layout`.
pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastLayoutChange>().add_systems(
            Update,
            compute_layout
                .run_if(require_layout_recompute)
                .in_set(ComputeLayout)
                .in_set(ComputeLayoutSet),
        );
        #[cfg(feature = "debug")]
        app.add_plugins(debug::Plugin);

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
