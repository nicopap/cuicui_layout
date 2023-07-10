//! The [`LayoutDsl`] type used to bring layout bundles to the [`cuicui_dsl::dsl`] macro.

use std::ops::{Deref, DerefMut};

use bevy::prelude::{BuildChildren, Component, Entity};
#[cfg(feature = "reflect")]
use bevy::prelude::{FromReflect, Reflect, ReflectComponent};
use cuicui_dsl::{BaseDsl, DslBundle, EntityCommands};

use crate::bundles::{FlowBundle, RootBundle};
use crate::Node;
use crate::{Alignment, Container, Distribution, Flow, LeafRule, Oriented, Rule, Size};
#[cfg(doc)]
use crate::{Root, ScreenRoot};

pub use crate::ui_bundle::{IntoUiBundle, UiBundle};

/// Dynamically update the [`Node::Box`] rules fixed values of UI entities with
/// its native content.
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect), reflect(Component))]
pub struct ContentSized(pub Size<bool>);

/// Metadata internal to [`LayoutDsl`] to manage the state of things it
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
    /// The inner size, defaults to [`Rule::Children(1.0)`].
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
            rules: self.size.map(|r| r.unwrap_or(Rule::Children(1.0))),
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
pub struct LayoutDsl<T = BaseDsl> {
    inner: T,
    root: RootKind,
    layout: Layout,
}
impl<T> Deref for LayoutDsl<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> DerefMut for LayoutDsl<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<C: DslBundle> LayoutDsl<C> {
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
    pub fn width(&mut self, rule: Rule) {
        self.layout.size.width = Some(rule);
    }
    /// Set the height [`Rule`] of this [`Node`].
    pub fn height(&mut self, rule: Rule) {
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

    /// Spawn `ui_bundle` as an [`UiBundle`].
    ///
    /// If `ui_bundle` is "content sized" (ie: `ui_bundle.content_sized()`
    /// returns `true` **and** one of the axis for this statement wasn't
    /// set), then it will be spawned as a child of this node, and its size
    /// will track that of the content, rather than be defined by the layout
    /// algorithm.
    pub fn spawn_ui<M>(
        &mut self,
        ui_bundle: impl IntoUiBundle<M>,
        mut cmds: EntityCommands,
    ) -> Entity {
        use LeafRule::{Fixed, Parent};
        let mut ui_bundle = ui_bundle.into_ui_bundle();

        let content_defined = self.layout.size.map(|t| t.is_none());
        if content_defined.width {
            ui_bundle.width_content_sized_enabled();
        }
        if content_defined.height {
            ui_bundle.height_content_sized_enabled();
        }
        if ui_bundle.content_sized() {
            let child_node = Node::Box(Size {
                width: if content_defined.width { Fixed(1.0) } else { Parent(1.0) },
                height: if content_defined.height { Fixed(1.0) } else { Parent(1.0) },
            });

            let id = cmds.commands().spawn(ui_bundle).insert(child_node).id();
            cmds.add_child(id);
            id
        } else {
            let self_node = Node::Box(self.layout.size.map(LeafRule::from_rule));
            cmds.insert(self_node)
                .insert(ui_bundle)
                .remove::<ContentSized>()
                .id()
        }
    }
}
impl<C: DslBundle> DslBundle for LayoutDsl<C> {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        self.inner.insert(cmds);
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
        cmds.id()
    }
}

/// Returns [`Rule::Fixed`] with given `pixels`.
#[must_use]
pub const fn px(pixels: u16) -> Rule {
    Rule::Fixed(pixels as f32)
}
/// Returns [`Rule::Parent`] as `percent` percent of parent size.
///
/// # Panics
/// If `percent` is greater than 100. It would mean this node overflows its parent.
#[must_use]
pub fn pct(percent: u8) -> Rule {
    assert!(percent <= 100);
    Rule::Parent(f32::from(percent) / 100.0)
}
/// Returns [`Rule::Children`] as `ratio` of its children size.
///
/// # Panics
/// If `ratio` is smaller than 1. It would mean the container couldn't fit its
/// children.
#[must_use]
pub fn child(ratio: f32) -> Rule {
    assert!(ratio >= 1.0);
    Rule::Children(ratio)
}
