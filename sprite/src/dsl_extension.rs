//! Bundles wrapping [`bevy::sprite`] bundles with additional [`cuicui_layout`]
//! components.
use std::ops::{Deref, DerefMut};

#[cfg(feature = "sprite_text")]
use bevy::text::{Text, Text2dBundle, TextStyle};
use bevy::{
    ecs::system::EntityCommands,
    prelude::{Bundle, Color, Entity, Handle, Image, SpatialBundle},
    sprite,
    utils::default,
};
use cuicui_layout::dsl::{InsertKind, IntoUiBundle, MakeBundle, UiBundle};
use cuicui_layout::{LeafRule, Node, PosRect, Size};

use crate::content_sized::ContentSized;

/// An image leaf node wrapping a [`bevy::sprite::SpriteBundle`].
///
/// By default, will stretch to fit the parent container.
///
/// If the `rule`s are set to [`LeafRule::Fixed`], then the inner image
/// will have a fixed size equal to that of the image.
/// If the image's size change, then the fixed size value updates to that
/// of the new image.
#[derive(Bundle, Default)]
pub struct SpriteBundle {
    /// The [`cuicui_layout`] positional component.
    pub pos_rect: PosRect,
    /// The bevy bundle.
    pub inner: sprite::SpriteBundle,
    /// Mark this node for [`ContentSized`] size management.
    pub content_size: ContentSized,
    pub(crate) rules: Node,
}
impl SpriteBundle {
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
    #[must_use]
    pub fn height_rule(mut self, rule: LeafRule) -> Self {
        self.mut_box_size().height = rule;
        self
    }
}

/// A text leaf node wrapping a [`Text2dBundle`].
///
/// By default, a text node will stretch to fit the parent's size.
///
/// In order to have the text be bound to a fixed size, you should use
/// [`LeafRule::Parent`] and wrap the text in another container with a [`Rule::Fixed`].
///
/// [`Rule::Fixed`]: cuicui_layout::Rule::Fixed
#[cfg(feature = "sprite_text")]
#[derive(Bundle, Default)]
pub struct TextBundle {
    /// The [`cuicui_layout`] positional component.
    pub pos_rect: PosRect,
    /// The bevy bundle.
    pub inner: Text2dBundle,
    /// Mark this node for [`ContentSized`] size management.
    pub content_size: ContentSized,
    rules: Node,
}
#[cfg(feature = "sprite_text")]
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
#[cfg(feature = "sprite_text")]
impl From<Text> for TextBundle {
    fn from(text: Text) -> Self {
        Text2dBundle { text, ..default() }.into()
    }
}
#[cfg(feature = "sprite_text")]
impl From<Text2dBundle> for TextBundle {
    fn from(inner: Text2dBundle) -> Self {
        Self { inner, ..Self::default() }
    }
}
impl From<Handle<Image>> for SpriteBundle {
    fn from(texture: Handle<Image>) -> Self {
        sprite::SpriteBundle { texture, ..default() }.into()
    }
}
impl From<sprite::SpriteBundle> for SpriteBundle {
    fn from(inner: sprite::SpriteBundle) -> Self {
        Self { inner, ..Self::default() }
    }
}

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

#[cfg(feature = "sprite_text")]
from_delegate_impl!([&'_ str, String]);
#[cfg(feature = "sprite_text")]
from_delegate_impl! {
    [String, Text],
    |self| Text::from_section(self, TextStyle::default()).into_ui_bundle()
}
#[cfg(feature = "sprite_text")]
from_delegate_impl!([Text, TextBundle]);
#[cfg(feature = "sprite_text")]
from_delegate_impl!([Text2dBundle, TextBundle]);

from_delegate_impl!([Handle<Image>, SpriteBundle]);
from_delegate_impl!([sprite::SpriteBundle, SpriteBundle]);

impl IntoUiBundle<LayoutType> for SpriteBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
#[cfg(feature = "sprite_text")]
impl IntoUiBundle<LayoutType> for TextBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
impl UiBundle for SpriteBundle {
    fn set_fixed_width(&mut self) {
        self.mut_box_size().width = LeafRule::Fixed(1.0);
    }
    fn set_fixed_height(&mut self) {
        self.mut_box_size().height = LeafRule::Fixed(1.0);
    }
}
#[cfg(feature = "sprite_text")]
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
    bg_color: Option<Color>,
    bg_image: Option<Handle<Image>>,
}
impl<C> LayoutType<C> {
    /// Set the node's background color.
    pub fn bg(&mut self, color: Color) {
        self.bg_color = Some(color);
    }
    /// Set the node's background image.
    pub fn image(&mut self, image: &Handle<Image>) {
        self.bg_image = Some(image.clone());
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
            (Some(color), Some(texture)) => {
                let sprite = sprite::Sprite { color, ..default() };
                cmds.insert(sprite::SpriteBundle { sprite, texture, ..default() })
            }
            (Some(color), None) => {
                let sprite = sprite::Sprite { color, ..default() };
                cmds.insert(sprite::SpriteBundle { sprite, ..default() })
            }
            (None, Some(texture)) => cmds.insert((sprite::SpriteBundle { texture, ..default() },)),
            (None, None) => cmds.insert(SpatialBundle::default()),
        };
        id
    }
    fn ui_content_axis(&self) -> Size<bool> {
        self.inner.ui_content_axis()
    }
}