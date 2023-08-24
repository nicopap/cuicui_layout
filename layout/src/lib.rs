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

use bevy::app::{App, Plugin as BevyPlugin, Update};
use bevy::ecs::prelude::*;

pub use alignment::{Alignment, Distribution};
pub use content_sized::{AppContentSizeExt, ComputeContentParam, ComputeContentSize};
pub use cuicui_dsl::{dsl, DslBundle, IntoEntityCommands};
pub use direction::{Flow, Oriented, Size};
pub use dsl::LayoutDsl;
pub use error::ComputeLayoutError;
pub use labels::{
    ComputeLayout, ComputeLayoutSet, ContentSizedComputeSystem, ContentSizedComputeSystemSet,
};
pub use layout::{Container, LayoutRect, LeafRule, Node, Root, Rule};
pub use systems::{
    compute_layout, require_layout_recompute, update_leaf_nodes, LastLayoutChange,
    LayoutRootCamera, LeafNode, LeafNodeInsertWitness, ScreenRoot,
};

mod alignment;
mod content_sized;
mod direction;
mod error;
mod labels;
mod layout;
mod systems;

pub mod bundles;
#[cfg(feature = "debug")]
pub mod debug;
pub mod dsl;

/// Functions to simplify using [`dsl::LayoutDsl`].
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
///   [content-sized](ComputeContentSize) systems.
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
                    .before(ContentSizedComputeSystemSet),
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
            .register_type::<Size<f32>>()
            .register_type::<Size<LeafRule>>()
            .register_type::<Size<Rule>>();
    }
}
