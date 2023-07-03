use bevy::prelude::{Bundle, Handle, Image, Text, TextStyle, UiImage};
use bevy::ui::node_bundles as bevy_ui;
use cuicui_layout::LeafRule;

use crate::bundles::{ImageBundle, TextBundle};

macro_rules! from_delegate_impl {
    ([$from:ty, $to:ty]) => {
        from_delegate_impl!([$from, $to], |self| <$to>::from(self).into_ui_bundle());
    };
    ([$from:ty, $to:ty], |$s:ident| $delegate_adaptor:expr) => {
        impl IntoUiBundle for $from {
            type Target = <$to as IntoUiBundle>::Target;

            fn into_ui_bundle($s) -> Self::Target {
                $delegate_adaptor
            }
        }
    };
}

/// A bevy [`Bundle`] that can be spawned as a `cuicui_layout` terminal node.
///
/// [`UiBundle`] differs from [`Bundle`] in that it is possible to set its
/// layouting properties.
///
/// This includes images and text, not much else.
pub trait UiBundle: Bundle {
    fn set_fixed_width(&mut self);
    fn set_fixed_height(&mut self);
}

/// Something that can be converted into [`UiBundle`].
pub trait IntoUiBundle {
    type Target: UiBundle;
    fn into_ui_bundle(self) -> Self::Target;
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

impl IntoUiBundle for TextBundle {
    type Target = Self;
    fn into_ui_bundle(self) -> Self::Target {
        self
    }
}
impl IntoUiBundle for ImageBundle {
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
// impl IntoUiBundle for NodeBundle {}
