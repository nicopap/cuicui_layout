//! Bundles wrapping [`bevy::ui::node_bundles`] with additional [`cuicui_layout`]
//! components.
use bevy::{
    ecs::system::EntityCommands,
    prelude::{Bundle, Color, Deref, DerefMut, Entity, Handle, Image, Text, TextStyle, UiImage},
    ui::{node_bundles as bevy_ui, BackgroundColor},
    utils::default,
};
use cuicui_dsl::DslBundle;
use cuicui_layout::dsl::{ContentSized, IntoUiBundle, UiBundle};
#[cfg(doc)]
use cuicui_layout::{LeafRule, Rule};

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
    /// The bevy bundle.
    pub inner: bevy_ui::ImageBundle,
    /// Mark this node for [`ContentSized`] size management.
    pub content_size: ContentSized,
}

/// A text leaf node wrapping a [`bevy_ui::TextBundle`].
///
/// By default, a text node will stretch to fit the parent's size.
///
/// In order to have the text be bound to a fixed size, you should use
/// [`LeafRule::Parent`] and wrap the text in another container with a [`Rule::Fixed`].
#[derive(Bundle, Default)]
pub struct TextBundle {
    /// The bevy bundle.
    pub inner: bevy_ui::TextBundle,
    /// Mark this node for [`ContentSized`] size management.
    pub content_size: ContentSized,
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
        impl IntoUiBundle<Ui> for $from {
            type Target = <$to as IntoUiBundle<Ui>>::Target;

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

impl IntoUiBundle<Ui> for ImageBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
impl IntoUiBundle<Ui> for TextBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
impl UiBundle for ImageBundle {
    fn width_content_sized_enabled(&mut self) {
        self.content_size.managed_axis.width = true;
    }
    fn height_content_sized_enabled(&mut self) {
        self.content_size.managed_axis.height = true;
    }
    fn content_sized(&self) -> bool {
        self.content_size.managed_axis.width || self.content_size.managed_axis.height
    }
}
impl UiBundle for TextBundle {
    fn width_content_sized_enabled(&mut self) {
        self.content_size.managed_axis.width = true;
    }
    fn height_content_sized_enabled(&mut self) {
        self.content_size.managed_axis.height = true;
    }
    fn content_sized(&self) -> bool {
        self.content_size.managed_axis.width || self.content_size.managed_axis.height
    }
}

/// The [`DslBundle`] for `bevy_ui`.
#[derive(Default, Deref, DerefMut)]
pub struct Ui<C = cuicui_layout::dsl::LayoutDsl> {
    #[deref]
    inner: C,
    bg_color: Option<BackgroundColor>,
    bg_image: Option<UiImage>,
}
impl<C> Ui<C> {
    /// Set the node's background color.
    pub fn bg(&mut self, color: Color) {
        self.bg_color = Some(color.into());
    }
    /// Set the node's background image.
    pub fn image(&mut self, image: &Handle<Image>) {
        self.bg_image = Some(image.clone().into());
    }
}

impl<C: DslBundle> DslBundle for Ui<C> {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        let id = self.inner.insert(cmds);
        match (self.bg_color.take(), self.bg_image.take()) {
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
}
