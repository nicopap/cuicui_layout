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
//!
//! ## TODO:
//!
//! * Integrate Change detection
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(
    clippy::manual_range_contains,
    clippy::use_self,
    clippy::redundant_pub_crate
)]

mod alignment;
pub mod bundles;
mod direction;
pub mod dsl;
mod error;
mod layout;
mod macros;

use std::marker::PhantomData;

use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::prelude::*;
use bevy_mod_sysfail::sysfail;

pub use alignment::{Alignment, Distribution};
pub use direction::{Flow, Oriented, Size};
pub use error::ComputeLayoutError;
use error::Computed;
pub use layout::{Container, LeafRule, Node, NodeQuery, Root, Rule};

use crate::layout::Layout;

/// Use this camera's logical size as the root fixed-size container for
/// `cuicui_layout`.
///
/// Note that it is an error to have more than a single camera with this
/// component.
#[derive(Component, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect), reflect(Component))]
pub struct LayoutRootCamera;

/// Set this [`Root`] to track the [`LayoutRootCamera`]'s size.
#[derive(Component, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect), reflect(Component))]
pub struct ScreenRoot;

/// Position and size of a [`Node`] as computed by the layouting algo.
///
/// Note that `Pos` will always be **relative to** the top left position of the
/// containing node.
#[derive(Component, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
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

// TODO:
// - minimize recomputation using `Changed`
// - better error handling (log::error!)
// - maybe parallelize
/// Run the layout algorithm on entities with [`Node`] and [`PosRect`] components.
///
/// You may set `F` to any query filter in order to limit the layouting to a
/// subset of layout entities.
#[sysfail(log(level = "error"))]
pub fn compute_layout<F: ReadOnlyWorldQuery>(
    mut to_update: Query<&'static mut PosRect, F>,
    nodes: Query<NodeQuery, F>,
    names: Query<&'static Name>,
    roots: Query<(Entity, &'static Root, &'static Children), F>,
) -> Result<(), ComputeLayoutError> {
    for (entity, root, children) in &roots {
        let bounds = root.size();
        if let Ok(mut to_update) = to_update.get_mut(entity) {
            to_update.size = bounds;
        }
        let root = *root.get();
        let mut layout = Layout::new(entity, &mut to_update, &nodes, &names);
        let mut bounds: Size<Computed> = bounds.into();
        bounds.set_margin(root.margin, &layout)?;
        layout.container(root, children, bounds)?;
    }
    Ok(())
}
/// Update transform of things that have a `PosRect` component.
pub fn update_transforms(mut positioned: Query<(&PosRect, &mut Transform), Changed<PosRect>>) {
    for (pos, mut transform) in &mut positioned {
        transform.translation.x = pos.pos.width;
        transform.translation.y = pos.pos.height;
    }
}

/// Systems added by [`Plug`].
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub enum Systems {
    /// The layouting system, [`compute_layout`].
    ComputeLayout,
}

/// Add the [`compute_layout`] system to the bevy `Update` set.
///
/// ## Features
///
/// When the `"reflect"` feature is enabled, also register all the layouting
/// types used by `cuicui_layout`.
pub struct Plug<F = ()>(PhantomData<fn(F)>);
impl Plug<()> {
    /// Layout all relevant entities, without filters.
    #[must_use]
    pub const fn new() -> Self {
        Plug(PhantomData)
    }
    /// Layout entities with the provided filters.
    #[must_use]
    pub const fn filter<F: ReadOnlyWorldQuery + 'static>() -> Plug<F> {
        Plug(PhantomData)
    }
}

impl<F: ReadOnlyWorldQuery + 'static> Plugin for Plug<F> {
    fn build(&self, app: &mut App) {
        app.add_system(compute_layout::<F>.in_set(Systems::ComputeLayout));

        #[cfg(feature = "reflect")]
        app.register_type::<Alignment>()
            .register_type::<Rule>()
            .register_type::<Container>()
            .register_type::<Flow>()
            .register_type::<Distribution>()
            .register_type::<Node>()
            .register_type::<Oriented<LeafRule>>()
            .register_type::<PosRect>()
            .register_type::<Root>()
            .register_type::<Size<Rule>>()
            .register_type::<Size<f32>>()
            .register_type::<Size<LeafRule>>();
    }
}