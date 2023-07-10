//! Define [`UiBundle`] a trait to extend bevy [`Bundle`] with layout
//! constraint properties.
#![allow(clippy::module_name_repetitions)]

use bevy::prelude::Bundle;

/// A bevy [`Bundle`] that can be spawned as a `cuicui_layout` terminal node.
///
/// [`UiBundle`] differs from [`Bundle`] in that it is possible to set its
/// layouting properties.
///
/// This includes images and text, not much else.
pub trait UiBundle: Bundle {
    /// Mark this [`UiBundle`]'s width as determined by the its content.
    fn width_content_sized_enabled(&mut self) {}

    /// Mark this [`UiBundle`]'s height as determined by the its content.
    fn height_content_sized_enabled(&mut self) {}

    /// Whether this [`UiBundle`] should know its own size.
    fn content_sized(&self) -> bool {
        false
    }
}

/// Something that can be converted into [`UiBundle`].
///
/// `Marker` is completely ignored. It only exists to make it easier for
/// consumers of the API to extend the DSL with their own bundle.
pub trait IntoUiBundle<Marker> {
    /// The [`UiBundle`] this can be converted into.
    type Target: UiBundle;

    /// Convert `self` into an [`UiBundle`].
    fn into_ui_bundle(self) -> Self::Target;
}

/// Dummy implementation, does nothing, useful for testing.
impl UiBundle for () {
    fn width_content_sized_enabled(&mut self) {}
    fn height_content_sized_enabled(&mut self) {}
}
