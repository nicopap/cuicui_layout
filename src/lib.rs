//! Layouting system for bevy cuicui.
//!
//! The layouting system is very dumb. It is based on [`Container`]s.
//! A [`Container`] layouts its content in either a column or a row.
//!     
//! The individual items are positioned according to one of two possible [`SpaceUse`].
//! Either items are compactly put one after another with [`SpaceUse::Compact`],
//! or they [`SpaceUse::Stretch`] to the parent's Vertical/Horizontal space.
//!
//! If you want some margin, use [`Node::Spacer`].
//! [`Node::Spacer::0`] is the percent of the containing container's total size.
//!
//! All things in a cuicui layout has a known fixed size. This is why
//! everything needs to live in a root countainer of a fixed size.
//!
//! ## Things you can't do
//!
//! * Several `SpaceUse::Stretch` vertical layout within a vertical layout (same for horizontal)
//!   A single `SpaceUse::Stretch` is accepted, but several do not make sense.
//! * Note that this is transitive, so a `Stretch` vertical layout within
//!   an horizontal layout within a `Stretch` vertical layout is also a no-no.
//! * `Spacer` within a `SpaceUse::Compact`.
//!
//! ## TODO:
//!
//! * Integrate Change detection
//! * Accumulate errors instead of early exit. (doubt)
//! * Root expressed as percent of UiCamera
//! * Write a tool to make and export layouts.
#![allow(clippy::manual_range_contains)]

mod direction;
mod error;
mod layout;
#[cfg(feature = "bevy_render")]
pub mod render;
pub mod typed;

use bevy::prelude::*;
use bevy_mod_sysfail::sysfail;

pub use direction::Oriented;
use error::{Bounds, Why};
pub use layout::{Constraint, Container, Node, Root, Size, SpaceUse};

#[derive(Clone, Default, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Pos {
    pub left: f32,
    pub top: f32,
}
impl Pos {
    fn set_cross(&mut self, direction: Oriented, cross: f32) {
        match direction {
            Oriented::Vertical => self.left = cross,
            Oriented::Horizontal => self.top = cross,
        }
    }
    fn set_main(&mut self, direction: Oriented, main: f32) {
        match direction {
            Oriented::Vertical => self.top = main,
            Oriented::Horizontal => self.left = main,
        }
    }
}

/// Position and size of a [`Node`] as computed by the layouting algo.
///
/// Note that `Pos` will always be relative to the top left position of the
/// containing node.
#[derive(Component, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct PosRect {
    size: Size,
    pos: Pos,
}
impl PosRect {
    pub fn pos(&self) -> Pos {
        self.pos
    }
    pub fn size(&self) -> Size {
        self.size
    }
}

// TODO:
// - minimize recomputation using `Changed`
// - better error handling (log::error!)
// - maybe parallelize
/// Run the layout algorithm on
#[sysfail(log(level = "error"))]
fn compute_layout(
    mut to_update: Query<&mut PosRect>,
    nodes: Query<(Entity, &Node, &Children)>,
    names: Query<&Name>,
    roots: Query<(Entity, &Root, &Children)>,
) -> Result<(), Why> {
    for (entity, &Root { bounds, direction, space_use }, children) in &roots {
        if let Ok(mut to_update) = to_update.get_mut(entity) {
            to_update.size = bounds;
        }
        let container = Container {
            direction,
            space_use,
            width: Constraint::Fixed(bounds.width),
            height: Constraint::Fixed(bounds.height),
        };
        let bounds = Bounds::from(bounds);
        container.layout(entity, children, bounds, &mut to_update, &nodes, &names)?;
    }
    Ok(())
}
/// Update transform of things that have a `PosRect` component.
pub fn update_transforms(mut positioned: Query<(&PosRect, &mut Transform), Changed<PosRect>>) {
    for (pos, mut transform) in &mut positioned {
        transform.translation.x = pos.pos.left;
        transform.translation.y = pos.pos.top;
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub enum Systems {
    ComputeLayout,
}

pub struct Plug;
impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_system(compute_layout.in_set(Systems::ComputeLayout));

        #[cfg(feature = "reflect")]
        app.register_type::<Container>()
            .register_type::<Oriented>()
            .register_type::<Node>()
            .register_type::<Pos>()
            .register_type::<PosRect>()
            .register_type::<Root>()
            .register_type::<Size>()
            .register_type::<SpaceUse>();
    }
}
