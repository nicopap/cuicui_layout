use std::{fmt, str::FromStr};

use bevy::ecs::prelude::{Component, Entity};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent};
use thiserror::Error;

use crate::Size;

/// A constraint on an axis of containers.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
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
    ///
    /// [`Flow::Vertical`]: crate::Flow::Vertical
    /// [`Flow::Horizontal`]: crate::Flow::Horizontal
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
#[derive(Debug, Error)]
pub enum RuleParseError {
    #[error("Invalid float format: {0} for '{1}'")]
    ParseFloat(std::num::ParseFloatError, Box<str>),
    #[error(
        "Provided a negative pixel amount ({0:.0}), this is not how you get 'negative space' \
        there is no such thing as a negative pixel, provide a positive value instead."
    )]
    NegativePixels(f32),
    #[error("The provided percent for '%' was out of range. {0:.0} ∉ [0..100] (inclusive)")]
    BadPercent(f32),
    #[error("The provided ratio for '*' was out of range. {0:.3} ∉ [0..1] (inclusive)")]
    BadRatio(f32),
    #[error(
        "Rule format was not recognized: '{0}', rules end with '%', '*' or 'px'. \
        Examples: '53%', '0.35*' and '1024px'"
    )]
    BadFormat(Box<str>),
}
impl Default for Rule {
    fn default() -> Self {
        Self::Children(1.)
    }
}
impl FromStr for Rule {
    type Err = RuleParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = |err| RuleParseError::ParseFloat(err, s.into());
        if let Some(pixels) = s.strip_suffix("px") {
            let pixels = pixels.parse().map_err(invalid)?;
            if pixels < 0. {
                return Err(RuleParseError::NegativePixels(pixels));
            }
            Ok(Self::Fixed(pixels))
        } else if let Some(percents) = s.strip_suffix('%') {
            let percents: f32 = percents.parse().map_err(invalid)?;
            if percents > 100. || percents < 0. {
                return Err(RuleParseError::BadPercent(percents));
            }
            Ok(Self::Parent(percents / 100.))
        } else if let Some(child_ratio) = s.strip_suffix('*') {
            let ratio: f32 = child_ratio.parse().map_err(invalid)?;
            if ratio < 1. {
                return Err(RuleParseError::BadRatio(ratio));
            }
            Ok(Self::Children(ratio))
        } else {
            Err(RuleParseError::BadFormat(s.into()))
        }
    }
}

impl LeafRule {
    #[cfg(feature = "dsl")]
    pub(crate) const fn from_rule(rule: Option<Rule>) -> Self {
        match rule {
            None => Self::Content_(1.0),
            Some(Rule::Children(v)) => Self::Content_(v),
            Some(Rule::Fixed(v)) => Self::Fixed(v),
            Some(Rule::Parent(v)) => Self::Parent(v),
        }
    }
    /// Compute effective size, given a potentially set parent container size.
    pub(crate) fn inside(self, parent_size: Computed, content: Option<f32>) -> Option<f32> {
        use LeafRule::{Content_, Fixed};
        match (self, parent_size) {
            (Self::Parent(ratio), Computed::Valid(value)) => Some(value * ratio),
            (Self::Parent(_), Computed::ChildDefined(_, _)) => None,
            (Fixed(fixed), _) => Some(fixed),
            (Content_(ratio), _) => content.map(|px_size| px_size * ratio),
        }
    }

    pub(crate) const fn parent_rule(self) -> Option<f32> {
        match self {
            Self::Parent(ratio) => Some(ratio),
            Self::Fixed(_) | Self::Content_(_) => None,
        }
    }
}
impl Rule {
    pub(crate) const fn parent_rule(self) -> Option<f32> {
        match self {
            Self::Parent(ratio) => Some(ratio),
            Self::Children(_) | Self::Fixed(_) => None,
        }
    }
    /// Compute effective size, given a potentially set parent container size.
    pub(crate) fn inside(self, parent_size: Computed, this: Entity) -> Option<Computed> {
        use Computed::{ChildDefined, Valid};
        match (self, parent_size) {
            (Self::Parent(ratio), Valid(value)) => Some(Valid(value * ratio)),
            (Self::Parent(_), ChildDefined(_, parent)) => None,
            (Self::Fixed(fixed), _) => Some(Valid(fixed)),
            (Self::Children(ratio), ChildDefined(_, parent)) => Some(ChildDefined(ratio, parent)),
            (Self::Children(ratio), _) => Some(ChildDefined(ratio, this)),
        }
    }
}

/// A constraint on an axis of a terminal `Node` (ie: doesn't have a `Children` constraint).
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum LeafRule {
    /// The box's size is equal to its parent's size  times `f32`.
    /// (may not be above 1)
    Parent(f32),

    /// The box's size is equal to precisely `f32` pixels.
    Fixed(f32),

    /// The box's size on given axis is dependent on its content.
    ///
    /// The `f32` is the relative sized compared to the content, it works similarly
    /// to [`Rule::Children`].
    ///
    /// The size is read from the [`ContentSized`] component.
    Content_(f32),
}
impl Default for LeafRule {
    fn default() -> Self {
        Self::Parent(1.)
    }
}

/// The set size of content of a node with a content-sized rule.
///
/// Content-sized rules are rules that depends on content of a node, be it
/// text, images, or any user-defined things.
///
/// This component is added by this crate, other crates are meant to update it
/// with a meaningfull **fixed** value by adding the modifying system to
/// [`crate::ContentSizedSet`].
///
/// For example, for images, the value should always be the width/height in
/// pixels of the image, regardless of the actual size of the node.
#[derive(Clone, Copy, PartialEq, Debug, Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
pub struct ContentSized(pub Size<f32>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Computed {
    ChildDefined(f32, Entity),
    Valid(f32),
}
impl Computed {
    pub(crate) fn with_child(&self, child_size: f32) -> f32 {
        match self {
            // TODO: margin
            Self::ChildDefined(ratio, _) => *ratio * child_size,
            Self::Valid(size) => *size,
        }
    }
}
impl From<f32> for Computed {
    fn from(value: f32) -> Self {
        Self::Valid(value)
    }
}
impl fmt::Display for Computed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChildDefined(_, _) => fmt::Display::fmt("<child_size>", f),
            Self::Valid(value) => fmt::Display::fmt(value, f),
        }
    }
}

impl From<Size<f32>> for Size<Computed> {
    fn from(Size { width, height }: Size<f32>) -> Self {
        Self { width: width.into(), height: height.into() }
    }
}
