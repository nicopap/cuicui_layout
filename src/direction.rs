use std::fmt;

#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Oriented<T> {
    pub main: T,
    pub cross: T,
}

/// The layout direction of a [`Container`].
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum Direction {
    /// Children are arranged on the horizontal axis.
    Horizontal,

    /// Children are arranged on the vertical axis.
    Vertical,
}
impl Direction {
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
            Direction::Horizontal => Size::new(main, cross),
            Direction::Vertical => Size::new(cross, main),
        }
    }
    /// Perpendicular orientation.
    pub(crate) const fn perp(self) -> Self {
        self.orient(Size::new(Direction::Vertical, Direction::Horizontal))
    }
    pub(crate) const fn size_name(&self) -> &'static str {
        self.orient(Size::new("width", "height"))
    }
}

impl Size<f32> {
    pub const ZERO: Self = Size { width: 0.0, height: 0.0 };
}
impl<T> Size<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
    pub fn all(value: T) -> Self
    where
        T: Clone,
    {
        Size { width: value.clone(), height: value }
    }
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Size<U> {
        Size { width: f(self.width), height: f(self.height) }
    }
    pub fn set_cross(&mut self, direction: Direction, cross: T) {
        match direction {
            Direction::Horizontal => self.height = cross,
            Direction::Vertical => self.width = cross,
        }
    }
    pub fn set_main(&mut self, direction: Direction, main: T) {
        match direction {
            Direction::Horizontal => self.width = main,
            Direction::Vertical => self.height = main,
        }
    }
}
impl<T: Copy> Size<T> {
    pub(crate) const fn on(self, direction: Direction) -> T {
        direction.orient(self)
    }
}

impl<T: Copy> Oriented<T> {
    pub const fn new(main: T, cross: T) -> Self {
        Self { main, cross }
    }

    pub(crate) fn on(self, direction: Direction) -> Size<T> {
        direction.absolute(self)
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
            Direction::Vertical.relative(Size::new("width", "height"));

        let Oriented { main: main_h, cross: cross_h } =
            Direction::Horizontal.relative(Size::new("width", "height"));
        assert_eq!(main_v, cross_h);
        assert_eq!(main_h, cross_v);
    }
    #[test]
    fn absolute() {
        let Size { width: width_v, height: height_v } =
            Direction::Vertical.absolute(Oriented::new("main", "cross"));
        let Size { width: width_h, height: height_h } =
            Direction::Horizontal.absolute(Oriented::new("main", "cross"));
        assert_eq!(width_v, height_h);
        assert_eq!(width_h, height_v);
    }
}