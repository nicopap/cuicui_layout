//! Structs to help convert between a relative and absolute direction.
use std::fmt;

#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect};

/// A `T` that applies to the `width` and `height` of something.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Size<T> {
    /// `T` on the horizontal axis.
    pub width: T,
    /// `T` on the vertical axis.
    pub height: T,
}

/// Similar to [`Size`], but relative to a [`Flow`] direction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Oriented<T> {
    /// `T` on the same axis as the [`Flow`].
    pub main: T,
    /// `T` on the perpendicular axis of the [`Flow`].
    pub cross: T,
}

/// The layout direction of a [`Container`].
///
/// [`Container`]: crate::Container
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum Flow {
    /// Children are arranged on the horizontal axis.
    #[default]
    Horizontal,

    /// Children are arranged on the vertical axis.
    Vertical,
}
impl Flow {
    pub(crate) const fn orient<T: Copy>(self, size: Size<T>) -> T {
        self.relative(size).main
    }
    /// Returns [`Size`] oriented according to this orientation.
    ///
    /// This is the inverse of [`Oriented::absolute`].
    pub(crate) const fn relative<T: Copy>(self, Size { width, height }: Size<T>) -> Oriented<T> {
        let Size { width: main, height: cross } = self.absolute(Oriented::new(width, height));
        Oriented { main, cross }
    }
    /// Returns [`Oriented`] in oriented according to the global point of view.
    ///
    /// This is the inverse of [`Oriented::relative`].
    pub(crate) const fn absolute<T: Copy>(self, Oriented { main, cross }: Oriented<T>) -> Size<T> {
        match self {
            Flow::Horizontal => Size::new(main, cross),
            Flow::Vertical => Size::new(cross, main),
        }
    }
    /// Perpendicular orientation.
    pub(crate) const fn perp(self) -> Self {
        self.orient(Size::new(Flow::Vertical, Flow::Horizontal))
    }
    pub(crate) const fn size_name(self) -> &'static str {
        self.orient(Size::new("width", "height"))
    }
}

impl<T> Size<T> {
    /// Create a [`Size`] for given `width` and `height` `T`.
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
    /// Create a [`Size`] where `width` and `height` are set to `value`.
    pub fn all(value: T) -> Self
    where
        T: Clone,
    {
        Size { width: value.clone(), height: value }
    }
    /// Apply `f` on `width` and `height`, returning a `Size` with the output
    /// values of `f`.
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Size<U> {
        Size { width: f(self.width), height: f(self.height) }
    }
    /// Set `value` on axis perpendicular to `flow` direction.
    pub fn set_cross(&mut self, flow: Flow, value: T) {
        match flow {
            Flow::Horizontal => self.height = value,
            Flow::Vertical => self.width = value,
        }
    }
    /// Set `value` on axis of `flow` direction.
    pub fn set_main(&mut self, flow: Flow, value: T) {
        match flow {
            Flow::Horizontal => self.width = value,
            Flow::Vertical => self.height = value,
        }
    }
}
impl<T: Copy> Size<T> {
    pub(crate) const fn on(self, flow: Flow) -> T {
        flow.orient(self)
    }
}

impl<T: Copy> Oriented<T> {
    /// Create an [`Oriented`] for given `main` and `cross` `T`.
    pub const fn new(main: T, cross: T) -> Self {
        Self { main, cross }
    }
}

impl<T: fmt::Display> fmt::Display for Size<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}×{}", self.width, self.height)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative() {
        let Oriented { main: main_v, cross: cross_v } =
            Flow::Vertical.relative(Size::new("width", "height"));
        let Oriented { main: main_h, cross: cross_h } =
            Flow::Horizontal.relative(Size::new("width", "height"));

        assert_eq!(main_v, cross_h);
        assert_eq!(main_h, cross_v);
    }
    #[test]
    fn absolute() {
        let Size { width: width_v, height: height_v } =
            Flow::Vertical.absolute(Oriented::new("main", "cross"));
        let Size { width: width_h, height: height_h } =
            Flow::Horizontal.absolute(Oriented::new("main", "cross"));

        assert_eq!(width_v, height_h);
        assert_eq!(width_h, height_v);
    }
}
