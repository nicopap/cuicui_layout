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
#![allow(clippy::manual_range_contains)]

mod alignment;
// mod builder;
mod direction;
mod error;
mod layout;
pub mod typed;

use std::marker::PhantomData;

use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*};
use bevy_mod_sysfail::sysfail;

pub use alignment::{Alignment, Distribution};
pub use direction::{Flow, Oriented, Size};
use error::Bounds;
pub use error::ComputeLayoutError;
pub use layout::{Container, LayoutNode, LeafRule, Node, Root, Rule};

/// Position and size of a [`Node`] as computed by the layouting algo.
///
/// Note that `Pos` will always be relative to the top left position of the
/// containing node.
#[derive(Component, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct PosRect {
    size: Size<f32>,
    pos: Size<f32>,
}
impl PosRect {
    pub fn pos(&self) -> Vec2 {
        Vec2::new(self.pos.width, self.pos.height)
    }
    pub fn size(&self) -> Size<f32> {
        self.size
    }
}

#[derive(Bundle)]
pub struct LayoutBundle {
    pub node: Node,
    pub pos: PosRect,
}
impl LayoutBundle {
    pub fn new(node: Node) -> Self {
        LayoutBundle { node, pos: PosRect::default() }
    }
}
// TODO:
// - minimize recomputation using `Changed`
// - better error handling (log::error!)
// - maybe parallelize
/// Run the layout algorithm on
#[sysfail(log(level = "error"))]
pub fn compute_layout<F: ReadOnlyWorldQuery>(
    mut to_update: Query<&mut PosRect, F>,
    nodes: Query<LayoutNode, F>,
    names: Query<&Name>,
    roots: Query<(Entity, &Root, &Children), F>,
) -> Result<(), ComputeLayoutError> {
    for (entity, &Root { bounds, flow, align, distrib }, children) in &roots {
        if let Ok(mut to_update) = to_update.get_mut(entity) {
            to_update.size = bounds;
        }
        let container = Container {
            flow,
            align,
            distrib,
            size: bounds.map(Rule::Fixed),
        };
        let bounds = Bounds::from(bounds);
        container.layout::<F>(entity, children, bounds, &mut to_update, &nodes, &names)?;
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub enum Systems {
    ComputeLayout,
}

pub struct Plug<F = ()>(PhantomData<fn(F)>);
impl Plug<()> {
    pub const fn new() -> Self {
        Plug(PhantomData)
    }
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
