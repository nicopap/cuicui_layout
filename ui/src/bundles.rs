use bevy::{
    ecs::bundle::Bundle,
    text::Text,
    ui::{node_bundles as bevy_ui, UiImage},
};
use cuicui_layout::{
    Alignment, Container, Distribution, Flow, LeafRule, Node, Oriented, PosRect, Root, Size,
};

use crate::{content_size::ContentSize, ScreenRoot};

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
#[derive(Bundle, Default)]
pub struct ButtonBundle {
    pub pos_rect: PosRect,
    pub inner: bevy_ui::ButtonBundle,
    pub container: Node,
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
    pub pos_rect: PosRect,
    pub inner: bevy_ui::ImageBundle,
    pub content_size: ContentSize,
    pub(crate) rules: Node,
}
impl ImageBundle {
    pub fn mut_box_size(&mut self) -> &mut Size<LeafRule> {
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
    pub fn height_rule(mut self, rule: LeafRule) -> Self {
        self.mut_box_size().height = rule;
        self
    }
}

/// A terminal node, meant to add empty space/offsets between other nodes.
#[derive(Bundle)]
pub struct BoxBundle {
    pub pos_rect: PosRect,
    pub inner: bevy_ui::NodeBundle,
    node: Node,
}
impl BoxBundle {
    pub fn axis(orient: Oriented<LeafRule>) -> Self {
        Self {
            pos_rect: PosRect::default(),
            inner: Default::default(),
            node: Node::Axis(orient),
        }
    }
    pub fn sized(size: Size<LeafRule>) -> Self {
        Self {
            pos_rect: PosRect::default(),
            inner: Default::default(),
            node: Node::Box(size),
        }
    }
}
/// A container node, meant to hold one or several other UI elements.
#[derive(Bundle)]
pub struct FlowBundle {
    pub pos_rect: PosRect,
    pub inner: bevy_ui::NodeBundle,
    container: Node,
}
impl Default for FlowBundle {
    fn default() -> Self {
        Self {
            pos_rect: PosRect::default(),
            inner: Default::default(),
            container: Node::Box(Size::all(LeafRule::Fixed(1.0))),
        }
    }
}
impl FlowBundle {
    pub(crate) fn new(container: Container) -> Self {
        let container = Node::Container(container);
        Self { container, ..Default::default() }
    }
}

/// An text leaf node wrapping a [`bevy_ui::TextBundle`].
///
/// By default, a text node will stretch to fit the parent's size.
///
/// In order to have the text be bound to a fixed size, you should use
/// [`LeafRule::Parent`] and wrap the text in a [`FlowBundle`] with a [`Rule::Fixed`].
/// this into a parent node
#[derive(Bundle, Default)]
pub struct TextBundle {
    pub pos_rect: PosRect,
    pub inner: bevy_ui::TextBundle,
    pub content_size: ContentSize,
    rules: Node,
}
impl TextBundle {
    pub fn mut_box_size(&mut self) -> &mut Size<LeafRule> {
        let Node::Box(size) = &mut self.rules else {
            unreachable!("There is no way to make an `TextBundle` with a non-box rule");
        };
        size
    }
    pub fn width_rule(mut self, rule: LeafRule) -> Self {
        self.mut_box_size().width = rule;
        self
    }
    pub fn height_rule(mut self, rule: LeafRule) -> Self {
        self.mut_box_size().height = rule;
        self
    }
}
impl From<Text> for TextBundle {
    fn from(text: Text) -> Self {
        bevy_ui::TextBundle { text, ..Default::default() }.into()
    }
}
impl From<UiImage> for ImageBundle {
    fn from(image: UiImage) -> Self {
        bevy_ui::ImageBundle { image, ..Default::default() }.into()
    }
}

#[derive(Bundle, Default)]
pub struct RootBundle {
    pub inner: bevy_ui::NodeBundle,
    pub pos_rect: PosRect,
    pub root: Root,
    pub screen_root: ScreenRoot,
}
impl RootBundle {
    pub(crate) fn new(flow: Flow, align: Alignment, distrib: Distribution) -> Self {
        RootBundle {
            pos_rect: PosRect::default(),
            inner: Default::default(),
            root: Root { bounds: Size::ZERO, flow, align, distrib },
            screen_root: ScreenRoot,
        }
    }
}

impl_bundle!(ButtonBundle);
impl_bundle!(ImageBundle);
impl_bundle!(TextBundle);
impl From<bevy_ui::NodeBundle> for FlowBundle {
    fn from(inner: bevy_ui::NodeBundle) -> Self {
        Self { inner, ..Default::default() }
    }
}
impl From<bevy_ui::NodeBundle> for RootBundle {
    fn from(inner: bevy_ui::NodeBundle) -> Self {
        Self { inner, ..Default::default() }
    }
}

/// Convert a [`bevy_ui::NodeBundle`] into [`cuicui_layout`]-based bundles.
pub trait NodeBundleFlowExt {
    /// Get a default [`FlowBundle`] from this [`bevy_ui::NodeBundle`].
    fn flow(self) -> FlowBundle;
    /// Get a default [`RootBundle`] from this [`bevy_ui::NodeBundle`].
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
