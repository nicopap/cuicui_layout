//! Typed compile-time checked layouts.
//!
//! Use [`Container`] to create a hierarchy of containers.
//!
//! Note: this isn't the only way to build a layout, just a quick and dirty way.
#![allow(missing_docs)]
#![deprecated = "The constructors in this module may be type safe, but are compltely \
    useless as a way to create a UI. Prefer \
    cuicui_layout_bevy_ui::LayoutCommandsExt until a more general layout \
    declaration system is available."]

use super::{Alignment, Distribution, Flow, Rule};

/// The container's size is equal to `.0` times what is containing it.
/// Must be within the range `[0.0, 1.0]` (inclusive)
pub struct Parent(f32);
impl Parent {
    /// Create a new [`Parent`].
    ///
    /// # Panics
    ///
    /// When `value` is not in the range `[0.0, 1.0]` (inclusive)
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::float_cmp)]
    pub fn new(value: f32) -> Self {
        assert!(
            value as i32 == 0 || value == 1.0,
            "Invalid `Parent` constraint, it was not between 0 and 1, while it should!",
        );
        Parent(value)
    }
}

/// The container's size is equal to `.0` times the largest of its child.
/// Must be greater or equal to 1.
pub struct Children(f32);
impl Children {
    /// Create a new [`Children`].
    ///
    /// # Panics
    ///
    /// When `value` is greater than 1
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn new(value: f32) -> Self {
        assert!(
            value as i32 >= 1,
            "Invalid `Children` constraint, it was not greater than 1, while it should!",
        );
        Children(value)
    }
}

/// The container has a fixed size `.0` expressed in bevy world length.
/// (1.0 = 1 pixel in 2d)
pub struct Fixed(pub f32);

/// A constraint on the width or height of a [`Container`].
///
/// This enables expressing [`Rule`] at the type level and emitting
/// compile-time failures when a layout expressed in code is invalid.
pub trait Constrain {
    /// The [`Rule`] the constraint represents.
    fn rule(&self) -> Rule;
}
impl Constrain for Parent {
    fn rule(&self) -> Rule {
        Rule::Parent(self.0)
    }
}
impl Constrain for Children {
    fn rule(&self) -> Rule {
        Rule::Children(self.0)
    }
}
impl Constrain for Fixed {
    fn rule(&self) -> Rule {
        Rule::Fixed(self.0)
    }
}

trait MakeNode {
    fn node(&self) -> super::Node;
}

/// We don't care about the parent size.
pub trait FreeParent: Constrain {}
impl FreeParent for Fixed {}
impl FreeParent for Children {}

/// We don't care about the children size.
pub trait FreeChildren: Constrain {}
impl FreeChildren for Parent {}
impl FreeChildren for Fixed {}

/// A typed constructor for [`super::Container`].
///
/// The `W` and `H` parameters reprensent the constraints
/// on the `width` and `height` of the `Container`.
///
/// If the layout constraints do not allow it, a compilation error is
/// produced, generally looking like:
///
/// ```text
/// the trait bound `Width: FreeParent` is not satisfied
/// ```
///
/// ## Jargon
///
/// **size** in this documentation usually means **either width or height**,
/// it's just faster to say "size" than "width/height".
///
/// If we are talking about `size` as `width * height`, we will call it
/// "total size".
///
/// `Container`s are [`Constrain`]ed on their `Width` and `Height`,
/// a constraint tells how to compute the [`Container`]'s total size.
///
/// cuicui layout supports the following constraints:
///
/// - [`Fixed(size)`]: The size of the container is set at the given length,
///   it is expressed in pixels in the default bevy setup
/// - [`Children::new(multiple)`]: The container's size is equal to `multiple` times
///   the space its children occupies.
/// - [`Parent::new(fraction)`]: The size of the container is given `fraction` of
///   its parent's size. `fraction` must be within the range `[0.0, 1.0]`
///   (inclusive)
///
/// ## Constraints
///
/// cuicui layout explicitly fails if it can't infere a proper size for a node
/// based on the provided constraints. cuicui layout cannot infer a size for
/// a node if:
///
/// - Given `Node` size depends on the size of its parent, while its parent
///   depends on the size of `Node`
/// - This includes `spacers`, which depends on the size in the layout flow
///   of their direct parent.
///
/// Yeah I don't think there is any other failure mode.
///
/// ## Limitations
///
/// This doesn't eliminate all classes of errors.
/// For example, using a `Container::spacer` in a container going in a
/// flow which size is unconstrained compiles, but results in a runtime
/// error.
///
/// ## Usage
///
/// Use one of the constructor methods to make a container and attach children
/// to it using the `child` method.
///
/// ```
/// # use cuicui_layout::typed::*;
/// Container::v_stretch(Parent::new(1.), Parent::new(1.))
///     .child(
///         Container::h_stretch(Parent::new(1.0), Parent::new(0.1))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.)))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.)))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.))),
///     )
///     .child(
///         Container::h_stretch(Parent::new(1.0), Parent::new(0.1))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.)))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.)))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.))),
///     );
/// ```
pub struct Container<W: Constrain, H: Constrain> {
    width: W,
    height: H,
    flow: Flow,
    align: Alignment,
    distrib: Distribution,
    children: Vec<Box<dyn MakeNode>>,
}
struct Spacer(f32);
struct FixedNode(super::Size<f32>);
impl MakeNode for Spacer {
    fn node(&self) -> super::Node {
        super::Node::spacer_ratio(self.0).unwrap()
    }
}
impl MakeNode for FixedNode {
    fn node(&self) -> super::Node {
        super::Node::fixed(self.0)
    }
}
impl<W: Constrain, H: Constrain> MakeNode for Container<W, H> {
    fn node(&self) -> super::Node {
        super::Node::Container(super::Container {
            flow: self.flow,
            align: self.align,
            distrib: self.distrib,
            size: super::Size {
                width: self.width.rule(),
                height: self.height.rule(),
            },
        })
    }
}
impl<W: Constrain, H: Constrain> Container<W, H> {
    /// Add a fixed-size child to this container.
    #[must_use]
    pub fn fixed(mut self, width: f32, height: f32) -> Self {
        let size = super::Size { width, height };
        self.children.push(Box::new(FixedNode(size)));
        self
    }
}
impl<W: FreeChildren, H: FreeChildren> Container<W, H> {
    /// Add a container to this container, the added container will have its
    /// size depend on its own children.
    #[must_use]
    pub fn child<Width, Height>(mut self, child: Container<Width, Height>) -> Self
    where
        Width: Constrain + 'static,
        Height: Constrain + 'static,
    {
        self.children.push(Box::new(child));
        self
    }
}
impl<W: FreeChildren> Container<W, Children> {
    /// Add a container to this container, the added container will have its
    /// size depend on its own children.
    #[must_use]
    pub fn child<Width, Height>(mut self, child: Container<Width, Height>) -> Self
    where
        Width: Constrain + 'static,
        Height: FreeParent + 'static,
    {
        self.children.push(Box::new(child));
        self
    }
}
impl<H: FreeChildren> Container<Children, H> {
    /// Add a container to this container, the added container will have its
    /// size depend on its own children.
    #[must_use]
    pub fn child<Width, Height>(mut self, child: Container<Width, Height>) -> Self
    where
        Width: FreeParent + 'static,
        Height: Constrain + 'static,
    {
        self.children.push(Box::new(child));
        self
    }
}
impl<W: Constrain, H: Constrain> Container<W, H> {
    /// Vertically aligned (bottom to top), children stretch to fill
    /// the whole height of the container.
    #[must_use]
    pub const fn v_stretch(width: W, height: H) -> Self {
        Self::stretch(width, height, Flow::Vertical)
    }
    /// Horizontally aligned (left to right), children stretch to fill
    /// the whole width of the container.
    #[must_use]
    pub const fn h_stretch(width: W, height: H) -> Self {
        Self::stretch(width, height, Flow::Horizontal)
    }
    #[must_use]
    pub const fn v_compact(width: W, height: H) -> Self {
        Self::compact(width, height, Flow::Vertical)
    }
    #[must_use]
    pub const fn h_compact(width: W, height: H) -> Self {
        Self::compact(width, height, Flow::Horizontal)
    }
    #[must_use]
    pub const fn stretch(width: W, height: H, flow: Flow) -> Self {
        Self {
            width,
            height,
            flow,
            align: Alignment::Center,
            distrib: Distribution::FillMain,
            children: Vec::new(),
        }
    }
    #[must_use]
    pub const fn compact(width: W, height: H, flow: Flow) -> Self {
        Self {
            width,
            height,
            flow,
            align: Alignment::Start,
            distrib: Distribution::Start,
            children: Vec::new(),
        }
    }
}
