#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect, ReflectComponent};
use bevy::{
    ecs::query::QueryItem,
    prelude::{Children, Component, Entity, Name, Query},
};

use crate::{
    direction::{Direction, Oriented, Size},
    error::{self, BadParent, Bound, Bounds},
    PosRect,
};

impl Bounds {
    /// Bounds adapted to container with provided `Spec`.
    fn refine(
        &self,
        dir: Direction,
        this: Entity,
        main: Constraint,
        cross: Constraint,
        names: &Query<&Name>,
    ) -> Result<Self, error::Why> {
        let component = |spec, dir| match spec {
            Constraint::Children(_) => Ok(Err(BadParent(this))),
            Constraint::Parent(ratio) => Ok(Ok(self.on(dir).why(this, names)? * ratio)),
            Constraint::Fixed(fixed) => Ok(Ok(fixed)),
        };
        let main = component(main, dir)?;
        let cross = component(cross, dir.perp())?;
        Ok(Self(Oriented::new(main, cross).on(dir)))
    }

    fn inside(self, Size { width, height }: Size<LeafConstraint>) -> Size<Bound> {
        Size {
            width: width.inside(self.0.width),
            height: height.inside(self.0.height),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Container {
    pub direction: Direction,
    pub space_use: SpaceUse,
    pub size: Size<Constraint>,
}
impl Default for Container {
    fn default() -> Self {
        Container {
            direction: Direction::Horizontal,
            space_use: SpaceUse::Stretch,
            size: Size {
                width: Constraint::Parent(1.0),
                height: Constraint::Parent(1.0),
            },
        }
    }
}
impl Container {
    pub fn new(direction: Direction, space_use: SpaceUse) -> Self {
        let main = match space_use {
            SpaceUse::Stretch => Constraint::Parent(1.0),
            SpaceUse::Compact => Constraint::Children(1.0),
        };
        let cross = Constraint::Children(1.0);
        let size = direction.absolute(Oriented::new(main, cross));
        Self { direction, space_use, size }
    }
}

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum SpaceUse {
    Stretch,
    Compact,
}
#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect), reflect(Component))]
pub enum Node {
    Container(Container),
    /// A terminal node's constraints, dependent on its container's axis.
    Axis(Oriented<LeafConstraint>),
    /// A terminal node's constraints.
    Box(Size<LeafConstraint>),
}
impl Default for Node {
    /// DO NOT USE THE DEFAULT IMPL OF `Node`, this is only to satisfy `Reflect`
    /// requirements.
    fn default() -> Self {
        Node::Box(Size::all(LeafConstraint::Parent(1.0)))
    }
}
impl Node {
    /// An invisible [`Node`] occupying `value%` of it's parent container
    /// on the main axis.
    pub fn spacer_percent(value: f32) -> Option<Self> {
        Self::spacer_ratio(value / 100.0)
    }
    /// An invisible [`Node`] occupying `value` ratio of it's parent container
    /// on the main axis.
    pub fn spacer_ratio(value: f32) -> Option<Self> {
        let spacer = Node::Axis(Oriented {
            main: LeafConstraint::Parent(value),
            cross: LeafConstraint::Fixed(0.0),
        });
        (value <= 1.0 && value >= 0.0).then_some(spacer)
    }

    pub fn fixed(size: Size<f32>) -> Node {
        Node::Box(size.map(LeafConstraint::Fixed))
    }
}

/// A constraint on an axis of a terminal `Node` (ie: doesn't have a `Children` constraint).
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum LeafConstraint {
    /// The container's size is equal to its parent's size  times `.0`.
    /// (may not be above 1)
    Parent(f32),
    /// The container's size is equal to precisely `.0` pixels.
    Fixed(f32),
}
impl LeafConstraint {
    /// Compute effective size for given [`Node`] on [`Direction`], given
    /// a potentially set parent container size.
    fn inside(self, bound: Bound) -> Bound {
        Ok(match self {
            LeafConstraint::Parent(ratio) => bound? * ratio,
            LeafConstraint::Fixed(fixed) => fixed,
        })
    }
}
impl Default for LeafConstraint {
    fn default() -> Self {
        LeafConstraint::Parent(1.0)
    }
}

/// A constraint on an axis of containers.
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum Constraint {
    /// The container's size is equal to the total size of all its children
    /// times `.0`. (may not be below 1)
    Children(f32),
    /// The container's size is equal to its parent's size  times `.0`.
    /// (may not be above 1)
    Parent(f32),
    /// The container's size is equal to precisely `.0` pixels.
    Fixed(f32),
}
impl Default for Constraint {
    fn default() -> Self {
        Constraint::Children(1.0)
    }
}

#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Root {
    pub direction: Direction,
    pub space_use: SpaceUse,
    pub bounds: Size<f32>,
}
impl Root {
    pub fn new(bounds: Size<f32>, direction: Direction, space_use: SpaceUse) -> Self {
        Root { bounds, space_use, direction }
    }
}

pub type LayoutNode = (Entity, &'static Node, &'static Children);

impl Container {
    pub(crate) fn layout(
        &self,
        this: Entity,
        children: &Children,
        bounds: Bounds,
        to_update: &mut Query<&mut PosRect>,
        nodes: &Query<LayoutNode>,
        names: &Query<&Name>,
    ) -> Result<Size<f32>, error::Why> {
        use SpaceUse::*;
        let Self { direction, space_use, size } = *self;
        let Oriented { main, cross } = direction.relative(size);

        if children.is_empty() {
            return Ok(Size::ZERO);
        }
        let mut child_orient = Oriented { main: 0.0, cross: 0.0 };
        let mut children_count = 0;
        let bounds = bounds.refine(direction, this, main, cross, names)?;
        for child_item in nodes.iter_many(children) {
            let result = layout_at(child_item, direction, bounds, to_update, nodes, names)?;
            child_orient.main += result.main;
            child_orient.cross = child_orient.cross.max(result.cross);
            children_count += 1;
        }
        let cross = bounds.0.on(direction.perp()).unwrap_or(child_orient.cross);

        match space_use {
            Stretch => {
                let total_space_between =
                    bounds.on(direction).why(this, names)? - child_orient.main;

                if total_space_between < 0.0 {
                    return Err(error::Why::ContainerOverflow {
                        this: error::Handle::of(this, names),
                        bounds,
                        node_children_count: children_count,
                        dir_name: direction.size_name(),
                        child_size: child_orient.main,
                    });
                }
                let space_between = total_space_between / (children_count - 1) as f32;

                let mut main_offset = 0.0;
                let mut iter = to_update.iter_many_mut(children);
                while let Some(mut space) = iter.fetch_next() {
                    let cross_offset = (cross - direction.perp().orient(space.size)) / 2.0;
                    space.pos.set_main(direction, main_offset);
                    space.pos.set_cross(direction, cross_offset);
                    main_offset += direction.orient(space.size) + space_between;
                }
                Ok(Oriented::new(bounds.on(direction).why(this, names)?, cross).on(direction))
            }
            Compact => {
                let mut main_offset = 0.0;
                let mut iter = to_update.iter_many_mut(children);
                while let Some(mut space) = iter.fetch_next() {
                    space.pos.set_main(direction, main_offset);
                    space.pos.set_cross(direction, 0.0);
                    main_offset += direction.orient(space.size);
                }
                Ok(Oriented::new(child_orient.main, cross).on(direction))
            }
        }
    }
}
// This functions' responsability is to compute the layout for `current` Entity,
// and all its children.
//
// Rules for this function:
//
// - Nodes will set **their own size** with the `to_update` query.
// - **the position of the children** will be set with `to_update`.
fn layout_at(
    (this, node, children): QueryItem<LayoutNode>,
    direction: Direction,
    bounds: Bounds,
    to_update: &mut Query<&mut PosRect>,
    nodes: &Query<LayoutNode>,
    names: &Query<&Name>,
) -> Result<Oriented<f32>, error::Why> {
    let size = match node {
        Node::Container(container) => {
            container.layout(this, children, bounds, to_update, nodes, names)?
        }
        Node::Axis(oriented) => bounds.inside(oriented.on(direction)).why(this, names)?,
        Node::Box(size) => bounds.inside(*size).why(this, names)?,
    };
    if let Ok(mut to_update) = to_update.get_mut(this) {
        to_update.size = size;
    }
    Ok(direction.relative(size))
}
