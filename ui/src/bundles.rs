//! Bundles wrapping [`bevy::ui::node_bundles`] with additional [`cuicui_layout`]
//! components.
use bevy::{
    prelude::{Bundle, Handle, Image, Text, TextStyle, UiImage},
    ui::node_bundles as bevy_ui,
    utils::default,
};
use cuicui_layout::{
    Alignment, Container, Distribution, Flow, LeafRule, Node, Oriented, PosRect, Root, Size,
};

use crate::{content_sized::ContentSized, ScreenRoot};

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

/// A terminal node, meant to add empty space/offsets between other nodes.
#[derive(Bundle)]
pub struct BoxBundle {
    /// The [`cuicui_layout`] positional component.
    pub pos_rect: PosRect,
    /// The bevy bundle.
    pub inner: bevy_ui::NodeBundle,
    node: Node,
}
impl BoxBundle {
    /// A terminal node with given `orient` [`LeafRule`], the axis of the rule
    /// depends on the parent's flow direction.
    #[must_use]
    pub fn axis(orient: Oriented<LeafRule>) -> Self {
        Self {
            pos_rect: default(),
            inner: default(),
            node: Node::Axis(orient),
        }
    }
    /// A terminal node with given `size` [`LeafRule`]. Unlike [`Self::axis`],
    /// the rules are set to a given axis, rather that dependent on the
    /// parent's direction.
    #[must_use]
    pub fn sized(size: Size<LeafRule>) -> Self {
        Self {
            pos_rect: default(),
            inner: default(),
            node: Node::Box(size),
        }
    }
}
/// A container node, meant to hold one or several other UI elements.
#[derive(Bundle)]
pub struct FlowBundle {
    /// The [`cuicui_layout`] positional component.
    pub pos_rect: PosRect,
    /// The bevy bundle.
    pub inner: bevy_ui::NodeBundle,
    container: Node,
}
impl Default for FlowBundle {
    fn default() -> Self {
        Self {
            pos_rect: PosRect::default(),
            inner: default(),
            container: Node::Box(Size::all(LeafRule::Fixed(1.0))),
        }
    }
}
impl FlowBundle {
    pub(crate) fn new(container: Container) -> Self {
        let container = Node::Container(container);
        Self { container, ..default() }
    }
}

/// A text leaf node wrapping a [`bevy_ui::TextBundle`].
///
/// By default, a text node will stretch to fit the parent's size.
///
/// In order to have the text be bound to a fixed size, you should use
/// [`LeafRule::Parent`] and wrap the text in a [`FlowBundle`] with a [`Rule::Fixed`].
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

/// A [`Root`] container node, it will always span the entire screen.
#[derive(Bundle, Default)]
pub struct RootBundle {
    /// The bevy bundle.
    pub inner: bevy_ui::NodeBundle,
    /// The [`cuicui_layout`] positional component.
    pub pos_rect: PosRect,
    /// The [`cuicui_layout::Root`].
    pub root: Root,
    /// Sets this [`cuicui_layout::Root`] to track the [`LayoutRootCamera`]'s size.
    ///
    /// [`LayoutRootCamera`]: crate::LayoutRootCamera
    pub screen_root: ScreenRoot,
}
impl RootBundle {
    #[must_use]
    pub(crate) fn new(flow: Flow, align: Alignment, distrib: Distribution) -> Self {
        RootBundle {
            pos_rect: default(),
            inner: default(),
            root: Root::new(Size::ZERO, flow, align, distrib, Size::ZERO),
            screen_root: ScreenRoot,
        }
    }
}

impl_bundle!(ImageBundle);
impl_bundle!(TextBundle);
impl From<bevy_ui::NodeBundle> for FlowBundle {
    fn from(inner: bevy_ui::NodeBundle) -> Self {
        Self { inner, ..default() }
    }
}
impl From<bevy_ui::NodeBundle> for RootBundle {
    fn from(inner: bevy_ui::NodeBundle) -> Self {
        Self { inner, ..default() }
    }
}

/// Convert a [`bevy_ui::NodeBundle`] into [`cuicui_layout`]-based bundles.
pub trait NodeBundleFlowExt {
    /// Get a default [`FlowBundle`] from this [`bevy_ui::NodeBundle`].
    #[must_use]
    fn flow(self) -> FlowBundle;
    /// Get a default [`RootBundle`] from this [`bevy_ui::NodeBundle`].
    #[must_use]
    fn root(self) -> RootBundle;
}
impl NodeBundleFlowExt for bevy_ui::NodeBundle {
    fn flow(self) -> FlowBundle {
        FlowBundle::from(self)
    }
    fn root(self) -> RootBundle {
        RootBundle::from(self)
    }
}

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
pub trait IntoUiBundle {
    /// The type of the [`UiBundle`] it can be converted into.
    type Target: UiBundle;
    /// Convert `self` into an [`UiBundle`].
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
