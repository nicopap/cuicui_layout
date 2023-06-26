#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect, ReflectComponent};
use bevy::{
    ecs::query::QueryItem,
    prelude::{Children, Component, Entity, Name, Query},
};

use crate::{
    alignment::{Align, Alignment, Distribution},
    direction::{Flow, Oriented, Size},
    error::{self, BadParent, Bound, Bounds},
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

// TODO(clean): Split out `size` so that I can re-use it in `Root`
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Container {
    pub flow: Flow,
    pub align: Alignment,
    pub distrib: Distribution,
    pub size: Size<Rule>,
}
impl Default for Container {
    fn default() -> Self {
        Container {
            flow: Flow::Horizontal,
            align: Alignment::Center,
            distrib: Distribution::FillParent,
            size: Size {
                width: Rule::Parent(1.0),
                height: Rule::Parent(1.0),
            },
        }
    }
}

#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect), reflect(Component))]
pub enum Node {
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
    /// An invisible [`Node`] occupying `value%` of it's parent container
    /// on the main axis.
    pub fn spacer_percent(value: f32) -> Option<Self> {
        Self::spacer_ratio(value / 100.0)
    }
    /// An invisible [`Node`] occupying `value` ratio of it's parent container
    /// on the main axis.
    pub fn spacer_ratio(value: f32) -> Option<Self> {
        let spacer = Node::Axis(Oriented {
            main: LeafRule::Parent(value),
            cross: LeafRule::Fixed(0.0),
        });
        (value <= 1.0 && value >= 0.0).then_some(spacer)
    }

    pub fn fixed(size: Size<f32>) -> Node {
        Node::Box(size.map(LeafRule::Fixed))
    }
}

/// A constraint on an axis of a terminal `Node` (ie: doesn't have a `Children` constraint).
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum LeafRule {
    /// The container's size is equal to its parent's size  times `.0`.
    /// (may not be above 1)
    Parent(f32),
    /// The container's size is equal to precisely `.0` pixels.
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
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum Rule {
    /// The container's size is equal to the total size of all its children
    /// times `.0`. (may not be below 1)
    Children(f32),
    /// The container's size is equal to its parent's size  times `.0`.
    /// (may not be above 1)
    Parent(f32),
    /// The container's size is equal to precisely `.0` pixels.
    Fixed(f32),
}
impl Default for Rule {
    fn default() -> Self {
        Rule::Children(1.0)
    }
}

#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Root {
    pub bounds: Size<f32>,
    pub flow: Flow,
    pub align: Alignment,
    pub distrib: Distribution,
}
impl Root {
    pub const fn new(
        bounds: Size<f32>,
        flow: Flow,
        align: Alignment,
        distrib: Distribution,
    ) -> Self {
        Root { bounds, align, distrib, flow }
    }
    pub const fn stretch(bounds: Size<f32>, flow: Flow) -> Self {
        Root::new(bounds, flow, Alignment::Center, Distribution::FillParent)
    }
    pub const fn compact(bounds: Size<f32>, flow: Flow) -> Self {
        Root::new(bounds, flow, Alignment::Start, Distribution::Start)
    }
}

pub type LayoutNode = (Entity, &'static Node, &'static Children);

impl Container {
    pub const fn new(flow: Flow, align: Alignment, distrib: Distribution) -> Self {
        let main = match distrib {
            Distribution::FillParent => Rule::Parent(1.0),
            Distribution::Start | Distribution::End => Rule::Children(1.0),
        };
        let cross = Rule::Children(1.0);
        let size = flow.absolute(Oriented::new(main, cross));
        Self { flow, distrib, align, size }
    }
    pub const fn stretch(flow: Flow) -> Self {
        Self::new(flow, Alignment::Center, Distribution::FillParent)
    }
    pub const fn compact(flow: Flow) -> Self {
        Self::new(flow, Alignment::Start, Distribution::Start)
    }
    pub(crate) fn layout(
        &self,
        this: Entity,
        children_entities: &Children,
        bounds: Bounds,
        to_update: &mut Query<&mut PosRect>,
        nodes: &Query<LayoutNode>,
        names: &Query<&Name>,
    ) -> Result<Size<f32>, error::Why> {
        let Self { flow, distrib, align, size } = *self;

        if children_entities.is_empty() {
            return Ok(Size::ZERO);
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
            Distribution::FillParent => {
                let total_space_between = bounds.on(flow).why(this, names)? - child_size.main;
                if total_space_between < 0.0 {
                    return Err(error::Why::ContainerOverflow {
                        this: error::Handle::of(this, names),
                        bounds,
                        node_children_count: children_count,
                        dir_name: flow.size_name(),
                        child_size: child_size.main,
                    });
                }
                let space_between = total_space_between / (children_count - 1) as f32;
                (bounds.on(flow).why(this, names)?, 0.0, space_between)
            }
            Distribution::Start => (child_size.main, 0.0, 0.0),
            Distribution::End => {
                let main_parent_size = bounds.on(flow).why(this, names)?;
                (child_size.main, main_parent_size - child_size.main, 0.0)
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
    flow: Flow,
    bounds: Bounds,
    to_update: &mut Query<&mut PosRect>,
    nodes: &Query<LayoutNode>,
    names: &Query<&Name>,
) -> Result<Oriented<f32>, error::Why> {
    let size = match node {
        Node::Container(container) => {
            container.layout(this, children, bounds, to_update, nodes, names)?
        }
        &Node::Axis(oriented) => bounds.inside(flow.absolute(oriented)).why(this, names)?,
        &Node::Box(size) => bounds.inside(size).why(this, names)?,
    };
    if let Ok(mut to_update) = to_update.get_mut(this) {
        to_update.size = size;
    }
    Ok(flow.relative(size))
}
