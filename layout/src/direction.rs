//! Structs to help convert between a relative and absolute direction.
use std::{fmt, ops, str::FromStr};

#[cfg(feature = "reflect")]
use bevy::prelude::Reflect;

/// A synonymous for [`Flow`].
pub type Axis = Flow;

/// A `T` that applies to the `width` and `height` of something.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct Size<T> {
    /// `T` on the horizontal axis.
    pub width: T,
    /// `T` on the vertical axis.
    pub height: T,
}

/// Similar to [`Size`], but relative to a [`Flow`] direction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
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
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum Flow {
    /// Children are arranged on the horizontal axis. May also be "width".
    #[default]
    Horizontal,

    /// Children are arranged on the vertical axis. May also be "height".
    Vertical,
}
impl Flow {
    /// Returns [`Size`] oriented according to this orientation.
    ///
    /// This is the inverse of [`Flow::absolute`].
    pub const fn relative<T: Copy>(self, Size { width, height }: Size<T>) -> Oriented<T> {
        let Size { width: main, height: cross } = self.absolute(Oriented::new(width, height));
        Oriented { main, cross }
    }
    /// Returns [`Oriented`] in oriented according to the global point of view.
    ///
    /// This is the inverse of [`Flow::relative`].
    pub const fn absolute<T: Copy>(self, Oriented { main, cross }: Oriented<T>) -> Size<T> {
        match self {
            Self::Horizontal => Size::new(main, cross),
            Self::Vertical => Size::new(cross, main),
        }
    }
}
impl Size<f32> {
    /// A `Size<f32>` with 0 width and 0 height.
    pub const ZERO: Self = Self { width: 0., height: 0. };
}
impl ops::Add for Size<f32> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
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
        Self { width: value.clone(), height: value }
    }
    /// Apply `f` on `width` and `height`, returning a `Size` with the output
    /// values of `f`.
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Size<U> {
        Size { width: f(self.width), height: f(self.height) }
    }
    /// Convert the content of this `Size`
    pub fn map_into<U: From<T>>(self) -> Size<U> {
        self.map(Into::into)
    }
    /// Go from `&Size<T>` to `Size<&T>`.
    pub const fn as_ref(&self) -> Size<&T> {
        let Self { width, height } = self;
        Size { width, height }
    }
}

impl<T: Copy> Oriented<T> {
    /// Create an [`Oriented`] for given `main` and `cross` `T`.
    pub const fn new(main: T, cross: T) -> Self {
        Self { main, cross }
    }
    /// Go from `&Oriented<T>` to `Oriented<&T>`.
    pub const fn as_ref(&self) -> Oriented<&T> {
        let Self { main, cross } = self;
        Oriented { main, cross }
    }
}

impl fmt::Display for Flow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Horizontal => f.write_str("width"),
            Self::Vertical => f.write_str("height"),
        }
    }
}
impl FromStr for Flow {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v" => Ok(Self::Vertical),
            ">" => Ok(Self::Horizontal),
            _ => Err(()),
        }
    }
}

impl From<bevy::math::Vec2> for Size<f32> {
    fn from(value: bevy::math::Vec2) -> Self {
        Self::new(value.x, value.y)
    }
}
impl From<Size<f32>> for bevy::math::Vec2 {
    fn from(value: Size<f32>) -> Self {
        Self::new(value.width, value.height)
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
