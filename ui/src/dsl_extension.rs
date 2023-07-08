//! Bundles wrapping [`bevy::ui::node_bundles`] with additional [`cuicui_layout`]
//! components.
use std::ops::{Deref, DerefMut};

use bevy::{
    ecs::system::EntityCommands,
    prelude::{Bundle, Color, Entity, Handle, Image, Text, TextStyle, UiImage},
    ui::{node_bundles as bevy_ui, BackgroundColor},
    utils::default,
};
use cuicui_layout::dsl::{InsertKind, IntoUiBundle, MakeBundle, UiBundle};
use cuicui_layout::{LeafRule, Node, PosRect, Size};

use crate::content_sized::ContentSized;

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
        impl IntoUiBundle<LayoutType> for $from {
            type Target = <$to as IntoUiBundle<LayoutType>>::Target;

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

impl IntoUiBundle<LayoutType> for ImageBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
impl IntoUiBundle<LayoutType> for TextBundle {
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

/// The [`MakeBundle`] for `bevy_ui`.
#[derive(Default)]
pub struct LayoutType<C = cuicui_layout::dsl::LayoutType> {
    inner: C,
    bg_color: Option<BackgroundColor>,
    bg_image: Option<UiImage>,
}
impl<C> LayoutType<C> {
    /// Set the node's background color.
    pub fn bg(&mut self, color: Color) {
        self.bg_color = Some(color.into());
    }
    /// Set the node's background image.
    pub fn image(&mut self, image: &Handle<Image>) {
        self.bg_image = Some(image.clone().into());
    }
}

impl<C> Deref for LayoutType<C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<C> DerefMut for LayoutType<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<C: MakeBundle> MakeBundle for LayoutType<C> {
    fn insert(self, kind: InsertKind, cmds: &mut EntityCommands) -> Entity {
        let id = self.inner.insert(kind, cmds);
        match (self.bg_color, self.bg_image) {
            (Some(background_color), Some(image)) => {
                cmds.insert((bevy_ui::NodeBundle { background_color, ..default() }, image))
            }
            (Some(background_color), None) => {
                cmds.insert(bevy_ui::NodeBundle { background_color, ..default() })
            }
            (None, Some(image)) => cmds.insert((
                bevy_ui::NodeBundle { background_color: Color::WHITE.into(), ..default() },
                image,
            )),
            (None, None) => cmds.insert(bevy_ui::NodeBundle::default()),
        };
        id
    }
    fn ui_content_axis(&self) -> Size<bool> {
        self.inner.ui_content_axis()
    }
}
