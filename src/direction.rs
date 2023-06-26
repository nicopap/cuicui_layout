use std::fmt;

#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect};

// TODO(clean): Rename Rect to Axis, have a Axis{x,y} and a Oriented{main, cross}
/// The layout direction of a [`Container`].
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum Oriented {
    /// Children are arranged on the horizontal axis.
    Horizontal,

    /// Children are arranged on the vertical axis.
    Vertical,
}
impl Oriented {
    pub(crate) const fn orient<T: Copy>(self, width: T, height: T) -> T {
        self.relative(width, height).0
    }
    /// Returns (main, cross) according to this orientation.
    ///
    /// This is the inverse of [`Oriented::absolute`].
    pub(crate) const fn relative<T>(self, width: T, height: T) -> (T, T) {
        self.absolute(width, height)
    }
    /// Returns (width, height) according to this orientation.
    ///
    /// This is the inverse of [`Oriented::relative`].
    pub(crate) const fn absolute<T>(self, main: T, cross: T) -> (T, T) {
        match self {
            Oriented::Horizontal => (main, cross),
            Oriented::Vertical => (cross, main),
        }
    }
    /// Perpendicular orientation.
    pub(crate) const fn perp(self) -> Self {
        self.orient(Oriented::Vertical, Oriented::Horizontal)
    }
    pub(crate) const fn size_name(&self) -> &'static str {
        self.orient("width", "height")
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct Rect<T> {
    pub width: T,
    pub height: T,
}
impl<T> Rect<T> {
    pub(crate) const fn with(direction: Oriented, main: T, cross: T) -> Self {
        match direction {
            Oriented::Vertical => Self { height: main, width: cross },
            Oriented::Horizontal => Self { height: cross, width: main },
        }
    }
}
impl<T: Copy> Rect<T> {
    pub(crate) const fn on(self, direction: Oriented) -> T {
        direction.orient(self.width, self.height)
    }
    pub(crate) const fn cross(self, direction: Oriented) -> T {
        self.on(direction.perp())
    }
}
impl<T: fmt::Display> fmt::Display for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}×{}", self.width, self.height)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative() {
        let (main_v, cross_v) = Oriented::Vertical.relative("width", "height");
        let (main_h, cross_h) = Oriented::Horizontal.relative("width", "height");
        assert_eq!(main_v, cross_h);
        assert_eq!(main_h, cross_v);
    }
    #[test]
    fn absolute() {
        let (width_v, height_v) = Oriented::Vertical.absolute("main", "cross");
        let (width_h, height_h) = Oriented::Horizontal.absolute("main", "cross");
        assert_eq!(width_v, height_h);
        assert_eq!(width_h, height_v);
    }
}
