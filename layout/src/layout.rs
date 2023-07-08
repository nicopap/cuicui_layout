//! The `cuicui_layout` algorithm.

use std::{num::ParseFloatError, str::FromStr};

#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect, ReflectComponent};
use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{Children, Component, Entity, Name, Query},
};

const WIDTH: Flow = Flow::Horizontal;
const HEIGHT: Flow = Flow::Vertical;

use crate::{
    alignment::{Alignment, CrossAlign, Distribution},
    direction::{Flow, Oriented, Size},
    error::{self, Computed, Handle},
    PosRect,
};
impl<T> Size<Result<T, Entity>> {
    /// Go from a `Size<Result<T, Entity>>` to a `Result<Size<T>, error::Why>`.
    /// Assumes the error is a [`error::Why::CyclicRule`].
    fn transpose(self, queries: &Layout<impl ReadOnlyWorldQuery>) -> Result<Size<T>, error::Why> {
        let err = |flow, e: Entity| error::Why::bad_rule(flow, e, queries);
        let width = self.width.map_err(|e| err(WIDTH, e))?;
        let height = self.height.map_err(|e| err(HEIGHT, e))?;
        Ok(Size { width, height })
    }
}

impl Oriented<Computed> {
    fn with_children(self, Oriented { main, cross }: Oriented<f32>) -> Oriented<f32> {
        Oriented {
            main: self.main.with_child(main),
            cross: self.cross.with_child(cross),
        }
    }
}
impl Size<Computed> {
    pub(crate) fn set_margin(
        &mut self,
        margin: Size<f32>,
        queries: &Layout<impl ReadOnlyWorldQuery>,
    ) -> Result<(), error::Why> {
        if let Computed::Valid(width) = &mut self.width {
            // TODO(feat): This is where I'd set the margin
            if *width < 2. * margin.width {
                return Err(error::Why::TooMuchMargin {
                    this: Handle::of(queries),
                    axis: WIDTH,
                    margin: margin.width,
                    this_size: *width,
                });
            }
            if margin.width.is_sign_negative() {
                return Err(error::Why::NegativeMargin {
                    this: Handle::of(queries),
                    axis: WIDTH,
                    margin: margin.width,
                });
            }
            *width -= 2. * margin.width;
        }
        if let Computed::Valid(height) = &mut self.height {
            if *height < 2. * margin.height {
                return Err(error::Why::TooMuchMargin {
                    this: Handle::of(queries),
                    axis: HEIGHT,
                    margin: margin.height,
                    this_size: *height,
                });
            }
            if margin.height.is_sign_negative() {
                return Err(error::Why::NegativeMargin {
                    this: Handle::of(queries),
                    axis: HEIGHT,
                    margin: margin.height,
                });
            }
            *height -= 2. * margin.height;
        }
        Ok(())
    }

    fn container_size(
        self,
        Container { rules, margin, .. }: &Container,
        queries: &Layout<impl ReadOnlyWorldQuery>,
    ) -> Result<Self, error::Why> {
        let bounds = Size {
            width: rules.width.inside(self.width, queries.this),
            height: rules.height.inside(self.height, queries.this),
        };
        let mut bounds = bounds.transpose(queries)?;
        bounds.set_margin(*margin, queries)?;

        Ok(bounds)
    }

    fn leaf_size(self, Size { width, height }: Size<LeafRule>) -> Size<Result<f32, Entity>> {
        Size {
            width: width.inside(self.width),
            height: height.inside(self.height),
        }
    }
}

/// Parameters of a container, ie: a node that contains other nodes.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
// TODO(clean): Split out `size` so that I can re-use it in `Root`
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
    pub rules: Size<Rule>,

    /// The empty space to leave between this `Container` and its content, in pixels.
    ///
    /// Note that margins are symetric, so that left/right and top/bottom margins
    /// are identical.
    ///
    /// Note also that when a child is [`Rule::Parent`], it will substract the margin
    /// of the parent container when calculating its own size.
    pub margin: Size<f32>,
}
impl Default for Container {
    fn default() -> Self {
        Container {
            flow: Flow::Horizontal,
            align: Alignment::Center,
            distrib: Distribution::FillMain,
            margin: Size::ZERO,
            rules: Size::all(Rule::Parent(1.0)),
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
        let rules = flow.absolute(Oriented::new(main, Rule::Children(1.0)));
        let margin = Size::ZERO;
        Self { flow, align, distrib, rules, margin }
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
pub struct Root(Container);
impl Root {
    /// Get the [`Container`] in `Self`.
    #[must_use]
    pub const fn get(&self) -> &Container {
        &self.0
    }

    /// Get a mutable reference to the fixed size of this [`Root`] container
    #[must_use]
    pub fn size_mut(&mut self) -> Size<&mut f32> {
        use Rule::Fixed;
        let Size { width: Fixed(width), height: Fixed(height) } = &mut self.0.rules else {
            unreachable!("Can't construct a `Root` with non-fixed size");
        };
        Size { width, height }
    }
    /// Get the fixed size of this [`Root`] container.
    ///
    /// # Panics
    /// If one of the axis is unfixed.
    ///
    /// Normally, it is impossible to construct a `Root` with unfixed axis,
    /// but it is possible to accomplish it by modifying `Root` through reflection.
    #[must_use]
    pub const fn size(&self) -> Size<f32> {
        use Rule::Fixed;
        let Size { width: Fixed(width), height: Fixed(height) } = self.0.rules else {
            panic!("A Root container had an unfixed axis");
        };
        Size { width, height }
    }
    pub(crate) fn get_size(
        &self,
        entity: Entity,
        names: &Query<&Name>,
    ) -> Result<Size<f32>, error::Why> {
        use Rule::Fixed;
        let Size { width: Fixed(width), height: Fixed(height) } = self.0.rules else {
            let width_fix = matches!(self.0.rules.width, Fixed(_));
            let axis = if width_fix { HEIGHT } else { WIDTH };
            return Err(error::Why::invalid_root(axis, entity, names));
        };
        Ok(Size { width, height })
    }
    /// Create a new [`Root`] with given parameters.
    #[must_use]
    pub const fn new(
        Size { width, height }: Size<f32>,
        flow: Flow,
        align: Alignment,
        distrib: Distribution,
        margin: Size<f32>,
    ) -> Self {
        use Rule::Fixed;
        let rules = Size::new(Fixed(width), Fixed(height));
        Root(Container { flow, align, distrib, rules, margin })
    }
    /// Create a [`Root`] container where children are center-aligned and
    /// fill this container on the `flow` main axis.
    #[must_use]
    pub const fn stretch(bounds: Size<f32>, flow: Flow) -> Self {
        let distrib = Distribution::FillMain;
        Root::new(bounds, flow, Alignment::Center, distrib, Size::ZERO)
    }
    /// Create a [`Root`] container where children are compactly bunched at the
    /// start of the main and cross axis.
    #[must_use]
    pub const fn compact(bounds: Size<f32>, flow: Flow) -> Self {
        let distrib = Distribution::Start;
        Root::new(bounds, flow, Alignment::Start, distrib, Size::ZERO)
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
    /// > **IMPORTANT**: When [`Rule::Children`] is used on a container's size,
    /// > none of its children may depend on its parent size. It would lead to
    /// > a circular dependency.
    Children(f32),

    /// The container's size is equal to its parent's size  times `f32`.
    /// (may not be above 1)
    ///
    /// > **IMPORTANT**: this is the size of of the container **within margin**.
    /// > ie: it is the possible space the children can occupy, the total size
    /// > is substracted that of the margins.
    Parent(f32),

    /// The container's size is equal to precisely `f32` pixels.
    Fixed(f32),
}
impl Default for Rule {
    fn default() -> Self {
        Rule::Children(1.0)
    }
}
impl FromStr for Rule {
    type Err = Option<ParseFloatError>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(pixels) = s.strip_suffix("px") {
            let pixels = pixels.parse()?;
            if pixels < 0.0 {
                return Err(None);
            }
            Ok(Self::Fixed(pixels))
        } else if let Some(percents) = s.strip_suffix('%') {
            let percents: f32 = percents.parse()?;
            if percents > 100.0 || percents < 0.0 {
                return Err(None);
            }
            Ok(Self::Parent(percents / 100.0))
        } else if let Some(child_ratio) = s.strip_suffix('*') {
            let ratio: f32 = child_ratio.parse()?;
            if ratio > 1.0 || ratio < 0.0 {
                return Err(None);
            }
            Ok(Self::Children(ratio))
        } else {
            Err(None)
        }
    }
}

impl LeafRule {
    /// Compute effective size, given a potentially set parent container size.
    fn inside(self, parent_size: Computed) -> Result<f32, Entity> {
        match (self, parent_size) {
            (LeafRule::Parent(ratio), Computed::Valid(value)) => Ok(value * ratio),
            (LeafRule::Parent(_), Computed::ChildDefined(_, parent)) => Err(parent),
            (LeafRule::Fixed(fixed), _) => Ok(fixed),
        }
    }
}
impl Rule {
    /// Compute effective size, given a potentially set parent container size.
    fn inside(self, parent_size: Computed, this: Entity) -> Result<Computed, Entity> {
        use Computed::{ChildDefined, Valid};
        match (self, parent_size) {
            (Rule::Parent(ratio), Valid(value)) => Ok(Valid(value * ratio)),
            (Rule::Parent(_), ChildDefined(_, parent)) => Err(parent),
            (Rule::Fixed(fixed), _) => Ok(Valid(fixed)),
            (Rule::Children(ratio), ChildDefined(_, parent)) => Ok(ChildDefined(ratio, parent)),
            (Rule::Children(ratio), _) => Ok(ChildDefined(ratio, this)),
        }
    }
}

/// [`WorldQuery`] item used by the layout function.
///
/// [`WorldQuery`]: bevy::ecs::query::WorldQuery
pub type NodeQuery = (Entity, &'static Node, Option<&'static Children>);

/// The layouting algorithm's inner state.
///
/// The algo runs as follow:
///
/// 1. Compute the current container's size (or delegate to children if
///    the size is child-dependent)
/// 2. Compute each child's size. (note that this goes to step 1 for each child)
/// 3. Now, we resolve the current container's size if it is child-dependent.
/// 4. Compute each child's offset. (It is necessary to know the container's size
///    to compute children's offset if [`Distribution::FillMain`] or [`Distribution::End`]).
///    (It is also necessary to know each child's size to place them next to each-other)
///
/// Done.
pub struct Layout<'a, 'w, 's, F: ReadOnlyWorldQuery> {
    // This container's entity
    pub(crate) this: Entity,
    pub(crate) to_update: &'a mut Query<'w, 's, &'static mut PosRect, F>,
    pub(crate) nodes: &'a Query<'w, 's, NodeQuery, F>,
    pub(crate) names: &'a Query<'w, 's, &'static Name>,
}

impl<'a, 'w, 's, F: ReadOnlyWorldQuery> Layout<'a, 'w, 's, F> {
    pub(crate) fn new(
        this: Entity,
        to_update: &'a mut Query<'w, 's, &'static mut PosRect, F>,
        nodes: &'a Query<'w, 's, NodeQuery, F>,
        names: &'a Query<'w, 's, &'static Name>,
    ) -> Self {
        Self { this, to_update, nodes, names }
    }

    /// Compute layout for a [`Container`].
    ///
    /// `computed_size` is this container's _inner size_.
    /// ie: the size of the container **removed the margin**.
    ///
    /// Returns the _inner size_, it is likely that adding back the margins is
    /// necessary.
    #[allow(clippy::cast_precision_loss)] // count as f32
    pub(crate) fn container(
        &mut self,
        Container { flow, distrib, align, margin, .. }: Container,
        children: &Children,
        // Size of this container
        computed_size: Size<Computed>,
    ) -> Result<Size<f32>, error::Why> {
        if children.is_empty() {
            unreachable!("A bevy bug caused the `bevy_hierarchy::Children` component to be empty")
        }
        let mut child_size = Oriented { main: 0.0, cross: 0.0 };
        let mut children_count = 0;

        let this_entity = self.this;
        for (this, node, children) in self.nodes.iter_many(children) {
            self.this = this;
            let Oriented { main, cross } = self.node(node, children, flow, computed_size)?;
            child_size.main += main;
            child_size.cross = child_size.cross.max(cross);
            children_count += 1;
        }
        self.this = this_entity;

        let size = flow.relative(computed_size).with_children(child_size);

        // Error on overflow (TODO(clean): consider logging instead)
        self.validate_size(
            children_count,
            flow.absolute(child_size),
            flow.absolute(size),
        )?;

        let count = children_count.saturating_sub(1).max(1) as f32;
        let (main_offset, space_between) = match distrib {
            Distribution::FillMain => (0.0, (size.main - child_size.main) / count),
            Distribution::Start => (0.0, 0.0),
            Distribution::End => (size.main - child_size.main, 0.0),
        };

        let margin = flow.relative(margin);
        let mut offset = Oriented::new(main_offset + margin.main, 0.0);

        let cross_align = CrossAlign::new(size, align);
        let mut iter = self.to_update.iter_many_mut(children);
        while let Some(mut space) = iter.fetch_next() {
            let child_size = flow.relative(space.size);

            offset.cross = cross_align.offset(child_size.cross) + margin.cross;
            space.pos = flow.absolute(offset);
            offset.main += child_size.main + space_between;
        }
        Ok(flow.absolute(size))
    }

    fn node(
        &mut self,
        node: &Node,
        children: Option<&Children>,
        flow: Flow,
        parent: Size<Computed>,
    ) -> Result<Oriented<f32>, error::Why> {
        let size = match *node {
            Node::Container(container) => match children {
                Some(children) => {
                    let computed_size = parent.container_size(&container, self);
                    let mut inner_size = self.container(container, children, computed_size?)?;
                    inner_size.width += container.margin.width * 2.;
                    inner_size.height += container.margin.height * 2.;
                    inner_size
                }
                None => return Err(error::Why::ChildlessContainer(Handle::of(self))),
            },
            Node::Axis(oriented) => parent.leaf_size(flow.absolute(oriented)).transpose(self)?,
            Node::Box(size) => parent.leaf_size(size).transpose(self)?,
        };
        if let Ok(mut to_update) = self.to_update.get_mut(self.this) {
            to_update.size = size;
        }
        Ok(flow.relative(size))
    }

    fn validate_size(
        &self,
        node_children_count: u32,
        child_size: Size<f32>,
        size: Size<f32>,
    ) -> Result<(), error::Why> {
        if child_size.width > size.width {
            return Err(error::Why::ContainerOverflow {
                this: Handle::of(self),
                size,
                node_children_count,
                axis: WIDTH,
                child_size: child_size.width,
            });
        }
        if child_size.height > size.height {
            return Err(error::Why::ContainerOverflow {
                this: Handle::of(self),
                size,
                node_children_count,
                axis: HEIGHT,
                child_size: child_size.height,
            });
        }
        Ok(())
    }
}
