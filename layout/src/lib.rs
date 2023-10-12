/*!
[`Alignment`]: Alignment
[`bevy-inspector-egui`]: bevy-inspector-egui
[`Children`]: bevy::prelude::Children
[`Component`]: Component
[`Container`]: Container
[`cuicui_chirp`]: cuicui_chirp
[`cuicui_dsl`]: cuicui_dsl
[`Distribution`]: Distribution
[`dsl!`]: dsl!
[`DslBundle`]: DslBundle
[`Flow`]: Flow
[`LayoutDsl`]: LayoutDsl
[`LayoutRootCamera`]: LayoutRootCamera
[`Node`]: Node
[`ParseDsl`]: cuicui_chirp::ParseDsl
[`Root`]: Root
[`Rule`]: Rule
[`ScreenRoot`]: ScreenRoot
*/
#![doc = include_str!("../README.md")]
#![allow(clippy::match_bool, clippy::manual_range_contains)]

use bevy::app::{App, Plugin as BevyPlugin, Update};
use bevy::ecs::prelude::*;

pub use alignment::{Alignment, Distribution};
#[cfg(feature = "dsl")]
pub use cuicui_dsl::{dsl, DslBundle};
pub use direction::{Flow, Oriented, Size};
#[cfg(feature = "dsl")]
pub use dsl::LayoutDsl;
pub use error::ComputeLayoutError;
pub use labels::{ComputeLayout, ComputeLayoutSet};
pub use layout::{Container, LayoutRect, Node, Root};
pub use rule::{LeafRule, Rule};
pub use systems::{
    compute_layout, require_layout_recompute, update_leaf_nodes, LastLayoutChange,
    LayoutRootCamera, LeafNode, LeafNodeInsertWitness, ScreenRoot,
};

mod alignment;
mod direction;
mod error;
mod labels;
mod layout;
mod rule;
mod systems;

pub mod bundles;
pub mod content_sized;
#[cfg(feature = "debug")]
pub mod debug;
#[cfg(feature = "dsl")]
pub mod dsl;

/// Functions to simplify using [`dsl::LayoutDsl`].
#[cfg(feature = "dsl")]
pub mod dsl_functions {
    pub use crate::dsl::{child, pct, px};
}

/// Add layout-related sets and systems to the `Update` schedule.
///
/// This adds:
/// - [`compute_layout`] system as member of [`ComputeLayout`] and
///   [`ComputeLayoutSet`].
/// - [`ComputeLayout`]: this set only contains `compute_layout`.
/// - [`ComputeLayoutSet`]: contains `compute_layout` and
///   [content-sized](content_sized::ComputeContentSize) systems.
///
/// ## Features
///
/// When the `"reflect"` feature is enabled, also register all the layouting
/// types used by `cuicui_layout`.
pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastLayoutChange>()
            .init_resource::<LeafNodeInsertWitness>();
        let should_update = LeafNodeInsertWitness::new(true);
        app.add_systems(
            Update,
            (
                compute_layout
                    .run_if(require_layout_recompute)
                    .in_set(ComputeLayout)
                    .in_set(ComputeLayoutSet),
                (
                    update_leaf_nodes,
                    apply_deferred.run_if(resource_exists_and_equals(should_update)),
                )
                    .chain()
                    .in_set(ComputeLayoutSet)
                    .before(content_sized::ContentSizedComputeSystemSet),
            ),
        );
        #[cfg(feature = "debug")]
        app.add_plugins(debug::Plugin);

        #[cfg(feature = "reflect")]
        app.register_type::<Alignment>()
            .register_type::<Container>()
            .register_type::<Distribution>()
            .register_type::<Flow>()
            .register_type::<LeafNode>()
            .register_type::<LeafRule>()
            .register_type::<Node>()
            .register_type::<Oriented<LeafRule>>()
            .register_type::<LayoutRect>()
            .register_type::<Root>()
            .register_type::<Rule>()
            .register_type::<ScreenRoot>()
            .register_type::<Size<f32>>()
            .register_type::<Size<LeafRule>>()
            .register_type::<Size<Rule>>();
    }
}
