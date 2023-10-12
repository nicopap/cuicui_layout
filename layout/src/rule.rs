use std::str::FromStr;

use bevy::ecs::prelude::Entity;
#[cfg(feature = "reflect")]
use bevy::prelude::Reflect;
use thiserror::Error;

use crate::error::Computed;

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
            // TODO(err)
            Some(Rule::Children(_)) | None => Self::Content(0.),
            Some(Rule::Fixed(v)) => Self::Fixed(v),
            Some(Rule::Parent(v)) => Self::Parent(v),
        }
    }
    /// Compute effective size, given a potentially set parent container size.
    pub(crate) fn inside(self, parent_size: Computed) -> Result<f32, Entity> {
        use LeafRule::{Content, Fixed};
        match (self, parent_size) {
            (Self::Parent(ratio), Computed::Valid(value)) => Ok(value * ratio),
            (Self::Parent(_), Computed::ChildDefined(_, parent)) => Err(parent),
            (Fixed(fixed) | Content(fixed), _) => Ok(fixed),
        }
    }

    pub(crate) const fn parent_rule(self) -> Option<f32> {
        match self {
            Self::Parent(ratio) => Some(ratio),
            Self::Fixed(_) | Self::Content(_) => None,
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
    pub(crate) fn inside(self, parent_size: Computed, this: Entity) -> Result<Computed, Entity> {
        use Computed::{ChildDefined, Valid};
        match (self, parent_size) {
            (Self::Parent(ratio), Valid(value)) => Ok(Valid(value * ratio)),
            (Self::Parent(_), ChildDefined(_, parent)) => Err(parent),
            (Self::Fixed(fixed), _) => Ok(Valid(fixed)),
            (Self::Children(ratio), ChildDefined(_, parent)) => Ok(ChildDefined(ratio, parent)),
            (Self::Children(ratio), _) => Ok(ChildDefined(ratio, this)),
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
    /// The `f32` is populated by a system added by [`add_content_sized`].
    /// This will otherwise act pretty much like [`Self::Fixed`].
    ///
    /// [`add_content_sized`]: crate::content_sized::AppContentSizeExt::add_content_sized
    Content(f32),
}
impl Default for LeafRule {
    fn default() -> Self {
        Self::Parent(1.)
    }
}
