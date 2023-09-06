//! enums for main axis and cross axis alignment.

use std::{mem::replace, str::FromStr};

#[cfg(feature = "reflect")]
use bevy::prelude::Reflect;

use crate::Oriented;

/// The cross axis alignment. Aka alignment.
///
/// Note that you can use any alignments with any kind of constraint.
///
/// ```text
///          Direction::Vertical
///
///    Start    |   Center    |     End     |
/// ▕██  ⁞    ▏ | ▕    ██   ▏ | ▕    ⁞  ██▏ |
/// ▕███████  ▏ | ▕ ███████ ▏ | ▕  ███████▏ |
/// ▕    ⁞    ▏ | ▕    ⁞    ▏ | ▕    ⁞    ▏ |
/// ▕███ ⁞    ▏ | ▕   ███   ▏ | ▕    ⁞ ███▏ |
/// ▕█   ⁞    ▏ | ▕    █    ▏ | ▕    ⁞   █▏ |
/// ▕███ ⁞    ▏ | ▕   ███   ▏ | ▕    ⁞ ███▏ |
/// ▕    ⁞    ▏ | ▕    ⁞    ▏ | ▕    ⁞    ▏ |
/// ▕█   ⁞    ▏ | ▕    █    ▏ | ▕    ⁞   █▏ |
/// ```
///
/// ```text
///          Direction::Horizontal
///
/// |   Start   |  Center   |    End    |
/// |▁▁▁▁▁▁▁▁▁▁▁|▁▁▁▁▁▁▁▁▁▁▁|▁▁▁▁▁▁▁▁▁▁▁|
/// |██ ███ █ ██|           |           |
/// |██ █ █   ██| █         |           |
/// | █ ███   ██|██ ███   ██| █         |
/// |⋯█⋯⋯⋯⋯⋯⋯⋯⋯⋯|██⋯█⋯█⋯█⋯██|⋯█⋯⋯⋯⋯⋯⋯⋯⋯⋯|
/// | █         | █ ███   ██| █ ███   ██|
/// |           | █         |██ █ █   ██|
/// |           |           |██ ███ █ ██|
/// |▔▔▔▔▔▔▔▔▔▔▔|▔▔▔▔▔▔▔▔▔▔▔|▔▔▔▔▔▔▔▔▔▔▔|
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum Alignment {
    /// The items within the container are all aligned to the top or left.
    ///
    /// If the container's axis is `Direction::Vertical`, a start alignment
    /// will align all items to the left.
    #[default]
    Start,

    /// The items within the container are all centered on the container's axis.
    Center,

    /// The items within the container are all aligned to the bottom or right.
    ///
    /// If the container's axis is `Direction::Vertical`, an end alignment
    /// will align all items to the right.
    End,
}

/// The main axis alignment. Aka distribution.
///
/// The following suposes an [`Alignment::Start`].
///
/// ```text
///          Direction::Vertical
///
///    Start    |  FillMain   |     End     |
/// ▕██  ⁞    ▏ | ▕██  ⁞    ▏ | ▕    ⁞    ▏ |
/// ▕███████  ▏ | ▕    ⁞    ▏ | ▕    ⁞    ▏ |
/// ▕███ ⁞    ▏ | ▕███████  ▏ | ▕    ⁞    ▏ |
/// ▕█   ⁞    ▏ | ▕    ⁞    ▏ | ▕██  ⁞    ▏ |
/// ▕    ⁞    ▏ | ▕███ ⁞    ▏ | ▕███████  ▏ |
/// ▕    ⁞    ▏ | ▕    ⁞    ▏ | ▕███ ⁞    ▏ |
/// ▕    ⁞    ▏ | ▕█   ⁞    ▏ | ▕█   ⁞    ▏ |
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[doc(alias = "justification")]
pub enum Distribution {
    /// Items are clumped together at the left/top.
    #[default]
    Start,

    /// Items are distributed evenly, with no space left on the sides of the container.
    FillMain,

    /// Items are clumped together at the right/bottom.
    End,

    /// Items overlap at the left/top.
    OverlapStart,

    /// Items overlap centered on the middle of the container.
    OverlapCenter,

    /// Items overlap at the right/bottom.
    OverlapEnd,
}

/// Manage cross alignment.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct CrossAlign {
    cross_parent_size: f32,
    align: Alignment,
}

impl Alignment {
    pub(crate) const fn compute(self, parent_size: Oriented<f32>) -> CrossAlign {
        CrossAlign { cross_parent_size: parent_size.cross, align: self }
    }
}
impl CrossAlign {
    pub fn offset(self, cross_child_size: f32) -> f32 {
        match self.align {
            Alignment::Start => 0.0,
            Alignment::Center => (self.cross_parent_size - cross_child_size) / 2.0,
            Alignment::End => self.cross_parent_size - cross_child_size,
        }
    }
}

/// Manage main alignment based on [`Distribution`].
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct MainAlign {
    offset: f32,
    gap: f32,
    distrib: Distribution,
}

impl Distribution {
    /// Whether children are meant to overlap.
    #[must_use]
    pub const fn overlaps(self) -> bool {
        use Distribution::{OverlapCenter, OverlapEnd, OverlapStart};
        matches!(self, OverlapStart | OverlapCenter | OverlapEnd)
    }
    pub(crate) fn compute(
        self,
        main_size: f32,
        child_main_size: f32,
        single_child: bool,
        count: f32,
    ) -> MainAlign {
        let (offset, gap) = match self {
            Self::FillMain if single_child => ((main_size - child_main_size) / 2., 0.),
            Self::OverlapCenter => (0., main_size / 2.),
            Self::FillMain => (0., (main_size - child_main_size) / count),
            Self::Start | Self::OverlapStart => (0., 0.),
            Self::End => (main_size - child_main_size, 0.),
            Self::OverlapEnd => (0., main_size),
        };
        MainAlign { offset, gap, distrib: self }
    }
}
impl MainAlign {
    pub fn offset(&mut self, child_size: f32) -> f32 {
        use Distribution::{End, FillMain, Start};
        match self.distrib {
            Start | FillMain | End => {
                let new_offset = self.offset + child_size + self.gap;
                replace(&mut self.offset, new_offset)
            }
            Distribution::OverlapStart => 0.,
            Distribution::OverlapCenter => self.gap - child_size / 2.,
            Distribution::OverlapEnd => self.gap - child_size,
        }
    }
}

impl FromStr for Distribution {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dS" => Ok(Self::Start),
            "dE" => Ok(Self::End),
            "dC" => Ok(Self::FillMain),
            "oS" => Ok(Self::OverlapStart),
            "oE" => Ok(Self::OverlapEnd),
            "oC" => Ok(Self::OverlapCenter),
            _ => Err(()),
        }
    }
}

impl FromStr for Alignment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aS" => Ok(Self::Start),
            "aE" => Ok(Self::End),
            "aC" => Ok(Self::Center),
            _ => Err(()),
        }
    }
}
