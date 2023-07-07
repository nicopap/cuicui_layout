use bevy::prelude::Bundle;

/// A bevy [`Bundle`] that can be spawned as a `cuicui_layout` terminal node.
///
/// [`UiBundle`] differs from [`Bundle`] in that it is possible to set its
/// layouting properties.
///
/// This includes images and text, not much else.
pub trait UiBundle: Bundle {
    /// Mark this [`UiBundle`]'s width as fixed to the dynamic size of what
    /// it contains.
    ///
    /// This will be the size of an image or text.
    fn set_fixed_width(&mut self);
    /// Mark this [`UiBundle`]'s height as fixed to the dynamic size of what
    /// it contains.
    ///
    /// This will be the size of an image or text.
    fn set_fixed_height(&mut self);
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
    fn set_fixed_width(&mut self) {}
    fn set_fixed_height(&mut self) {}
}
