//! Bundles wrapping [`bevy::ui::node_bundles`] with additional [`cuicui_layout`]
//! components.

use std::num::NonZeroU16;

use bevy::asset::Handle;
use bevy::ecs::{prelude::*, system::EntityCommands};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{Deref, DerefMut};
use bevy::render::{color::Color, texture::Image};
use bevy::text::{BreakLineOn, Font, Text, TextAlignment, TextSection, TextStyle};
use bevy::ui::node_bundles as bevy_ui;
use bevy::ui::widget::UiImageSize;
use bevy::ui::{prelude::*, ContentSize};
use bevy::utils::default;
use cuicui_dsl::DslBundle;
use cuicui_layout::dsl::IntoUiBundle;
#[cfg(doc)]
use cuicui_layout::{LeafRule, Rule};
use enumset::{EnumSet, EnumSetType};
use thiserror::Error;

/// An image leaf node wrapping a [`bevy_ui::ImageBundle`].
///
/// By default, will stretch to fit the parent container.
///
/// If the `rule`s are set to [`LeafRule::Fixed`], then the inner image
/// will have a fixed size equal to that of the image.
/// If the image's size change, then the fixed size value updates to that
/// of the new image.
#[derive(Bundle, Default)]
#[allow(missing_docs)]
pub struct ImageBundle {
    pub calculated_size: ContentSize,
    pub image: UiImage,
    pub image_size: UiImageSize,
    pub bg: BackgroundColor,
}

/// A text leaf node wrapping a [`bevy_ui::TextBundle`].
///
/// By default, a text node will stretch to fit the parent's size.
///
/// In order to have the text be bound to a fixed size, you should use
/// [`LeafRule::Parent`] and wrap the text in another container with a [`Rule::Fixed`].
#[derive(Bundle, Default)]
#[allow(missing_docs)]
pub struct TextBundle {
    pub text: Text,
    pub calculated_size: ContentSize,
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

impl From<bevy_ui::ImageBundle> for ImageBundle {
    fn from(value: bevy_ui::ImageBundle) -> Self {
        Self {
            calculated_size: value.calculated_size,
            image_size: value.image_size,
            image: value.image,
            bg: Color::WHITE.into(),
        }
    }
}
impl From<bevy_ui::TextBundle> for TextBundle {
    fn from(value: bevy_ui::TextBundle) -> Self {
        Self {
            calculated_size: value.calculated_size,
            text: value.text,
        }
    }
}

macro_rules! from_delegate_impl {
    ([$from:ty, $to:ty]) => {
        from_delegate_impl!([$from, $to], |self| <$to>::from(self).into_ui_bundle());
    };
    ([$from:ty, $to:ty], |$s:ident| $delegate_adaptor:expr) => {
        impl IntoUiBundle<UiDsl> for $from {
            type Target = <$to as IntoUiBundle<UiDsl>>::Target;

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
// TODO(perf): bevy_ui::ImageBundle's `content_size` field force-inlines
// ContentSize::drop, as a result, we have several copies of it here.
from_delegate_impl!([Handle<Image>, UiImage]);
from_delegate_impl!([Text, TextBundle]);
from_delegate_impl!([UiImage, ImageBundle]);
from_delegate_impl!([bevy_ui::ImageBundle, ImageBundle]);
from_delegate_impl!([bevy_ui::TextBundle, TextBundle]);

impl IntoUiBundle<UiDsl> for ImageBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
impl IntoUiBundle<UiDsl> for TextBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}

/// Error occuring when failing to parse a bevy [`Color`] according to the
/// [`css_color`] crate implementation.
#[derive(Debug, Error)]
#[error(
    "'{0}' is not a valid color, try using the syntax found in the `css-color` crate\n\n\
    https://lib.rs/crates/css-color"
)]
pub struct ParseColorError(String);

#[cfg(feature = "chirp")]
fn parse_color(
    _: &bevy::reflect::TypeRegistry,
    _: Option<&mut bevy::asset::LoadContext>,
    input: &str,
) -> Result<Color, ParseColorError> {
    use css_color::Srgb;
    let err = |_| ParseColorError(input.to_string());
    let Srgb { red, green, blue, alpha } = input.parse::<Srgb>().map_err(err)?;
    Ok(Color::rgba(red, green, blue, alpha))
}

#[derive(Debug, EnumSetType)]
enum UiDslFlags {
    AlignLeft,
    AlignRight,
    BreakOnWord,
    BreakOnChar,
    BgFlipX,
    BgFlipY,
}

/// The [`DslBundle`] for `bevy_ui`.
#[derive(Deref, DerefMut, Debug)]
pub struct UiDsl<D = cuicui_layout::dsl::LayoutDsl> {
    #[deref]
    inner: D,
    bg_color: Option<BackgroundColor>,
    bg_image: Option<Handle<Image>>,
    border_color: Option<BorderColor>,
    border_px: Option<NonZeroU16>,
    text: Option<Box<str>>,
    text_color: Color,
    font_size: u16,
    font: Option<Handle<Font>>,
    flags: EnumSet<UiDslFlags>,
}
impl<D: Default> Default for UiDsl<D> {
    fn default() -> Self {
        Self {
            inner: D::default(),
            bg_color: None,
            bg_image: None,
            border_color: None,
            border_px: None,
            text: None,
            flags: UiDslFlags::BreakOnWord | UiDslFlags::AlignLeft,
            text_color: Color::WHITE,
            font_size: 12,
            font: None,
        }
    }
}
#[cfg_attr(
    feature = "chirp",
    cuicui_chirp::parse_dsl_impl(delegate = inner, type_parsers(Color = parse_color)),
)]
impl<D> UiDsl<D> {
    /// Set the node's border width, in pixels. Note that this is only visual and has
    /// no effect on the `cuicui_layout` algorithm.
    ///
    /// Due to a limitation of CSS, border will be spawned as a child of the
    /// actual node entity.
    ///
    /// This is because it would be otherwise impossible to arrange children
    /// independently of parent properties.
    pub fn border_px(&mut self, pixels: u16) {
        self.border_px = NonZeroU16::new(pixels);
    }
    /// Set the node's border [color](Self::border_color) and [width](Self::border_px).
    pub fn border(&mut self, pixels: u16, color: Color) {
        self.border_px(pixels);
        self.border_color(color);
    }
    /// Set the node's border color.
    ///
    /// Due to a limitation of CSS, border will be spawned as a child of the
    /// actual node entity.
    ///
    /// This is because it would be otherwise impossible to arrange children
    /// independently of parent properties.
    pub fn border_color(&mut self, color: Color) {
        self.border_color = Some(color.into());
    }
    /// Set the node's background color.
    pub fn bg(&mut self, color: Color) {
        self.bg_color = Some(color.into());
    }
    /// Set the node's background image.
    pub fn image(&mut self, image: &Handle<Image>) {
        self.bg_image = Some(image.clone());
    }
    /// If this node has a background image, flip it on its X axis.
    pub fn flip_x(&mut self) {
        self.flags |= UiDslFlags::BgFlipX;
    }
    /// If this node has a background image, flip it on its Y axis.
    pub fn flip_y(&mut self) {
        self.flags |= UiDslFlags::BgFlipY;
    }
    /// Set the node's text.
    pub fn text(&mut self, text: &str) {
        self.text = Some(text.into());
    }
    /// If this node contains text, set its break behavior to breaking on
    /// individual characters.
    ///
    /// By default, text breaks on word.
    pub fn break_on_char(&mut self) {
        self.flags |= UiDslFlags::BreakOnChar;
    }
    /// If this node contains text, only go to next line on '\n' in text.
    ///
    /// By default, text breaks on word.
    pub fn no_wrap(&mut self) {
        use UiDslFlags::{BreakOnChar, BreakOnWord};
        self.flags.remove_all(BreakOnChar | BreakOnWord);
    }
    /// If this node contains text, align it to the left.
    ///
    /// By default, text is aligned left
    pub fn text_right_align(&mut self) {
        self.flags |= UiDslFlags::AlignRight;
    }
    /// If this node contains text, align it to the center.
    ///
    /// By default, text is aligned left.
    pub fn text_center_align(&mut self) {
        use UiDslFlags::{AlignLeft, AlignRight};
        self.flags.remove_all(AlignLeft | AlignRight);
    }
    /// Set the text size for this node.
    pub fn font_size(&mut self, size: u16) {
        self.font_size = size;
    }
    /// Set the text font.
    pub fn font(&mut self, font: &Handle<Font>) {
        self.font = Some(font.clone());
    }
}
impl<D> UiDsl<D> {
    fn text_alignment(&self) -> TextAlignment {
        use UiDslFlags::{AlignLeft, AlignRight};
        match () {
            () if self.flags.contains(AlignRight) => TextAlignment::Right,
            () if self.flags.contains(AlignLeft) => TextAlignment::Left,
            () => TextAlignment::Center,
        }
    }
    fn break_line_on(&self) -> BreakLineOn {
        use UiDslFlags::{BreakOnChar, BreakOnWord};
        match () {
            () if self.flags.contains(BreakOnChar) => BreakLineOn::AnyCharacter,
            () if self.flags.contains(BreakOnWord) => BreakLineOn::WordBoundary,
            () => BreakLineOn::NoWrap,
        }
    }
}

impl<D: DslBundle> DslBundle for UiDsl<D> {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        let mut node_bundle = bevy_ui::NodeBundle::default();
        if self.bg_image.is_some() {
            node_bundle.background_color = Color::WHITE.into();
        }
        if let Some(background_color) = self.bg_color.take() {
            node_bundle.background_color = background_color;
        }
        if let (Some(pixels), Some(border_color)) = (self.border_px, self.border_color.take()) {
            let child_bundle = NodeBundle {
                border_color,
                style: bevy::ui::Style {
                    position_type: bevy::ui::PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    border: UiRect::all(Val::Px(f32::from(pixels.get()))),
                    ..default()
                },
                ..default()
            };
            cmds.with_children(|c| {
                c.spawn(child_bundle);
            });
        }
        if let Some(text) = self.text.take() {
            let mut text_style = TextStyle {
                font_size: f32::from(self.font_size),
                color: self.text_color,
                ..default()
            };
            if let Some(font) = self.font.take() {
                text_style.font = font;
            }
            let text = Text {
                sections: vec![TextSection::new(text, text_style)],
                alignment: self.text_alignment(),
                linebreak_behavior: self.break_line_on(),
            };
            cmds.insert(TextBundle { text, ..default() });
        }
        match self.bg_image.take() {
            Some(image) => {
                let ui_image = UiImage {
                    texture: image,
                    flip_x: self.flags.contains(UiDslFlags::BgFlipX),
                    flip_y: self.flags.contains(UiDslFlags::BgFlipY),
                };
                cmds.insert(ImageBundle::from(ui_image)).insert(node_bundle)
            }
            None => cmds.insert(node_bundle),
        };
        self.inner.insert(cmds)
    }
}
