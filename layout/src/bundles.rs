//! Bevy [`Bundle`]s grouping components used by the layouting systems.

use bevy::prelude::{default, Bundle};

use crate::{Alignment, Distribution, Oriented, Size};
use crate::{Container, Flow, LayoutRect, LeafRule, Node, Root, Rule, ScreenRoot};

/// Layout information.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// [`Flow`] direction.
    pub flow: Flow,
    /// Default to [`Alignment::Center`].
    pub align: Alignment,
    /// Default to [`Distribution::FillMain`].
    pub distrib: Distribution,
    /// The [margin](Container::margin) size.
    pub margin: Oriented<f32>,
    /// The inner size, defaults to [`Rule::Children(1.5)`].
    pub size: Size<Option<Rule>>,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            align: Alignment::Center,
            distrib: Distribution::FillMain,
            margin: Oriented::default(),
            size: Size::all(None),
            flow: Flow::Horizontal,
        }
    }
}

impl Layout {
    /// Get the `Layout` as a [`Container`], useful with [`LayoutBundle::node`].
    #[must_use]
    pub fn container(&self) -> Container {
        Container {
            flow: self.flow,
            align: self.align,
            distrib: self.distrib,
            rules: self.size.map(|r| r.unwrap_or(Rule::Children(1.5))),
            margin: self.flow.absolute(self.margin),
        }
    }
}

/// A [`Root`] container node, it will always span the entire screen.
#[derive(Bundle, Default)]
pub struct RootBundle {
    /// The positional component.
    pub pos_rect: LayoutRect,
    /// The container description, as a root.
    pub root: Root,
    /// Sets this [`Root`] to track the [`LayoutRootCamera`]'s size.
    ///
    /// [`LayoutRootCamera`]: crate::LayoutRootCamera
    pub screen_root: ScreenRoot,
}
impl RootBundle {
    /// Create a [`RootBundle`] based on given [`Layout`].
    #[must_use]
    pub fn new(Layout { align, distrib, margin, flow, .. }: Layout) -> Self {
        let size = Size::all(f32::MAX);
        Self {
            pos_rect: default(),
            root: Root::new(size, flow, align, distrib, flow.absolute(margin)),
            screen_root: ScreenRoot,
        }
    }
}

/// A layout node, may be terminal or contain other nodes.
#[derive(Bundle, Default, Debug)]
pub struct LayoutBundle {
    /// The positional component.
    pub pos_rect: LayoutRect,
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
