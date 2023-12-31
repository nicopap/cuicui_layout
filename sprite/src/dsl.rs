//! Bundles wrapping [`bevy::sprite`] bundles with additional [`cuicui_layout`]
//! components.
use bevy::asset::Handle;
use bevy::ecs::{prelude::*, system::EntityCommands};
use bevy::prelude::{Deref, DerefMut};
use bevy::render::prelude::*;
use bevy::sprite;
#[cfg(feature = "sprite_text")]
use bevy::text::prelude::*;
use bevy::utils::default;
use cuicui_dsl::DslBundle;
use cuicui_layout::dsl::IntoUiBundle;
use thiserror::Error;

/// An image leaf node wrapping a [`bevy::sprite::SpriteBundle`].
///
/// If a `SpriteBundle`'s layout axis is not set, it will be dynamically computed
/// based on the image.
#[derive(Bundle, Default)]
pub struct SpriteBundle {
    /// The bevy bundle.
    pub inner: sprite::SpriteBundle,
}

/// A text leaf node wrapping a [`Text2dBundle`].
///
/// If a `TextBundle`'s layout axis is not set, it will be dynamically computed
/// based on the text's content.
#[cfg(feature = "sprite_text")]
#[derive(Bundle, Default)]
pub struct TextBundle {
    /// The bevy bundle.
    pub inner: Text2dBundle,
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
        Self { inner }
    }
}
impl From<Handle<Image>> for SpriteBundle {
    fn from(texture: Handle<Image>) -> Self {
        sprite::SpriteBundle { texture, ..default() }.into()
    }
}
impl From<sprite::SpriteBundle> for SpriteBundle {
    fn from(inner: sprite::SpriteBundle) -> Self {
        Self { inner }
    }
}

macro_rules! from_delegate_impl {
    ([$from:ty, $to:ty]) => {
        from_delegate_impl!([$from, $to], |self| <$to>::from(self).into_ui_bundle());
    };
    ([$from:ty, $to:ty], |$s:ident| $delegate_adaptor:expr) => {
        impl IntoUiBundle<SpriteDsl> for $from {
            type Target = <$to as IntoUiBundle<SpriteDsl>>::Target;

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

impl IntoUiBundle<SpriteDsl> for SpriteBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
#[cfg(feature = "sprite_text")]
impl IntoUiBundle<SpriteDsl> for TextBundle {
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

/// The [`DslBundle`] for `bevy_ui`.
#[derive(Default, Deref, DerefMut)]
pub struct SpriteDsl<D = cuicui_layout::dsl::LayoutDsl> {
    #[deref]
    inner: D,
    bg_color: Option<Color>,
    bg_image: Option<Handle<Image>>,
}
#[cfg_attr(
    feature = "chirp",
    cuicui_chirp::parse_dsl_impl(delegate = inner, type_parsers(Color = parse_color)),
)]
impl<D> SpriteDsl<D> {
    /// Set the node's background color.
    pub fn bg(&mut self, color: Color) {
        self.bg_color = Some(color);
    }
    /// Set the node's background image.
    pub fn image(&mut self, image: &Handle<Image>) {
        self.bg_image = Some(image.clone());
    }
}

impl<D: DslBundle> DslBundle for SpriteDsl<D> {
    fn insert(&mut self, cmds: &mut EntityCommands) {
        match (self.bg_color.take(), self.bg_image.take()) {
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
        self.inner.insert(cmds);
    }
}
