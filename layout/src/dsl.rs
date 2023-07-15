//! The [`LayoutDsl`] type used to bring layout bundles to the [`cuicui_dsl::dsl`] macro.

use bevy::prelude::{Bundle, Deref, DerefMut, Entity};
use cuicui_dsl::{BaseDsl, DslBundle, EntityCommands};

use crate::bundles::{LayoutBundle, RootBundle};
use crate::{Alignment, Container, Distribution, Flow, LeafRule, Node, Oriented, Rule, Size};
#[cfg(doc)]
use crate::{Root, ScreenRoot};

/// Something that can be converted into a bevy [`Bundle`].
///
/// Implement this trait on anything you want, then you can use [`LayoutDsl::spawn_ui`]
/// with anything you want!
///
/// `Marker` is completely ignored. It only exists to make it easier for
/// consumers of the API to extend the DSL with their own bundle.
///
/// # Example
///
/// ```
/// # use bevy::prelude::*;
/// use cuicui_layout::{LayoutDsl, dsl};
/// use cuicui_layout::dsl::IntoUiBundle;
/// use cuicui_layout::dsl_functions::px;
///
/// # #[derive(Component)] struct TextBundle;
/// enum MyDsl {}
///
/// impl IntoUiBundle<MyDsl> for &'_ str {
///     type Target = TextBundle;
///
///     fn into_ui_bundle(self) -> Self::Target {
///         TextBundle {
///             // ...
///             // text: Text::from_section(self, Default::default()),
///             // ...
///         }
///     }
/// }
///
/// fn setup(mut cmds: Commands) {
///
///     dsl! {
///         <LayoutDsl> &mut cmds,
///         spawn_ui("Hello world", width px(350));
///         spawn_ui("Even hi!", width px(350));
///         spawn_ui("Howdy partner", width px(350));
///     };
/// }
/// ```
pub trait IntoUiBundle<Marker> {
    /// The [`Bundle`] this can be converted into.
    type Target: Bundle;

    /// Convert `self` into an [`Self::Target`], will be directly inserted by
    /// [`LayoutDsl::spawn_ui`] with an additional [`Node`] component.
    ///
    /// Since `Target` is inserted _after_ the [`Node`] component, you can
    /// overwrite it by including it in the bundle.
    fn into_ui_bundle(self) -> Self::Target;
}
impl IntoUiBundle<()> for () {
    type Target = ();
    fn into_ui_bundle(self) {}
}

/// Metadata internal to [`LayoutDsl`] to manage the state of things it
/// should be spawning.
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
    /// The inner size, defaults to [`Rule::Children(1.0)`].
    pub size: Size<Option<Rule>>,
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            align: Alignment::Center,
            distrib: Distribution::FillMain,
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

/// The [`DslBundle`] for layouting.
///
/// The defaults are:
///
/// - For containers: Child sized, meaning a container node has the minimum size
///   to fit all its children.
/// - [`Distribution::FillMain`]
/// - [`Alignment::Center`]
///
/// For terminal nodes (spawned through `spawn` or `spawn_ui`) the default
/// size is [`LeafRule::Fixed(0.)`], or content-sized.
///
/// [`EntityCommands`]: bevy::ecs::system::EntityCommands
#[derive(Default, Deref, DerefMut)]
pub struct LayoutDsl<T = BaseDsl> {
    #[deref]
    inner: T,
    root: RootKind,
    layout: Layout,
    set_flow: bool,
}

impl<C: DslBundle> LayoutDsl<C> {
    /// Set the flow direction of a container node.
    pub fn flow(&mut self, flow: Flow) {
        self.set_flow = true;
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
    /// Push children of this [`Node`] to the start of the main flow axis,
    /// the default is [`Distribution::FillMain`].
    pub fn distrib_start(&mut self) {
        self.layout.distrib = Distribution::Start;
    }
    /// Push children of this [`Node`] to the end of the main flow axis,
    /// the default is [`Distribution::FillMain`].
    pub fn distrib_end(&mut self) {
        self.layout.distrib = Distribution::End;
    }
    /// Distribute the children of this [`Node`] to fill this [`Container`]'s main flow axis.
    ///
    /// Note that this is the default.
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

    /// Spawn an empty [`Node::Axis`] with the `main` axis set to `pixels` pixels
    /// and the `cross` axis to 0.
    pub fn empty_px(&mut self, pixels: u16, cmds: &mut EntityCommands) -> Entity {
        let node = Node::Axis(Oriented {
            main: LeafRule::Fixed(pixels as f32, false),
            cross: LeafRule::Fixed(0.0, false),
        });
        cmds.insert(LayoutBundle { node, ..Default::default() })
            .id()
    }
    /// Spawn `ui_bundle`.
    ///
    /// Note that axis without set rules or [`Rule::Children`]
    /// are considered [content-sized](crate::ComputeContentSize).
    pub fn spawn_ui<M>(
        &mut self,
        ui_bundle: impl IntoUiBundle<M>,
        cmds: &mut EntityCommands,
    ) -> Entity {
        let ui_bundle = ui_bundle.into_ui_bundle();
        let size = self.layout.size.map(LeafRule::from_rule);
        cmds.insert(LayoutBundle::boxy(size)).insert(ui_bundle).id()
    }
}
impl<C: DslBundle> DslBundle for LayoutDsl<C> {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        self.inner.insert(cmds);
        if !self.set_flow {
            return self.spawn_ui((), cmds);
        }
        let container = self.layout.container();
        let root_bundle = || RootBundle::new(self.layout);
        let non_screen_root_bundle = || {
            let r = RootBundle::new(self.layout);
            (r.pos_rect, r.root)
        };
        match self.root {
            RootKind::ScreenRoot => cmds.insert(root_bundle()),
            RootKind::Root => cmds.insert(non_screen_root_bundle()),
            RootKind::None => cmds.insert(LayoutBundle::node(container)),
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
