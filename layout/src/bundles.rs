//! Bevy [`Bundle`]s grouping components used by the layouting systems.

use bevy::prelude::{default, Bundle};

use crate::{dsl, Container, Node, PosRect, Root, ScreenRoot, Size};

/// A [`Root`] container node, it will always span the entire screen.
#[derive(Bundle, Default)]
pub struct RootBundle {
    /// The positional component.
    pub pos_rect: PosRect,
    /// The container description, as a root.
    pub root: Root,
    /// Sets this [`Root`] to track the [`LayoutRootCamera`]'s size.
    ///
    /// [`LayoutRootCamera`]: crate::LayoutRootCamera
    pub screen_root: ScreenRoot,
}
impl RootBundle {
    /// Create a [`RootBundle`] based on given [`dsl::Layout`].
    #[must_use]
    pub fn new(dsl::Layout { align, distrib, margin, flow, .. }: dsl::Layout) -> Self {
        let size = Size::all(f32::MAX);
        RootBundle {
            pos_rect: default(),
            root: Root::new(size, flow, align, distrib, flow.absolute(margin)),
            screen_root: ScreenRoot,
        }
    }
}

/// A container node, meant to hold one or several other UI elements.
#[derive(Bundle, Default)]
pub struct FlowBundle {
    /// The positional component.
    pub pos_rect: PosRect,
    /// The set of rules this node follows, it should be [`Node::Container`].
    pub container: Node,
}
impl FlowBundle {
    /// A container meant to hold other [`Node`].
    #[must_use]
    pub fn new(container: Container) -> Self {
        let container = Node::Container(container);
        Self { container, ..default() }
    }
}
