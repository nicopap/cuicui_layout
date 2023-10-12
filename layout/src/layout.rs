//! The `cuicui_layout` algorithm.

use bevy::ecs::{prelude::*, query::ReadOnlyWorldQuery};
use bevy::log::trace;
use bevy::prelude::{Children, Name, Vec2};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent};
use bevy::utils::FloatOrd;

use crate::alignment::{Alignment, Distribution};
use crate::direction::{Flow, Oriented, Size};
use crate::error::{self, Computed, Handle, Relative};
use crate::rule::{LeafRule, Rule};

const WIDTH: Flow = Flow::Horizontal;
const HEIGHT: Flow = Flow::Vertical;

/// Position and size of a [`Node`] as computed by the layouting algo.
///
/// Note that [`pos`] will always be **relative to** the top left position of the
/// containing node.
///
/// [`pos`]: Self::pos
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
pub struct LayoutRect {
    pub(crate) size: Size<f32>,
    pub(crate) pos: Size<f32>,
}
impl LayoutRect {
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
#[cfg_attr(feature = "reflect", derive(Reflect))]
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
        Self {
            flow: Flow::Horizontal,
            align: Alignment::Center,
            distrib: Distribution::FillMain,
            margin: Size::ZERO,
            rules: Size::all(Rule::Parent(1.)),
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
            Distribution::Start => Rule::Children(1.),
            _ => Rule::Parent(1.),
        };
        let rules = flow.absolute(Oriented::new(main, Rule::Children(1.)));
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

/// A root [`Container`].
///
/// This acts as a [`Container`], but layouting "starts" from it.
///
/// Unlike a [`Container`], a `Root` never has a parent and its axis
/// are always [`Rule::Fixed`].
#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
pub struct Root {
    pub(crate) node: Container,
    /// Whether this node and its children should be displayed in the debug overlay.
    ///
    /// `true` by default. To debug layout, enable the `cuicui_layout/debug`
    /// cargo feature.
    pub debug: bool,
}
impl Default for Root {
    fn default() -> Self {
        Self { node: Container::default(), debug: true }
    }
}
impl Root {
    /// Get the [`Container`] in `Self`.
    #[must_use]
    pub const fn get(&self) -> &Container {
        &self.node
    }

    /// Get a mutable reference to the fixed size of this [`Root`] container
    #[must_use]
    pub fn size_mut(&mut self) -> Size<&mut f32> {
        use Rule::Fixed;
        let Size { width: Fixed(width), height: Fixed(height) } = &mut self.node.rules else {
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
        let Size { width: Fixed(width), height: Fixed(height) } = self.node.rules else {
            panic!("A Root container had an unfixed axis");
        };
        Size { width, height }
    }
    pub(crate) fn get_size(
        &self,
        entity: Entity,
        names: &Query<&Name>,
    ) -> Result<Size<Computed>, error::Why> {
        let to_child_rule = |rule| match rule {
            Rule::Fixed(pixels) => Some(Computed::Valid(pixels)),
            Rule::Children(ratio) => Some(Computed::ChildDefined(ratio, entity)),
            Rule::Parent(_) => None,
        };
        let Some(width) = to_child_rule(self.node.rules.width) else {
            return Err(error::Why::invalid_root(WIDTH, entity, names));
        };
        let Some(height) = to_child_rule(self.node.rules.height) else {
            return Err(error::Why::invalid_root(HEIGHT, entity, names));
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
        let node = Container { flow, align, distrib, rules, margin };
        Self { node, debug: true }
    }
}

/// A [`Component`] integrating the attached [`Entity`] in `cuicui_layout`.
#[derive(Component, Clone, Copy, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
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
        Self::Box(Size::all(LeafRule::Parent(1.)))
    }
}
impl Node {
    /// Is this node both terminal and content-sized?
    #[must_use]
    pub(crate) const fn content_sized(&self) -> bool {
        use LeafRule::Content;
        matches!(
            self,
            Self::Box(Size { width: Content(_), .. } | Size { height: Content(_), .. })
        )
    }
    /// A [`Node`] occupying `value%` of it's parent container on the main axis.
    ///
    /// Returns `None` if `value` is not between 0 and 100.
    #[must_use]
    pub fn spacer_percent(value: f32) -> Option<Self> {
        Self::spacer_ratio(value / 100.)
    }
    /// A [`Node`] occupying `value` ratio of it's parent container on the main axis.
    ///
    /// Returns `None` if `ratio` is not between 0 and 1.
    #[must_use]
    pub fn spacer_ratio(value: f32) -> Option<Self> {
        let spacer = Self::Axis(Oriented {
            main: LeafRule::Parent(value),
            cross: LeafRule::Fixed(0.),
        });
        (value <= 1. && value >= 0.).then_some(spacer)
    }
    /// A fixed size terminal [`Node`], without children.
    #[must_use]
    pub fn fixed(size: Size<f32>) -> Self {
        Self::Box(size.map(LeafRule::Fixed))
    }
    const fn parent_rule(&self, flow: Flow, axis: Flow) -> Option<f32> {
        match self {
            Self::Container(Container { rules, .. }) => {
                axis.relative(rules.as_ref()).main.parent_rule()
            }
            Self::Axis(oriented) => {
                let rules = flow.absolute(oriented.as_ref());
                axis.relative(rules).main.parent_rule()
            }
            Self::Box(rules) => axis.relative(rules.as_ref()).main.parent_rule(),
        }
    }
}

/// [`WorldQuery`] item used by the layout function.
///
/// [`WorldQuery`]: bevy::ecs::query::WorldQuery
pub(crate) type NodeQuery = (Entity, &'static Node, Option<&'static Children>);

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
    pub(crate) to_update: &'a mut Query<'w, 's, &'static mut LayoutRect, F>,
    pub(crate) nodes: &'a Query<'w, 's, NodeQuery, F>,
    pub(crate) names: &'a Query<'w, 's, &'static Name>,
}

impl<'a, 'w, 's, F: ReadOnlyWorldQuery> Layout<'a, 'w, 's, F> {
    pub(crate) fn new(
        this: Entity,
        to_update: &'a mut Query<'w, 's, &'static mut LayoutRect, F>,
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
        computed_size: Size<Computed>,
    ) -> Result<Size<f32>, error::Why> {
        let mut child_size = Oriented { main: 0., cross: 0. };
        let mut children_count: u32 = 0;

        let this_entity = self.this;
        for (this, node, children) in self.nodes.iter_many(children) {
            self.this = this;
            let Oriented { main, cross } = self.leaf(node, children, flow, computed_size)?;
            child_size.main += main;
            child_size.cross = child_size.cross.max(cross);
            children_count += 1;
        }
        self.this = this_entity;

        let size = flow.relative(computed_size).with_children(child_size);
        // TODO(BUG): Warn on cross max exceeds & children dependence
        if !distrib.overlaps() {
            self.validate_size(children, flow, child_size, size)?;
        }

        trace!("Setting offsets of children of {}", Handle::of(self));
        let single_child = children_count == 1;
        let count = children_count.saturating_sub(1).max(1) as f32;
        let cross_align = align.compute(size);
        let mut main_align = distrib.compute(size.main, child_size.main, single_child, count);
        let mut iter = self.to_update.iter_many_mut(children);
        while let Some(mut space) = iter.fetch_next() {
            let child_size = flow.relative(space.size);

            let offset = Oriented::new(
                main_align.offset(child_size.main),
                cross_align.offset(child_size.cross),
            );
            space.pos = flow.absolute(offset) + margin;
        }
        Ok(flow.absolute(size))
    }

    fn leaf(
        &mut self,
        node: &Node,
        children: Option<&Children>,
        flow: Flow,
        parent: Size<Computed>,
    ) -> Result<Oriented<f32>, error::Why> {
        let size = match *node {
            Node::Container(container) => match children {
                Some(children) => {
                    let margin = container.margin;
                    let computed_size = parent.container_size(&container, self);
                    let inner_size = self.container(container, children, computed_size?)?;
                    Size {
                        width: margin.width.mul_add(2., inner_size.width),
                        height: margin.height.mul_add(2., inner_size.height),
                    }
                }
                None => return Err(error::Why::ChildlessContainer(Handle::of(self))),
            },
            Node::Axis(oriented) => parent.leaf_size(flow.absolute(oriented)).transpose(self)?,
            Node::Box(size) => parent.leaf_size(size).transpose(self)?,
        };
        trace!("Setting size of {}", Handle::of(self));
        if let Ok(mut to_update) = self.to_update.get_mut(self.this) {
            to_update.size = size;
        }
        Ok(flow.relative(size))
    }

    fn validate_size(
        &self,
        children: &Children,
        flow: Flow,
        oriented_child_size: Oriented<f32>,
        oriented_size: Oriented<f32>,
    ) -> Result<(), error::Why> {
        let child_size = flow.absolute(oriented_child_size);
        let size = flow.absolute(oriented_size);

        if child_size.width <= size.width && child_size.height <= size.height {
            return Ok(());
        }
        let width_too_large = child_size.width > size.width;
        let axis = if width_too_large { WIDTH } else { HEIGHT };
        let largest_child = children.iter().max_by_key(|e| {
            let Ok(LayoutRect { size, .. }) = self.to_update.get(**e) else {
                return FloatOrd(0.);
            };
            FloatOrd(if width_too_large { size.width } else { size.height })
        });
        let relative_size = children.iter().filter_map(|e| {
            let node = self.nodes.get(*e).ok()?;
            node.1.parent_rule(flow, axis)
        });
        let relative_size = relative_size.sum();
        let largest_child = *largest_child.unwrap();
        Err(error::Why::ContainerOverflow {
            this: Handle::of(self),
            size,
            axis,
            node_children_count: u32::try_from(self.nodes.iter_many(children).count()).unwrap(),
            child_size: axis.relative(child_size).main,
            largest_child: Handle::of_entity(largest_child, self.names),
            child_relative_size: Relative::of(axis, flow, relative_size),
        })
    }
}
