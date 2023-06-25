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
//! * `ChildDefined(how_much_larger_than_child)`
//! * Integrate Change detection
//! * Accumulate errors instead of early exit. (doubt)
//! * Root expressed as percent of UiCamera
//! * Write a tool to make and export layouts.
#![allow(clippy::manual_range_contains)]

use std::fmt;

use bevy::prelude::*;
use bevy_mod_sysfail::sysfail;

use self::error::{parent_is_stretch, Why};

mod error;
#[cfg(feature = "bevy_render")]
pub mod render;
pub mod typed;

#[derive(Clone, Default, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Pos {
    pub left: f32,
    pub top: f32,
}
impl Pos {
    fn set_cross(&mut self, direction: Direction, cross: f32) {
        match direction {
            Direction::Vertical => self.left = cross,
            Direction::Horizontal => self.top = cross,
        }
    }
    fn set_main(&mut self, direction: Direction, main: f32) {
        match direction {
            Direction::Vertical => self.top = main,
            Direction::Horizontal => self.left = main,
        }
    }
}

// TODO(clean): replace `Size` and `Bounds` by `AxisRect<f32>` and `AxisRect<Bound>`
#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Size {
    pub width: f32,
    pub height: f32,
}
impl Size {
    fn with(direction: Direction, main: f32, cross: f32) -> Self {
        match direction {
            Direction::Vertical => Self { height: main, width: cross },
            Direction::Horizontal => Self { height: cross, width: main },
        }
    }
    fn on(&self, direction: Direction) -> f32 {
        direction.of(self.width, self.height)
    }
    fn cross(&self, direction: Direction) -> f32 {
        self.on(direction.perp())
    }
}
impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}×{}", self.width, self.height)
    }
}
type Bound = Result<f32, Entity>;
#[derive(Clone, Debug, Copy, PartialEq)]
struct Bounds {
    width: Bound,
    height: Bound,
}
impl Bounds {
    fn on(&self, dir: Direction, this: Entity, names: &Query<&Name>) -> Result<f32, Why> {
        let component = dir.of(self.width, self.height);
        let name = dir.of("width", "height");
        or_why(component, name, this, names)
    }
    /// Bounds adapted to container with provided `Spec`.
    fn refine(
        &self,
        dir: Direction,
        this: Entity,
        main: Constraint,
        cross: Constraint,
        names: &Query<&Name>,
    ) -> Result<Self, Why> {
        let component = |spec, dir| match spec {
            Constraint::Children(_) => Ok(Err(this)),
            Constraint::Parent(ratio) => Ok(Ok(self.on(dir, this, names)? * ratio)),
            Constraint::Fixed(fixed) => Ok(Ok(fixed)),
        };
        let main = component(main, dir)?;
        let cross = component(cross, dir.perp())?;
        Ok(dir.of(
            Self { width: main, height: cross },
            Self { width: cross, height: main },
        ))
    }
}
impl fmt::Display for Bounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.width {
            Ok(v) => write!(f, "{v}×")?,
            Err(_) => write!(f, "?×")?,
        }
        match self.height {
            Ok(v) => write!(f, "{v}"),
            Err(_) => write!(f, "?"),
        }
    }
}
impl From<Size> for Bounds {
    fn from(value: Size) -> Self {
        Self { width: Ok(value.width), height: Ok(value.height) }
    }
}
fn or_why(
    bound: Bound,
    name: &'static str,
    this: Entity,
    names: &Query<&Name>,
) -> Result<f32, Why> {
    bound.map_err(|e| parent_is_stretch(name, this, e, names))
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

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Container {
    pub direction: Direction,
    pub space_use: SpaceUse,
    width: Constraint,
    height: Constraint,
}
impl Default for Container {
    fn default() -> Self {
        Container {
            direction: Direction::Horizontal,
            space_use: SpaceUse::Stretch,
            width: Constraint::Parent(1.0),
            height: Constraint::Parent(1.0),
        }
    }
}
impl Container {
    pub fn new(direction: Direction, space_use: SpaceUse) -> Self {
        let axis = match space_use {
            SpaceUse::Stretch => Constraint::Parent(1.0),
            SpaceUse::Compact => Constraint::Children(1.0),
        };
        let cross = Constraint::Children(1.0);
        let (width, height) = direction.real_of(axis, cross);
        Self { direction, space_use, width, height }
    }
}

/// The layout direction of a [`Container`].
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum Direction {
    /// Children are arranged on the vertical axis.
    Vertical,
    /// Children are arranged on the horizontal axis.
    Horizontal,
}
impl Direction {
    const fn of<T: Copy>(self, width: T, height: T) -> T {
        match self {
            Direction::Vertical => height,
            Direction::Horizontal => width,
        }
    }
    /// Returns (width, height) according to axis and cross of this direction.
    const fn real_of<T>(self, main: T, cross: T) -> (T, T) {
        match self {
            Direction::Vertical => (cross, main),
            Direction::Horizontal => (main, cross),
        }
    }
    /// Returns (main, cross) according to this direction.
    const fn size_of(self, bounds: Bounds) -> (Bound, Bound) {
        self.real_of(bounds.width, bounds.height)
    }
    const fn perp(self) -> Self {
        use self::Direction::*;
        self.of(Vertical, Horizontal)
    }
    const fn size_name(&self) -> &'static str {
        self.of("width", "height")
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
    pub fn inside(self, bound: Bound) -> Bound {
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
    pub bounds: Size,
}
impl Root {
    pub fn new(bounds: Size, direction: Direction, space_use: SpaceUse) -> Self {
        Root { bounds, space_use, direction }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct AtOutput {
    main: f32,
    cross: f32,
}
impl Container {
    fn layout(
        &self,
        this: Entity,
        children: &Children,
        bounds: Bounds,
        to_update: &mut Query<&mut PosRect>,
        nodes: &Query<(Entity, &Node, &Children)>,
        names: &Query<&Name>,
    ) -> Result<Size, Why> {
        use SpaceUse::*;
        let Self { direction: dir, space_use, width, height } = *self;
        let main = dir.of(width, height);
        let cross = dir.perp().of(width, height);
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
        let cross = dir.of(bounds.height, bounds.width).unwrap_or(child_cross);
        match space_use {
            Stretch => {
                let total_space_between = bounds.on(dir, this, names)? - child_main;
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
                Ok(Size::with(dir, bounds.on(dir, this, names)?, cross))
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
    parent_dir: Direction,
    bounds: Bounds,
    to_update: &mut Query<&mut PosRect>,
    nodes: &Query<(Entity, &Node, &Children)>,
    names: &Query<&Name>,
) -> Result<AtOutput, Why> {
    let or_why = |bound, name| or_why(bound, name, this, names);
    let size = match node {
        Node::Container(container) => {
            container.layout(this, children, bounds, to_update, nodes, names)?
        }
        Node::Axis { main, cross } => {
            let (main_b, cross_b) = parent_dir.size_of(bounds);
            Size::with(
                parent_dir,
                or_why(main.inside(main_b), parent_dir.size_name())?,
                or_why(cross.inside(cross_b), parent_dir.size_name())?,
            )
        }
        Node::Box { width, height } => Size {
            width: or_why(width.inside(bounds.width), "width")?,
            height: or_why(height.inside(bounds.height), "height")?,
        },
    };
    if let Ok(mut to_update) = to_update.get_mut(this) {
        to_update.size = size;
    }
    Ok(AtOutput {
        main: size.on(parent_dir),
        cross: size.cross(parent_dir),
    })
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
            .register_type::<Direction>()
            .register_type::<Node>()
            .register_type::<Pos>()
            .register_type::<PosRect>()
            .register_type::<Root>()
            .register_type::<Size>()
            .register_type::<SpaceUse>();
    }
}
