//! Typed compile-time checked layouts.
//!
//! Use [`Container`] to create a hierarchy of containers.
//!
//! Note: this isn't the only way to build a layout, just a quick and dirty way.

use super::{Constraint, Direction, SpaceUse};

// TODO(feat): const constructors that panic at compile time if not within [0.0 1.0]
/// The container's size is equal to `.0` times what is containing it.
/// Must be within the range `[0.0, 1.0]` (inclusive)
pub struct Parent(pub f32);

/// The container's size is equal to `.0` times the largest of its child.
/// Must be greater or equal to 1.
pub struct Children(pub f32);

/// The container has a fixed size `.0` expressed in bevy world length.
/// (1.0 = 1 pixel in 2d)
pub struct Fixed(pub f32);

/// A constraint on the width or height of a [`Container`].
///
/// This enables expressing [`Spec`] at the type level and emitting
/// compile-time failures when a layout expressed in code is invalid.
pub trait Constrain {
    /// The [`Constraint`] the constraint represents.
    fn spec(&self) -> Constraint;
}
impl Constrain for Parent {
    fn spec(&self) -> Constraint {
        Constraint::Parent(self.0)
    }
}
impl Constrain for Children {
    fn spec(&self) -> Constraint {
        Constraint::Children(self.0)
    }
}
impl Constrain for Fixed {
    fn spec(&self) -> Constraint {
        Constraint::Fixed(self.0)
    }
}

trait MakeNode {
    fn node(&self) -> super::Node;
}

/// A constraint that doesn't need to know
pub trait AllowsUnconstrainedParent: Constrain {}
impl AllowsUnconstrainedParent for Fixed {}
impl AllowsUnconstrainedParent for Children {}

/// A constraint that isn't `Child`. It means that its children can
/// compute their own size based on their parent's size.
pub trait IsNotChild: Constrain {}
impl IsNotChild for Parent {}
impl IsNotChild for Fixed {}

/// A typed constructor for [`super::Container`].
///
/// The `W` and `H` parameters reprensent the constraints
/// on the `width` and `height` of the `Container`.
///
/// If the layout constraints do not allow it, a compilation error is
/// produced, generally looking like:
///
/// > the trait bound `Width: AllowsUnconstrainedParent` is not satisfied
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
/// - [`Children(multiple)`]: The container's size is equal to `multiple` times
///   the space its children occupies.
/// - [`Parent(fraction)`]: The size of the container is given `fraction` of
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
/// - This includes `spacers`, which depends on the size in the layout direction
///   of their direct parent.
///
/// Yeah I don't think there is any other failure mode.
///
/// ## Limitations
///
/// This doesn't eliminate all classes of errors.
/// For example, using a `Container::spacer` in a container going in a
/// direction which size is unconstrained compiles, but results in a runtime
/// error.
///
/// ## Usage
///
/// Use one of the constructor methods to make a container and attach children
/// to it using the `child` method.
///
/// ```
/// # use cuicui_layout::typed::*;
/// Container::v_stretch(Parent(1.), Parent(1.))
///     .child(
///         Container::h_stretch(Parent(1.0), Parent(0.1))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.)))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.)))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.))),
///     )
///     .child(
///         Container::h_stretch(Parent(1.0), Parent(0.1))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.)))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.)))
///             .child(Container::v_compact(Fixed(10.), Fixed(10.))),
///     );
/// ```
pub struct Container<W: Constrain, H: Constrain> {
    width: W,
    height: H,
    direction: Direction,
    space_use: SpaceUse,
    children: Vec<Box<dyn MakeNode>>,
}
struct Spacer(f32);
struct FixedNode(super::Size);
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
            direction: self.direction,
            space_use: self.space_use,
            width: self.width.spec(),
            height: self.height.spec(),
        })
    }
}
impl<W: Constrain, H: Constrain> Container<W, H> {
    pub fn fixed(mut self, width: f32, height: f32) -> Self {
        let size = super::Size { width, height };
        self.children.push(Box::new(FixedNode(size)));
        self
    }
}
impl<W: IsNotChild, H: IsNotChild> Container<W, H> {
    pub fn child<Width, Height>(mut self, child: Container<Width, Height>) -> Self
    where
        Width: Constrain + 'static,
        Height: Constrain + 'static,
    {
        self.children.push(Box::new(child));
        self
    }
}
impl<W: IsNotChild> Container<W, Children> {
    pub fn child<Width, Height>(mut self, child: Container<Width, Height>) -> Self
    where
        Width: Constrain + 'static,
        Height: AllowsUnconstrainedParent + 'static,
    {
        self.children.push(Box::new(child));
        self
    }
}
impl<H: IsNotChild> Container<Children, H> {
    pub fn child<Width, Height>(mut self, child: Container<Width, Height>) -> Self
    where
        Width: AllowsUnconstrainedParent + 'static,
        Height: Constrain + 'static,
    {
        self.children.push(Box::new(child));
        self
    }
}
impl<W: Constrain, H: Constrain> Container<W, H> {
    /// Vertically aligned (bottom to top), children stretch to fill
    /// the whole height of the container.
    pub const fn v_stretch(width: W, height: H) -> Self {
        Self::new(width, height, Direction::Vertical, SpaceUse::Stretch)
    }
    /// Horizontally aligned (left to right), children stretch to fill
    /// the whole width of the container.
    pub const fn h_stretch(width: W, height: H) -> Self {
        Self::new(width, height, Direction::Horizontal, SpaceUse::Stretch)
    }
    pub const fn v_compact(width: W, height: H) -> Self {
        Self::new(width, height, Direction::Vertical, SpaceUse::Compact)
    }
    pub const fn h_compact(width: W, height: H) -> Self {
        Self::new(width, height, Direction::Horizontal, SpaceUse::Compact)
    }
    pub const fn new(width: W, height: H, direction: Direction, space_use: SpaceUse) -> Self {
        Self {
            width,
            height,
            direction,
            space_use,
            children: Vec::new(),
        }
    }
}
