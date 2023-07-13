//! Bevy [`Bundle`]s grouping components used by the layouting systems.

use bevy::prelude::{default, Bundle};

use crate::{dsl, Container, LeafRule, Node, PosRect, Root, ScreenRoot, Size};

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

/// A layout node, may be terminal or contain other nodes.
#[derive(Bundle, Default)]
pub struct LayoutBundle {
    /// The positional component.
    pub pos_rect: PosRect,
    /// The set of rules this node follows.
    pub node: Node,
}
impl LayoutBundle {
    /// A container meant to hold other [`Node`].
    #[must_use]
    pub fn node(container: Container) -> Self {
        Self { node: Node::Container(container), ..default() }
    }
    /// A set-size leaf node.
    #[must_use]
    pub fn boxy(size: Size<LeafRule>) -> Self {
        Self { node: Node::Box(size), ..default() }
    }
}
