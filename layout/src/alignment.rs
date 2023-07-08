//! enums for main axis and cross axis alignment.

#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect};

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
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
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
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
#[doc(alias = "justification")]
pub enum Distribution {
    /// All item will be clumped together at the left/top.
    #[default]
    Start,

    /// Items are distributed evenly, with no space left on the sides of the container.
    FillMain,

    /// All item will be clumped together at the right/bottom.
    End,
}

/// Manage cross alignment.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct CrossAlign {
    cross_parent_size: f32,
    align: Alignment,
}

impl CrossAlign {
    pub const fn new(parent_size: Oriented<f32>, align: Alignment) -> Self {
        CrossAlign { cross_parent_size: parent_size.cross, align }
    }
    pub fn offset(self, cross_child_size: f32) -> f32 {
        match self.align {
            Alignment::Start => 0.0,
            Alignment::Center => (self.cross_parent_size - cross_child_size) / 2.0,
            Alignment::End => self.cross_parent_size - cross_child_size,
        }
    }
}
