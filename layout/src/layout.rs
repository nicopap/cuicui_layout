//! The `cuicui_layout` algorithm.

#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect, ReflectComponent};
use bevy::{
    ecs::query::{QueryItem, ReadOnlyWorldQuery},
    prelude::{Children, Component, Entity, Name, Query},
};

use crate::{
    alignment::{Align, Alignment, Distribution},
    direction::{Flow, Oriented, Size},
    error::{self, BadParent, Bound, Bounds, Handle},
    PosRect,
};

impl Bounds {
    /// Bounds adapted to container with provided `Rule`.
    fn refine(
        &self,
        flow: Flow,
        this: Entity,
        Oriented { main, cross }: Oriented<Rule>,
        names: &Query<&Name>,
    ) -> Result<Self, error::Why> {
        let component = |rule, dir| match rule {
            Rule::Children(_) => Ok(Err(BadParent(this))),
            Rule::Parent(ratio) => Ok(Ok(self.on(dir).why(this, names)? * ratio)),
            Rule::Fixed(fixed) => Ok(Ok(fixed)),
        };
        let main = component(main, flow)?;
        let cross = component(cross, flow.perp())?;
        Ok(Self(flow.absolute(Oriented::new(main, cross))))
    }

    fn inside(self, Size { width, height }: Size<LeafRule>) -> Size<Bound> {
        Size {
            width: width.inside(self.0.width),
            height: height.inside(self.0.height),
        }
    }
}

/// Parameters of a container, ie: a node that contains other nodes.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
// TODO(clean): Split out `size` so that I can re-use it in `Root`
// TODO(feat): Add margin to this.
pub struct Container {
    /// The axis on which the nodes in this containers are arranged.
    pub flow: Flow,

    /// Where on the cross axis are nodes aligned.
    ///
    /// When [`Flow::Vertical`], `align` decides whether children nodes are:
    /// - [`Alignment::Start`]: aligned to the left of this container.
    /// - [`Alignment::Center`]: centered in the middle of the main axis of this container.
    /// - [`Alignment::End`]: aligned to the right of this container.
    ///
    /// For [`Flow::Horizontal`], instead of left/right, it would be top/bottom.
    pub align: Alignment,

    /// How to distribute the children of this container.
    ///
    /// When [`Flow::Horizontal`], `distrib` tells whether children nodes are:
    /// - [`Distribution::Start`]: all clumped to the left.
    /// - [`Distribution::FillMain`] distributed evenly on the horizontal axis to
    ///   fill this container.
    /// - [`Distribution::End`]: all clumped to the right.
    ///
    /// For [`Flow::Vertical`], instead of left/right, it would be top/bottom.
    ///
    /// > **Warning**: [`Distribution`] other than [`Distribution::Start`] requires
    /// > this container to have their `size` not depend on children size on the main
    /// > axis!
    /// >
    /// > When [`Flow::Horizontal`] and [`Distribution::FillMain`], `size.width`
    /// > cannot be [`Rule::Children`]!
    pub distrib: Distribution,

    /// How to evaluate the size of this container.
    ///
    /// See [`Rule`] for details.
    pub size: Size<Rule>,
}
impl Default for Container {
    fn default() -> Self {
        Container {
            flow: Flow::Horizontal,
            align: Alignment::Center,
            distrib: Distribution::FillMain,
            size: Size {
                width: Rule::Parent(1.0),
                height: Rule::Parent(1.0),
            },
        }
    }
}
impl Container {
    /// Create a new [`Container`] with given parameters, fitting the size of
    /// its children on the `flow` perpendicular axis.
    ///
    /// The `main` axis is [`Rule::Children(1.0)`] when `distrib` is [`Distribution::Start`],
    /// [`Rule::Parent(1.0)`] otherwise.
    #[must_use]
    pub const fn new(flow: Flow, align: Alignment, distrib: Distribution) -> Self {
        let main = match distrib {
            Distribution::FillMain | Distribution::End => Rule::Parent(1.0),
            Distribution::Start => Rule::Children(1.0),
        };
        let size = flow.absolute(Oriented::new(main, Rule::Children(1.0)));
        Self { flow, align, distrib, size }
    }
    /// Create a [`Container`] where children are center-aligned and
    /// fill this container on the `flow` main axis.
    #[must_use]
    pub const fn stretch(flow: Flow) -> Self {
        Self::new(flow, Alignment::Center, Distribution::FillMain)
    }
    /// Create a [`Container`] where children are compactly bunched at the
    /// start of the main and cross axis.
    #[must_use]
    pub const fn compact(flow: Flow) -> Self {
        Self::new(flow, Alignment::Start, Distribution::Start)
    }
}

/// A root [`Container`]. This acts as a [`Container`], but layouting "starts" from it.
///
/// This is a marker [`Component`] used in [`crate::compute_layout`] system.
///
/// Unlike a [`Container`], a `Root` always has a fixed `size`, (`bounds`).
#[derive(Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect), reflect(Component))]
pub struct Root {
    /// The fixed size of this root container.
    pub bounds: Size<f32>,
    /// See [`Container::flow`].
    pub flow: Flow,
    /// See [`Container::align`].
    pub align: Alignment,
    /// See [`Container::distrib`].
    pub distrib: Distribution,
}
impl Root {
    /// Create a new [`Root`] with given parameters.
    #[must_use]
    pub const fn new(
        bounds: Size<f32>,
        flow: Flow,
        align: Alignment,
        distrib: Distribution,
    ) -> Self {
        Root { bounds, flow, align, distrib }
    }
    /// Create a [`Root`] container where children are center-aligned and
    /// fill this container on the `flow` main axis.
    #[must_use]
    pub const fn stretch(bounds: Size<f32>, flow: Flow) -> Self {
        Root::new(bounds, flow, Alignment::Center, Distribution::FillMain)
    }
    /// Create a [`Root`] container where children are compactly bunched at the
    /// start of the main and cross axis.
    #[must_use]
    pub const fn compact(bounds: Size<f32>, flow: Flow) -> Self {
        Root::new(bounds, flow, Alignment::Start, Distribution::Start)
    }
}

/// A [`Component`] integrating the attached [`Entity`] into the `cuicui_layout`
/// layouting algorithm.
#[derive(Component, Clone, Copy, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect), reflect(Component))]
pub enum Node {
    /// This container holds other `Node`s, it is an error for a `Container`
    /// to not have children.
    Container(Container),

    /// A terminal node's constraints, dependent on its container's axis.
    Axis(Oriented<LeafRule>),

    /// A terminal node's constraints.
    Box(Size<LeafRule>),
}
impl Default for Node {
    /// DO NOT USE THE DEFAULT IMPL OF `Node`, this is only to satisfy `Reflect`
    /// requirements.
    fn default() -> Self {
        Node::Box(Size::all(LeafRule::Parent(1.0)))
    }
}
impl Node {
    /// A [`Node`] occupying `value%` of it's parent container on the main axis.
    ///
    /// Returns `None` if `value` is not between 0 and 100.
    #[must_use]
    pub fn spacer_percent(value: f32) -> Option<Self> {
        Self::spacer_ratio(value / 100.0)
    }
    /// A [`Node`] occupying `value` ratio of it's parent container on the main axis.
    ///
    /// Returns `None` if `ratio` is not between 0 and 1.
    #[must_use]
    pub fn spacer_ratio(value: f32) -> Option<Self> {
        let spacer = Node::Axis(Oriented {
            main: LeafRule::Parent(value),
            cross: LeafRule::Fixed(0.0),
        });
        (value <= 1.0 && value >= 0.0).then_some(spacer)
    }
    /// A fixed size terminal [`Node`], without children.
    #[must_use]
    pub fn fixed(size: Size<f32>) -> Self {
        Node::Box(size.map(LeafRule::Fixed))
    }
}

/// A constraint on an axis of a terminal `Node` (ie: doesn't have a `Children` constraint).
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum LeafRule {
    /// The container's size is equal to its parent's size  times `f32`.
    /// (may not be above 1)
    Parent(f32),

    /// The container's size is equal to precisely `f32` pixels.
    Fixed(f32),
}
impl LeafRule {
    /// Compute effective size for given [`Node`] on [`Flow`], given
    /// a potentially set parent container size.
    fn inside(self, bound: Bound) -> Bound {
        Ok(match self {
            LeafRule::Parent(ratio) => bound? * ratio,
            LeafRule::Fixed(fixed) => fixed,
        })
    }
}
impl Default for LeafRule {
    fn default() -> Self {
        LeafRule::Parent(1.0)
    }
}

/// A constraint on an axis of containers.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum Rule {
    /// The container's size is equal to the total size of all its children
    /// times `f32`. (may not be below 1).
    ///
    /// The computed size of children depends on the container's main axis.
    /// For a [`Flow::Horizontal`] container:
    /// - The **horizontal** size of children is the sum of the width of every children.
    /// - The **vertical** size of children is that of the tallest child.
    ///
    /// The reverse is true for [`Flow::Vertical`].
    ///
    /// When [`Rule::Children`] is used on a container's size, none of its children
    /// may depend on its parent size. It would lead to a circular dependency.
    Children(f32),

    /// The container's size is equal to its parent's size  times `f32`.
    /// (may not be above 1)
    Parent(f32),

    /// The container's size is equal to precisely `f32` pixels.
    Fixed(f32),
}
impl Default for Rule {
    fn default() -> Self {
        Rule::Children(1.0)
    }
}

/// [`WorldQuery`] item used by the layout function.
///
/// [`WorldQuery`]: bevy::ecs::query::WorldQuery
pub type NodeQuery = (Entity, &'static Node, Option<&'static Children>);

// TODO(bug): There should be an error when overflow on cross size.
#[allow(clippy::cast_precision_loss)] // count as f32
pub(crate) fn layout<F: ReadOnlyWorldQuery>(
    Container { flow, distrib, align, size }: Container,
    this: Entity,
    children_entities: &Children,
    bounds: Bounds,
    to_update: &mut Query<&mut PosRect, F>,
    nodes: &Query<NodeQuery, F>,
    names: &Query<&Name>,
) -> Result<Size<f32>, error::Why> {
    if children_entities.is_empty() {
        return Ok(Size::all(0.0));
    }
    let mut child_size = Oriented { main: 0.0, cross: 0.0 };
    let mut children_count = 0;
    let bounds = bounds.refine(flow, this, flow.relative(size), names)?;
    for child_item in nodes.iter_many(children_entities) {
        let result = layout_at(child_item, flow, bounds, to_update, nodes, names)?;
        child_size.main += result.main;
        child_size.cross = child_size.cross.max(result.cross);
        children_count += 1;
    }
    let cross_size = bounds.0.on(flow.perp()).unwrap_or(child_size.cross);
    let cross_align = Align::new(cross_size, align);

    let (main_size, mut main_offset, space_between) = match distrib {
        Distribution::FillMain => {
            // TODO(bug)TODO(err): error message here is bogus "this needs to know size of this"
            // instead, this error should be created when computing child layout.
            let main_size = bounds.on(flow).why(this, names)?;
            let total_space_between = main_size - child_size.main;
            if total_space_between < 0.0 {
                return Err(error::Why::ContainerOverflow {
                    this: error::Handle::of(this, names),
                    bounds,
                    node_children_count: children_count,
                    dir_name: flow.size_name(),
                    child_size: child_size.main,
                });
            }
            let count = children_count.saturating_sub(1).max(1);
            let space_between = total_space_between / count as f32;
            (main_size, 0.0, space_between)
        }
        Distribution::Start => (child_size.main, 0.0, 0.0),
        Distribution::End => {
            // TODO(bug)TODO(err): error message here is bogus "this needs to know size of this"
            // instead, this error should be created when computing child layout.
            let main_size = bounds.on(flow).why(this, names)?;
            (child_size.main, main_size - child_size.main, 0.0)
        }
    };
    let mut iter = to_update.iter_many_mut(children_entities);
    while let Some(mut space) = iter.fetch_next() {
        let child_cross_size = flow.relative(space.size).cross;
        let cross_offset = cross_align.offset(child_cross_size);
        space.pos.set_cross(flow, cross_offset);
        space.pos.set_main(flow, main_offset);
        main_offset += flow.orient(space.size) + space_between;
    }
    let oriented = Oriented::new(main_size, cross_size);
    Ok(flow.absolute(oriented))
}

// This functions' responsability is to compute the layout for `current` Entity,
// and all its children.
//
// Rules for this function:
//
// - Nodes will set **their own size** with the `to_update` query.
// - **the position of the children** will be set with `to_update`.
fn layout_at<F: ReadOnlyWorldQuery>(
    (this, node, children): QueryItem<NodeQuery>,
    flow: Flow,
    bounds: Bounds,
    to_update: &mut Query<&mut PosRect, F>,
    nodes: &Query<NodeQuery, F>,
    names: &Query<&Name>,
) -> Result<Oriented<f32>, error::Why> {
    let size = match *node {
        Node::Container(container) => match children {
            Some(children) => layout(container, this, children, bounds, to_update, nodes, names)?,
            None => return Err(error::Why::ChildlessContainer(Handle::of(this, names))),
        },
        Node::Axis(oriented) => bounds.inside(flow.absolute(oriented)).why(this, names)?,
        Node::Box(size) => bounds.inside(size).why(this, names)?,
    };
    if let Ok(mut to_update) = to_update.get_mut(this) {
        to_update.size = size;
    }
    Ok(flow.relative(size))
}
