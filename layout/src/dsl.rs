//! Trait extension to bring layout construction methods to [`Commands`]-like types.
//!
//! See the "Trait Implementations" section of [`LayoutCommands`] for details on
//! what are [`Commands`]-like types.
//!
//! [`Commands`]: bevy::prelude::Commands
#![allow(clippy::module_name_repetitions)]

mod command_like;
mod ui_bundle;

use std::borrow::Cow;

use bevy::prelude::{ChildBuilder, Entity, Name};

use crate::bundles::{FlowBundle, RootBundle};
use crate::{Alignment, Container, Distribution, Flow, Oriented, Rule, Size};
#[cfg(doc)]
use crate::{Node, Root, ScreenRoot};

pub use command_like::{CommandLike, IntoLayoutCommands};
pub use ui_bundle::{IntoUiBundle, UiBundle};

/// Metadata internal to [`LayoutCommands`] to manage the state of things it
/// should be spawning.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Default to [`Alignment::Center`].
    pub align: Alignment,
    /// Default to [`Distribution::Start`].
    pub distrib: Distribution,
    /// The [margin](Container::margin) size.
    pub margin: Oriented<f32>,
    /// The inner size, defaults to [`Rule::Parent(1.0)`].
    pub size: Size<Option<Rule>>,
}

enum RootKind {
    ScreenRoot,
    Root,
    None,
}

// TODO(feat): Use similar compilation checks as in layout/src/typed.rs
/// A wrapper around [`EntityCommands`] with additional  layouting information.
///
/// [`EntityCommands`]: bevy::ecs::system::EntityCommands
pub struct LayoutCommands<C> {
    name: Option<Cow<'static, str>>,
    /// The [`CommandLike`] underlying this layout.
    pub inner: C,
    layout: Layout,
    root: RootKind,
}
impl<C: CommandLike> LayoutCommands<C> {
    /// Convert this [`LayoutCommands<C>`] into a `LayoutCommands` with a different
    /// underlying [`CommandLike`].
    pub fn with<CC: CommandLike>(self, f: impl FnOnce(C) -> CC) -> LayoutCommands<CC> {
        LayoutCommands {
            name: self.name,
            inner: f(self.inner),
            layout: self.layout,
            root: self.root,
        }
    }
    /// Create a new default [`LayoutCommands`].
    pub fn new(inner: C) -> Self {
        LayoutCommands {
            name: None,
            inner,
            layout: Layout {
                align: Alignment::Center,
                distrib: Distribution::Start,
                margin: Oriented::default(),
                size: Size::all(None),
            },
            root: RootKind::None,
        }
    }
    fn container(&self, flow: Flow) -> Container {
        Container {
            flow,
            align: self.layout.align,
            distrib: self.layout.distrib,
            rules: self.layout.size.map(|r| r.unwrap_or(Rule::Parent(1.0))),
            margin: flow.absolute(self.layout.margin),
        }
    }
    fn flow(mut self, flow: Flow, f: impl FnOnce(&mut ChildBuilder)) {
        let container = self.container(flow);
        let root_bundle = || RootBundle::new(flow, self.layout);
        let non_screen_root_bundle = || {
            let r = RootBundle::new(flow, self.layout);
            (r.pos_rect, r.root)
        };
        let cmds = &mut self.inner;
        match self.root {
            RootKind::ScreenRoot => cmds.insert(root_bundle()),
            RootKind::Root => cmds.insert(non_screen_root_bundle()),
            RootKind::None => cmds.insert(FlowBundle::new(container)),
        };
        if let Some(name) = &self.name {
            cmds.insert(Name::new(name.clone()));
        }
        cmds.with_children(f);
    }
    /// Spawn this [`Node`] as a [`Node::Container`] with a single [`Node::Box`]
    /// child, a UI element.
    pub fn spawn_ui<M>(mut self, bundle: impl IntoUiBundle<M>) -> Entity {
        let mut bundle = bundle.into_ui_bundle();
        let set_size = self.layout.size.width.is_some() || self.layout.size.height.is_some();
        if let Some(name) = self.name.take() {
            self.inner.insert(Name::new(name));
        }
        if set_size {
            let mut id = None;
            let rules = self.layout.size.map(|r| r.unwrap_or(Rule::Children(1.0)));
            let container = Container { rules, ..Container::compact(Flow::Horizontal) };
            let bundle_container = FlowBundle::new(container);
            self.inner.insert(bundle_container);
            self.inner.with_children(|cmds| {
                if self.layout.size.width.is_none() {
                    bundle.set_fixed_width();
                }
                if self.layout.size.height.is_none() {
                    bundle.set_fixed_height();
                }
                id = Some(cmds.spawn(bundle).id());
            });
            // SAFETY: this is guarenteed to be set by now.
            unsafe { id.unwrap_unchecked() }
        } else {
            self.inner.insert(bundle);
            self.inner.entity()
        }
    }
    /// Spawn this [`Node`] as a [`Node::Container`] with children flowing vertically.
    ///
    /// `f` will then build the children of this [`Container`].
    pub fn column(self, f: impl FnOnce(&mut ChildBuilder)) {
        self.flow(Flow::Vertical, f);
    }
    /// Spawn this [`Node`] as a [`Node::Container`] with children flowing horizontally.
    ///
    /// `f` will then build the children of this [`Container`].
    pub fn row(self, f: impl FnOnce(&mut ChildBuilder)) {
        self.flow(Flow::Horizontal, f);
    }
    /// Push children of this [`Node`] to the end of the main flow axis,
    /// the default is [`Distribution::Start`].
    ///
    /// > **Warning**: This [`Node`] **Must not** be [`Rule::Children`] on the main flow axis.
    #[must_use]
    pub const fn distrib_end(mut self) -> Self {
        self.layout.distrib = Distribution::End;
        self
    }
    /// Distribute the children of this [`Node`] to fill this [`Container`]'s main flow axis.
    /// the default is [`Distribution::Start`].
    ///
    /// > **Warning**: This [`Node`] **Must not** be [`Rule::Children`] on the main flow axis.
    #[must_use]
    pub const fn fill_main_axis(mut self) -> Self {
        self.layout.distrib = Distribution::FillMain;
        self
    }

    /// Set this [`Container`]'s margin on the main flow axis.
    #[must_use]
    pub const fn main_margin(mut self, pixels: f32) -> Self {
        self.layout.margin.main = pixels;
        self
    }
    /// Set this [`Container`]'s margin on the cross flow axis.
    #[must_use]
    pub const fn cross_margin(mut self, pixels: f32) -> Self {
        self.layout.margin.cross = pixels;
        self
    }
    /// Set the width [`Rule`] of this [`Node`].
    #[must_use]
    pub const fn width_rule(mut self, rule: Rule) -> Self {
        self.layout.size.width = Some(rule);
        self
    }
    /// Set the height [`Rule`] of this [`Node`].
    #[must_use]
    pub const fn height_rule(mut self, rule: Rule) -> Self {
        self.layout.size.height = Some(rule);
        self
    }

    /// Use [`Alignment::Start`] for this [`Node`], the default is [`Alignment::Center`].
    #[must_use]
    pub const fn align_start(mut self) -> Self {
        self.layout.align = Alignment::Start;
        self
    }
    /// Use [`Alignment::End`] for this [`Node`], the default is [`Alignment::Center`].
    #[must_use]
    pub const fn align_end(mut self) -> Self {
        self.layout.align = Alignment::End;
        self
    }

    /// Set this node as the [`ScreenRoot`], its size will follow that of the
    /// [`LayoutRootCamera`] camera.
    ///
    /// [`LayoutRootCamera`]: crate::LayoutRootCamera
    #[must_use]
    pub const fn screen_root(mut self) -> Self {
        self.root = RootKind::ScreenRoot;
        self
    }
    /// Set this node as a [`Root`].
    #[must_use]
    pub const fn root(mut self) -> Self {
        self.root = RootKind::Root;
        self
    }
    /// Set the name of this [`Node`]'s entity.
    #[must_use]
    pub fn named(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Return self (useful for the [`crate::layout!`] macro)
    #[must_use]
    pub const fn lyout(self) -> Self {
        self
    }
}
