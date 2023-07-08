//! The base [`MakeBundle`] for layouts.
#![allow(clippy::module_name_repetitions)]

mod command_like;
mod ui_bundle;

use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use bevy::prelude::{BuildChildren, Entity, Name};

use crate::bundles::{FlowBundle, RootBundle};
use crate::{Alignment, Container, Distribution, Flow, Oriented, Rule, Size};
#[cfg(doc)]
use crate::{Node, Root, ScreenRoot};

pub use bevy::ecs::system::EntityCommands;
pub use command_like::{InsertKind, MakeBundle, MakeSpawner};
pub use ui_bundle::{IntoUiBundle, UiBundle};

/// Metadata internal to [`LayoutType`] to manage the state of things it
/// should be spawning.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// [`Flow`] direction.
    pub flow: Flow,
    /// Default to [`Alignment::Center`].
    pub align: Alignment,
    /// Default to [`Distribution::Start`].
    pub distrib: Distribution,
    /// The [margin](Container::margin) size.
    pub margin: Oriented<f32>,
    // TODO(feat): consider changing the default to Rule::Children(1.0) when layout is wihin another container.
    /// The inner size, defaults to [`Rule::Parent(1.0)`].
    pub size: Size<Option<Rule>>,
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            align: Alignment::Center,
            distrib: Distribution::Start,
            margin: Oriented::default(),
            size: Size::all(None),
            flow: Flow::Horizontal,
        }
    }
}

impl Layout {
    fn container(&self) -> Container {
        Container {
            flow: self.flow,
            align: self.align,
            distrib: self.distrib,
            rules: self.size.map(|r| r.unwrap_or(Rule::Parent(1.0))),
            margin: self.flow.absolute(self.margin),
        }
    }
}

#[derive(Default)]
enum RootKind {
    ScreenRoot,
    Root,
    #[default]
    None,
}

/// A wrapper around [`EntityCommands`] with additional  layouting information.
///
/// [`EntityCommands`]: bevy::ecs::system::EntityCommands
#[derive(Default)]
pub struct LayoutType<T = ()> {
    inner: T,
    name: Option<Cow<'static, str>>,
    root: RootKind,
    layout: Layout,
}
impl<T> Deref for LayoutType<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> DerefMut for LayoutType<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<C: MakeBundle> LayoutType<C> {
    /// Set the flow direction of a container node.
    pub fn flow(&mut self, flow: Flow) {
        self.layout.flow = flow;
    }
    /// Spawn this [`Node`] as a [`Node::Container`] with children flowing vertically.
    ///
    /// `f` will then build the children of this [`Container`].
    pub fn column(&mut self) {
        self.flow(Flow::Vertical);
    }
    /// Spawn this [`Node`] as a [`Node::Container`] with children flowing horizontally.
    ///
    /// `f` will then build the children of this [`Container`].
    pub fn row(&mut self) {
        self.flow(Flow::Horizontal);
    }
    /// Push children of this [`Node`] to the end of the main flow axis,
    /// the default is [`Distribution::Start`].
    ///
    /// > **Warning**: This [`Node`] **Must not** be [`Rule::Children`] on the main flow axis.
    pub fn distrib_end(&mut self) {
        self.layout.distrib = Distribution::End;
    }
    /// Distribute the children of this [`Node`] to fill this [`Container`]'s main flow axis.
    /// the default is [`Distribution::Start`].
    ///
    /// > **Warning**: This [`Node`] **Must not** be [`Rule::Children`] on the main flow axis.
    pub fn fill_main_axis(&mut self) {
        self.layout.distrib = Distribution::FillMain;
    }

    /// Set this [`Container`]'s margin on the main flow axis.
    pub fn main_margin(&mut self, pixels: f32) {
        self.layout.margin.main = pixels;
    }
    /// Set this [`Container`]'s margin on the cross flow axis.
    pub fn cross_margin(&mut self, pixels: f32) {
        self.layout.margin.cross = pixels;
    }
    /// Set the width [`Rule`] of this [`Node`].
    pub fn width_rule(&mut self, rule: Rule) {
        self.layout.size.width = Some(rule);
    }
    /// Set the height [`Rule`] of this [`Node`].
    pub fn height_rule(&mut self, rule: Rule) {
        self.layout.size.height = Some(rule);
    }

    /// Use [`Alignment::Start`] for this [`Node`], the default is [`Alignment::Center`].
    pub fn align_start(&mut self) {
        self.layout.align = Alignment::Start;
    }
    /// Use [`Alignment::End`] for this [`Node`], the default is [`Alignment::Center`].
    pub fn align_end(&mut self) {
        self.layout.align = Alignment::End;
    }

    /// Set this node as the [`ScreenRoot`], its size will follow that of the
    /// [`LayoutRootCamera`] camera.
    ///
    /// [`LayoutRootCamera`]: crate::LayoutRootCamera
    pub fn screen_root(&mut self) {
        self.root = RootKind::ScreenRoot;
    }
    /// Set this node as a [`Root`].
    pub fn root(&mut self) {
        self.root = RootKind::Root;
    }
    /// Set the name of this [`Node`]'s entity.
    pub fn named(&mut self, name: impl Into<Cow<'static, str>>) {
        self.name = Some(name.into());
    }

    fn spawn_node(mut self, cmds: &mut EntityCommands) -> Entity {
        self.inner.insert(InsertKind::Node, cmds);
        let container = self.layout.container();
        let root_bundle = || RootBundle::new(self.layout);
        let non_screen_root_bundle = || {
            let r = RootBundle::new(self.layout);
            (r.pos_rect, r.root)
        };
        match self.root {
            RootKind::ScreenRoot => cmds.insert(root_bundle()),
            RootKind::Root => cmds.insert(non_screen_root_bundle()),
            RootKind::None => cmds.insert(FlowBundle::new(container)),
        };
        if let Some(name) = self.name.take() {
            cmds.insert(Name::new(name));
        }
        cmds.id()
    }
    fn spawn_leaf(mut self, cmds: &mut EntityCommands) -> Entity {
        self.inner.insert(InsertKind::Leaf, cmds);
        let set_size = self.layout.size.width.is_some() || self.layout.size.height.is_some();
        if let Some(name) = self.name.take() {
            cmds.insert(Name::new(name));
        }
        if set_size {
            let rules = self.layout.size.map(|r| r.unwrap_or(Rule::Children(1.0)));
            let container = Container { rules, ..Container::compact(Flow::Horizontal) };
            let bundle_container = FlowBundle::new(container);
            cmds.insert(bundle_container);

            let id = cmds.commands().spawn_empty().id();
            cmds.add_child(id);
            id
        } else {
            cmds.id()
        }
    }
}
impl<C: MakeBundle> MakeBundle for LayoutType<C> {
    fn insert(self, insert: InsertKind, cmds: &mut EntityCommands) -> Entity {
        match insert {
            InsertKind::Node => self.spawn_node(cmds),
            InsertKind::Leaf => self.spawn_leaf(cmds),
        }
    }
    fn ui_content_axis(&self) -> Size<bool> {
        self.layout.size.map(|t| t.is_none())
    }
}
