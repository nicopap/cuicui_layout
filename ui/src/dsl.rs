//! Bundles wrapping [`bevy::ui::node_bundles`] with additional [`cuicui_layout`]
//! components.

use bevy::asset::{Handle, LoadContext};
use bevy::ecs::{prelude::*, system::EntityCommands};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{Deref, DerefMut};
use bevy::reflect::TypeRegistryInternal as Registry;
use bevy::render::{color::Color, texture::Image};
use bevy::text::{Text, TextLayoutInfo, TextStyle};
use bevy::ui::node_bundles as bevy_ui;
use bevy::ui::widget::{TextFlags, UiImageSize};
use bevy::ui::{prelude::*, ContentSize};
use bevy::utils::default;
use css_color::Srgb;
use cuicui_dsl::DslBundle;
use cuicui_layout::dsl::IntoUiBundle;
#[cfg(doc)]
use cuicui_layout::{LeafRule, Rule};
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
    pub text_layout_info: TextLayoutInfo,
    pub text_flags: TextFlags,
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
            text_layout_info: value.text_layout_info,
            text_flags: value.text_flags,
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

fn parse_color(
    _: &Registry,
    _: Option<&mut LoadContext>,
    input: &str,
) -> Result<Color, ParseColorError> {
    let err = |_| ParseColorError(input.to_string());
    let Srgb { red, green, blue, alpha } = input.parse::<Srgb>().map_err(err)?;
    Ok(Color::rgba(red, green, blue, alpha))
}

/// The [`DslBundle`] for `bevy_ui`.
#[derive(Default, Deref, DerefMut)]
pub struct UiDsl<D = cuicui_layout::dsl::LayoutDsl> {
    #[deref]
    inner: D,
    bg_color: Option<BackgroundColor>,
    bg_image: Option<UiImage>,
    border_color: Option<BorderColor>,
    border_px: Option<u16>,
}
#[cuicui_chirp::parse_dsl_impl(delegate = inner, type_parsers(Color = parse_color))]
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
        self.border_px = Some(pixels);
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
        self.bg_image = Some(image.clone().into());
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
                    border: UiRect::all(Val::Px(f32::from(pixels))),
                    ..default()
                },
                ..default()
            };
            cmds.with_children(|c| {
                c.spawn(child_bundle);
            });
        }
        match self.bg_image.take() {
            Some(image) => cmds.insert(ImageBundle::from(image)).insert(node_bundle),
            None => cmds.insert(node_bundle),
        };
        self.inner.insert(cmds)
    }
}
