//! enums for main axis and cross axis alignment.

#[cfg(doc)]
use crate::Rule;
#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect};

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
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub enum Alignment {
    /// The items within the container are all aligned to the top or left.
    ///
    /// If the container's axis is `Direction::Vertical`, a start alignment
    /// will align all items to the left.
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
/// Note that [`Distribution::FillParent`] and [`Distribution::End`] requires
/// a parent with a known size (ie: the container's main axis constraint
/// must not be [`Rule::Children`]).
///
/// The following suposes an [`Alignment::Start`].
///
/// ```text
///          Direction::Vertical
///
///    Start    | FillParent  |     End     |
/// ▕██  ⁞    ▏ | ▕██  ⁞    ▏ | ▕    ⁞    ▏ |
/// ▕███████  ▏ | ▕    ⁞    ▏ | ▕    ⁞    ▏ |
/// ▕███ ⁞    ▏ | ▕███████  ▏ | ▕    ⁞    ▏ |
/// ▕█   ⁞    ▏ | ▕    ⁞    ▏ | ▕██  ⁞    ▏ |
/// ▕    ⁞    ▏ | ▕███ ⁞    ▏ | ▕███████  ▏ |
/// ▕    ⁞    ▏ | ▕    ⁞    ▏ | ▕███ ⁞    ▏ |
/// ▕    ⁞    ▏ | ▕█   ⁞    ▏ | ▕█   ⁞    ▏ |
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
#[doc(alias = "justification")]
pub enum Distribution {
    /// All item will be clumped together at the left/top.
    Start,
    /// Items are distributed evenly, with no space left on the sides of the container.
    ///
    /// > **Note**: This requires a known parent container main axis size.
    /// > Use [`Distribution::Start`] if you don't know it!
    /// >
    /// > If the parent container's constraint on the main axis is [`Rule::Children`],
    /// > `cuicui_layout` will log an error message.
    FillParent,
    /// All item will be clumped together at the right/bottom.
    ///
    /// > **Note**: This requires a known parent container main axis size.
    /// > Use [`Distribution::Start`] if you don't know it!
    /// >
    /// > If the parent container's constraint on the main axis is [`Rule::Children`],
    /// > `cuicui_layout` will log an error message.
    End,
}

/// Manage cross alignment.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Align {
    cross_parent_size: f32,
    align: Alignment,
}

impl Align {
    pub const fn new(cross_parent_size: f32, align: Alignment) -> Self {
        Align { cross_parent_size, align }
    }
    pub fn offset(self, cross_child_size: f32) -> f32 {
        match self.align {
            Alignment::Start => 0.0,
            Alignment::Center => (self.cross_parent_size - cross_child_size) / 2.0,
            Alignment::End => self.cross_parent_size - cross_child_size,
        }
    }
}
