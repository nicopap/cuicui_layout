//! Bundles wrapping [`bevy::ui::node_bundles`] with additional [`cuicui_layout`]
//! components.
use bevy::{
    ecs::system::EntityCommands,
    prelude::{Bundle, Color, Handle, Image, Text, TextStyle, UiImage},
    ui::{node_bundles as bevy_ui, BackgroundColor},
    utils::default,
};
use cuicui_layout::dsl::{CommandLike, IntoUiBundle, LayoutCommands, LayoutCommandsExt, UiBundle};
use cuicui_layout::{LeafRule, Node, PosRect, Size};

use crate::content_sized::ContentSized;
use crate::BevyUiLayout;

macro_rules! impl_bundle {
    ($t:ident) => {
        impl From<bevy_ui::$t> for $t {
            fn from(inner: bevy_ui::$t) -> Self {
                #[allow(clippy::needless_update)]
                Self { inner, ..Self::default() }
            }
        }
    };
}

/// An image leaf node wrapping a [`bevy_ui::ImageBundle`].
///
/// By default, will stretch to fit the parent container.
///
/// If the `rule`s are set to [`LeafRule::Fixed`], then the inner image
/// will have a fixed size equal to that of the image.
/// If the image's size change, then the fixed size value updates to that
/// of the new image.
#[derive(Bundle, Default)]
pub struct ImageBundle {
    /// The [`cuicui_layout`] positional component.
    pub pos_rect: PosRect,
    /// The bevy bundle.
    pub inner: bevy_ui::ImageBundle,
    /// Mark this node for [`ContentSized`] size management.
    pub content_size: ContentSized,
    pub(crate) rules: Node,
}
impl ImageBundle {
    pub(crate) fn mut_box_size(&mut self) -> &mut Size<LeafRule> {
        let Node::Box(size) = &mut self.rules else {
            unreachable!("There is no way to make an `ImageBundle` with a non-box rule");
        };
        size
    }
    /// Set the [`LeafRule`] for the width of the image.
    ///
    /// a [`LeafRule::Parent`] will stretch the width to fit that of the
    /// parent.
    /// While [`LeafRule::Fixed`] — **regardless of the provided value** —
    /// will set the value to the image's native size.
    ///
    /// TODO: Account for ui scale.
    #[must_use]
    pub fn width_rule(mut self, rule: LeafRule) -> Self {
        self.mut_box_size().width = rule;
        self
    }
    /// Set the [`LeafRule`] for the height of the image.
    ///
    /// a [`LeafRule::Parent`] will stretch the height to fit that of the
    /// parent.
    /// While [`LeafRule::Fixed`] — **regardless of the provided value** —
    /// will set the value to the image's native size.
    ///
    /// TODO: Account for ui scale.
    #[must_use]
    pub fn height_rule(mut self, rule: LeafRule) -> Self {
        self.mut_box_size().height = rule;
        self
    }
}

/// A text leaf node wrapping a [`bevy_ui::TextBundle`].
///
/// By default, a text node will stretch to fit the parent's size.
///
/// In order to have the text be bound to a fixed size, you should use
/// [`LeafRule::Parent`] and wrap the text in another container with a [`Rule::Fixed`].
///
/// [`Rule::Fixed`]: cuicui_layout::Rule::Fixed
#[derive(Bundle, Default)]
pub struct TextBundle {
    /// The [`cuicui_layout`] positional component.
    pub pos_rect: PosRect,
    /// The bevy bundle.
    pub inner: bevy_ui::TextBundle,
    /// Mark this node for [`ContentSized`] size management.
    pub content_size: ContentSized,
    rules: Node,
}
impl TextBundle {
    pub(crate) fn mut_box_size(&mut self) -> &mut Size<LeafRule> {
        let Node::Box(size) = &mut self.rules else {
            unreachable!("There is no way to make an `TextBundle` with a non-box rule");
        };
        size
    }
    /// Set the width of this [`UiBundle`] to `rule`.
    ///
    /// If [`LeafRule::Fixed`], then the width of this layout node will fit
    /// exactly that of the text.
    #[must_use]
    pub fn width_rule(mut self, rule: LeafRule) -> Self {
        self.mut_box_size().width = rule;
        self
    }
    /// Set the height of this [`UiBundle`] to `rule`.
    ///
    /// If [`LeafRule::Fixed`], then the height of this layout node will fit
    /// exactly that of the text.
    #[must_use]
    pub fn height_rule(mut self, rule: LeafRule) -> Self {
        self.mut_box_size().height = rule;
        self
    }
}
impl From<Text> for TextBundle {
    fn from(text: Text) -> Self {
        bevy_ui::TextBundle { text, ..default() }.into()
    }
}
impl From<UiImage> for ImageBundle {
    fn from(image: UiImage) -> Self {
        bevy_ui::ImageBundle { image, ..default() }.into()
    }
}

impl_bundle!(ImageBundle);
impl_bundle!(TextBundle);

macro_rules! from_delegate_impl {
    ([$from:ty, $to:ty]) => {
        from_delegate_impl!([$from, $to], |self| <$to>::from(self).into_ui_bundle());
    };
    ([$from:ty, $to:ty], |$s:ident| $delegate_adaptor:expr) => {
        impl IntoUiBundle<BevyUiLayout> for $from {
            type Target = <$to as IntoUiBundle<BevyUiLayout>>::Target;

            fn into_ui_bundle($s) -> Self::Target {
                $delegate_adaptor
            }
        }
    };
}

from_delegate_impl!([&'_ str, String]);
from_delegate_impl! {
    [String, Text],
    |self| Text::from_section(self, TextStyle::default()).into_ui_bundle()
}
from_delegate_impl!([Handle<Image>, UiImage]);
from_delegate_impl!([Text, TextBundle]);
from_delegate_impl!([UiImage, ImageBundle]);
from_delegate_impl!([bevy_ui::ImageBundle, ImageBundle]);
from_delegate_impl!([bevy_ui::TextBundle, TextBundle]);

impl IntoUiBundle<BevyUiLayout> for ImageBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
impl IntoUiBundle<BevyUiLayout> for TextBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
impl UiBundle for ImageBundle {
    fn set_fixed_width(&mut self) {
        self.mut_box_size().width = LeafRule::Fixed(1.0);
    }
    fn set_fixed_height(&mut self) {
        self.mut_box_size().height = LeafRule::Fixed(1.0);
    }
}
impl UiBundle for TextBundle {
    fn set_fixed_width(&mut self) {
        self.mut_box_size().width = LeafRule::Fixed(1.0);
    }
    fn set_fixed_height(&mut self) {
        self.mut_box_size().height = LeafRule::Fixed(1.0);
    }
}

/// Wrapper for [`EntityCommands`] to enable UI handling.
pub struct BevyUiCommands<'w, 's, 'a> {
    bg_color: Option<BackgroundColor>,
    bg_image: Option<UiImage>,
    cmds: EntityCommands<'w, 's, 'a>,
}

impl<'w, 's, 'a> BevyUiCommands<'w, 's, 'a> {
    /// Create `Self` with default values.
    #[must_use]
    pub const fn new(cmds: EntityCommands<'w, 's, 'a>) -> Self {
        BevyUiCommands { bg_color: None, bg_image: None, cmds }
    }
}

impl<'w, 's, 'a> CommandLike for BevyUiCommands<'w, 's, 'a> {
    fn insert(&mut self, bundle: impl Bundle) {
        match (self.bg_color, self.bg_image.clone()) {
            (Some(background_color), Some(image)) => self.cmds.insert((
                bevy_ui::NodeBundle { background_color, ..default() },
                image,
                bundle,
            )),
            (Some(background_color), None) => self.cmds.insert((
                bevy_ui::NodeBundle { background_color, ..default() },
                bundle,
            )),
            (None, Some(image)) => {
                self.cmds
                    .insert((bevy_ui::NodeBundle::default(), image, bundle))
            }
            (None, None) => self.cmds.insert((bevy_ui::NodeBundle::default(), bundle)),
        };
    }
    fn entity(&self) -> bevy::prelude::Entity {
        self.cmds.id()
    }
    fn with_children(&mut self, f: impl FnOnce(&mut bevy::prelude::ChildBuilder)) {
        self.insert(());
        self.cmds.with_children(f);
    }
}
impl<'w, 's, 'a> From<EntityCommands<'w, 's, 'a>> for BevyUiCommands<'w, 's, 'a> {
    fn from(cmds: EntityCommands<'w, 's, 'a>) -> Self {
        Self { cmds, bg_color: None, bg_image: None }
    }
}

/// Add `bevy_ui` background data methods to [`LayoutCommands`].
pub trait BevyUiCommandsExt<'w, 's, 'a, C> {
    /// Set the node's background color.
    fn bg(self, color: Color) -> LayoutCommands<BevyUiCommands<'w, 's, 'a>>;
    /// Set the node's background image.
    fn bg_image(self, image: Handle<Image>) -> LayoutCommands<BevyUiCommands<'w, 's, 'a>>;
}
impl<'w, 's, 'a, C, T> BevyUiCommandsExt<'w, 's, 'a, C> for T
where
    T: LayoutCommandsExt<C>,
    C: CommandLike + Into<BevyUiCommands<'w, 's, 'a>>,
    'w: 'a,
    's: 'a,
{
    fn bg(self, color: Color) -> LayoutCommands<BevyUiCommands<'w, 's, 'a>> {
        self.into_lc()
            .with(|cmds| BevyUiCommands { bg_color: Some(color.into()), ..cmds.into() })
    }
    fn bg_image(self, image: Handle<Image>) -> LayoutCommands<BevyUiCommands<'w, 's, 'a>> {
        self.into_lc()
            .with(|cmds| BevyUiCommands { bg_image: Some(image.into()), ..cmds.into() })
    }
}

// impl<'w, 's, 'a> IntoCommandLike for &'a mut Commands<'w, 's> {
//     fn into_cmd(self) -> BevyUiCommands<'w, 's, 'a> {
//         self.spawn_empty().into()
//     }
// }
// impl<'w, 's, 'a> IntoCommandLike for &'a mut ChildBuilder<'w, 's, '_> {
//     fn into_cmd(self) -> BevyUiCommands<'w, 's, 'a> {
//         self.spawn_empty().into()
//     }
// }
