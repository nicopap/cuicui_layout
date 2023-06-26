use bevy::prelude::{Children, Component, Entity, Name, Query};
#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect, ReflectComponent};

use crate::{
    direction::{Oriented, Rect},
    error::{self, BadEntity, Bound, Bounds, MaybeDirectionalBound, ResultBadEntityExt},
    PosRect,
};

pub type Size = Rect<f32>;

impl Bounds {
    /// Bounds adapted to container with provided `Spec`.
    fn refine(
        &self,
        dir: Oriented,
        this: Entity,
        main: Constraint,
        cross: Constraint,
        names: &Query<&Name>,
    ) -> Result<Self, error::Why> {
        let component = |spec, dir| match spec {
            Constraint::Children(_) => Ok(Err(BadEntity(this))),
            Constraint::Parent(ratio) => Ok(Ok(self.on(dir).why(this, names)? * ratio)),
            Constraint::Fixed(fixed) => Ok(Ok(fixed)),
        };
        let main = component(main, dir)?;
        let cross = component(cross, dir.perp())?;
        Ok(Self(dir.orient(
            Rect { width: main, height: cross },
            Rect { width: cross, height: main },
        )))
    }
}
impl From<Size> for Bounds {
    fn from(value: Size) -> Self {
        Self(Rect { width: Ok(value.width), height: Ok(value.height) })
    }
}

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Container {
    pub direction: Oriented,
    pub space_use: SpaceUse,
    pub(crate) width: Constraint,
    pub(crate) height: Constraint,
}
impl Default for Container {
    fn default() -> Self {
        Container {
            direction: Oriented::Horizontal,
            space_use: SpaceUse::Stretch,
            width: Constraint::Parent(1.0),
            height: Constraint::Parent(1.0),
        }
    }
}
impl Container {
    pub fn new(direction: Oriented, space_use: SpaceUse) -> Self {
        let axis = match space_use {
            SpaceUse::Stretch => Constraint::Parent(1.0),
            SpaceUse::Compact => Constraint::Children(1.0),
        };
        let cross = Constraint::Children(1.0);
        let (width, height) = direction.absolute(axis, cross);
        Self { direction, space_use, width, height }
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
    Axis {
        main: LeafConstraint,
        cross: LeafConstraint,
    },
    /// A terminal node's constraints.
    Box {
        width: LeafConstraint,
        height: LeafConstraint,
    },
}
impl Default for Node {
    /// DO NOT USE THE DEFAULT IMPL OF `Node`, this is only to satisfy `Reflect`
    /// requirements.
    fn default() -> Self {
        Node::Box {
            width: LeafConstraint::Parent(1.0),
            height: LeafConstraint::Parent(1.0),
        }
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
        let spacer = Node::Axis {
            main: LeafConstraint::Parent(value),
            cross: LeafConstraint::Fixed(0.0),
        };
        (value <= 1.0 && value >= 0.0).then_some(spacer)
    }

    pub fn fixed(size: Size) -> Node {
        Node::Box {
            width: LeafConstraint::Fixed(size.width),
            height: LeafConstraint::Fixed(size.height),
        }
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
    fn inside_n(self, bound: MaybeDirectionalBound) -> MaybeDirectionalBound {
        bound.map(|bound| match self {
            LeafConstraint::Parent(ratio) => Ok(bound? * ratio),
            LeafConstraint::Fixed(fixed) => Ok(fixed),
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
    pub direction: Oriented,
    pub space_use: SpaceUse,
    pub bounds: Size,
}
impl Root {
    pub fn new(bounds: Size, direction: Oriented, space_use: SpaceUse) -> Self {
        Root { bounds, space_use, direction }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct AtOutput {
    main: f32,
    cross: f32,
}
impl Container {
    pub(crate) fn layout(
        &self,
        this: Entity,
        children: &Children,
        bounds: Bounds,
        to_update: &mut Query<&mut PosRect>,
        nodes: &Query<(Entity, &Node, &Children)>,
        names: &Query<&Name>,
    ) -> Result<Size, error::Why> {
        use SpaceUse::*;
        let Self { direction: dir, space_use, width, height } = *self;
        let main = dir.orient(width, height);
        let cross = dir.perp().orient(width, height);
        if children.is_empty() {
            return Ok(Size::default());
        }
        let mut child_main = 0.0;
        let mut child_cross = 0.0_f32;
        let mut node_children_count = 0;
        let bounds = bounds.refine(dir, this, main, cross, names)?;
        for child in nodes.iter_many(children) {
            let result = layout_at(child, dir, bounds, to_update, nodes, names)?;
            child_main += result.main;
            child_cross = child_cross.max(result.cross);
            node_children_count += 1;
        }
        let cross = bounds.0.on(dir.perp()).unwrap_or(child_cross);
        match space_use {
            Stretch => {
                let total_space_between = bounds.on(dir).why(this, names)? - child_main;
                if total_space_between < 0.0 {
                    return Err(error::Why::ContainerOverflow {
                        this: error::Handle::of(this, names),
                        bounds,
                        node_children_count,
                        dir_name: dir.size_name(),
                        child_size: child_main,
                    });
                }
                let space_between = total_space_between / (node_children_count - 1) as f32;
                let mut iter = to_update.iter_many_mut(children);
                let mut main_offset = 0.0;
                while let Some(mut space) = iter.fetch_next() {
                    space.pos.set_main(dir, main_offset);
                    let offset = (cross - space.size.cross(dir)) / 2.0;
                    space.pos.set_cross(dir, offset);
                    main_offset += space.size.on(dir) + space_between;
                }
                Ok(Size::with(dir, bounds.on(dir).why(this, names)?, cross))
            }
            Compact => {
                let mut main_offset = 0.0;
                let mut iter = to_update.iter_many_mut(children);
                while let Some(mut space) = iter.fetch_next() {
                    space.pos.set_main(dir, main_offset);
                    space.pos.set_cross(dir, 0.0);
                    main_offset += space.size.on(dir);
                }
                Ok(Size::with(dir, child_main, cross))
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
    (this, node, children): (Entity, &Node, &Children),
    orient: Oriented,
    bounds: Bounds,
    to_update: &mut Query<&mut PosRect>,
    nodes: &Query<(Entity, &Node, &Children)>,
    names: &Query<&Name>,
) -> Result<AtOutput, error::Why> {
    let size = match node {
        Node::Container(container) => {
            container.layout(this, children, bounds, to_update, nodes, names)?
        }
        Node::Axis { main, cross } => Size::with(
            orient,
            main.inside_n(bounds.on(orient)).why(this, names)?,
            cross.inside_n(bounds.on(orient.perp())).why(this, names)?,
        ),
        Node::Box { width, height } => Size {
            width: width.inside(bounds.0.width).why("width", this, names)?,
            height: height.inside(bounds.0.height).why("height", this, names)?,
        },
    };
    if let Ok(mut to_update) = to_update.get_mut(this) {
        to_update.size = size;
    }
    Ok(AtOutput { main: size.on(orient), cross: size.cross(orient) })
}
