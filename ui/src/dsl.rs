use std::borrow::Cow;

use bevy::{
    ecs::system::EntityCommands,
    prelude::{BuildChildren, ChildBuilder, Commands, Entity, Name},
};
use cuicui_layout::{Alignment, Container, Distribution, Flow, Oriented, Rule, Size};
#[cfg(doc)]
use cuicui_layout::{Node, Root};

use crate::bundles::{FlowBundle, IntoUiBundle, Layout, RootBundle, UiBundle};
use crate::ScreenRoot;

enum RootKind {
    ScreenRoot,
    Root,
    None,
}

// TODO(feat): Use similar compilation checks as in layout/src/typed.rs
/// A wrapper around [`EntityCommands`] with additional [`cuicui_layout`]
/// layouting informations.
pub struct LayoutCommands<'w, 's, 'a> {
    name: Option<Cow<'static, str>>,
    inner: EntityCommands<'w, 's, 'a>,
    layout: Layout,
    root: RootKind,
}
impl<'w, 's, 'a> LayoutCommands<'w, 's, 'a> {
    fn new(inner: EntityCommands<'w, 's, 'a>) -> Self {
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
        let cmds = &mut self.inner;
        match self.root {
            RootKind::ScreenRoot => cmds.insert(root_bundle()),
            RootKind::Root => cmds.insert(root_bundle()).remove::<ScreenRoot>(),
            RootKind::None => cmds.insert(FlowBundle::new(container)),
        };
        if let Some(name) = &self.name {
            cmds.insert(Name::new(name.clone()));
        }
        cmds.with_children(f);
    }
    fn spawn_ui(mut self, bundle: impl IntoUiBundle) -> Entity {
        let mut bundle = bundle.into_ui_bundle();
        let set_size = matches!(
            self.layout.size,
            Size { width: Some(_), .. } | Size { height: Some(_), .. }
        );
        if let Some(name) = self.name.take() {
            self.inner.insert(Name::new(name));
        }
        if set_size {
            let mut id = None;
            let rules = self.layout.size.map(|r| r.unwrap_or(Rule::Children(1.0)));
            let container = Container { rules, ..Container::compact(Flow::Horizontal) };
            let bundle_container = FlowBundle::new(container);
            self.inner.insert(bundle_container).with_children(|cmds| {
                if self.layout.size.width.is_none() {
                    bundle.set_fixed_width();
                }
                if self.layout.size.height.is_none() {
                    bundle.set_fixed_height();
                }
                id = Some(cmds.spawn(bundle).id());
            });
            id.unwrap()
        } else {
            self.inner.insert(bundle).id()
        }
    }
    fn column(self, f: impl FnOnce(&mut ChildBuilder)) {
        self.flow(Flow::Vertical, f);
    }
    fn row(self, f: impl FnOnce(&mut ChildBuilder)) {
        self.flow(Flow::Horizontal, f);
    }
    #[must_use]
    const fn distrib_end(mut self) -> Self {
        self.layout.distrib = Distribution::End;
        self
    }
    #[must_use]
    const fn fill_main_axis(mut self) -> Self {
        self.layout.distrib = Distribution::FillMain;
        self
    }

    #[must_use]
    const fn main_margin(mut self, pixels: f32) -> Self {
        self.layout.margin.main = pixels;
        self
    }
    #[must_use]
    const fn cross_margin(mut self, pixels: f32) -> Self {
        self.layout.margin.cross = pixels;
        self
    }
    // #[must_use]
    // const fn cross_margin(mut self, pixels: f32) -> Self {
    //     self.layout.margin.cross = pixels;
    //     self
    // }
    // #[must_use]
    // const fn main_margin_pct(mut self, percent: f32) -> Self;
    // #[must_use]
    // const fn cross_margin_pct(mut self, percent: f32) -> Self;

    #[must_use]
    const fn width_rule(mut self, rule: Rule) -> Self {
        self.layout.size.width = Some(rule);
        self
    }
    #[must_use]
    const fn height_rule(mut self, rule: Rule) -> Self {
        self.layout.size.height = Some(rule);
        self
    }

    #[must_use]
    const fn align_start(mut self) -> Self {
        self.layout.align = Alignment::Start;
        self
    }
    #[must_use]
    const fn align_end(mut self) -> Self {
        self.layout.align = Alignment::End;
        self
    }

    #[must_use]
    const fn screen_root(mut self) -> Self {
        self.root = RootKind::ScreenRoot;
        self
    }
    #[must_use]
    const fn root(mut self) -> Self {
        self.root = RootKind::Root;
        self
    }
    #[must_use]
    fn named(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Shorthand for [`LayoutCommands`].
pub type Lec<'w, 's, 'a> = LayoutCommands<'w, 's, 'a>;

// Note that the `fn method() { self.into().method() }` always calls the implementation
// of the inherent `.method()` impl in LayoutCommands (the previous `impl` block).
//
// Since the trait has a `Into<Lec<'w, 's, 'a>>` bound, `self.into()` becomes `Lec`,
// and rust calls the inherent impl for the method. Even if `Lec` itself implements
// `LayoutCommandsExt`
//
// See <https://users.rust-lang.org/t/method-call-resolution-behaviour/59492/5>

/// Add methods to various command types to make it easier to spawn layouts.
#[rustfmt::skip]
pub trait LayoutCommandsExt<'w, 's, 'a> : Into<Lec<'w, 's, 'a>> where 'w: 'a, 's: 'a {
    /// Spawn this [`Node`] as a [`Node::Container`] with a single [`Node::Box`]
    /// child, a UI element.
    fn spawn_ui<B: IntoUiBundle>(self, bundle: B) -> Entity { self.into().spawn_ui(bundle) }
    /// Spawn this [`Node`] as a [`Node::Container`] with children flowing vertically.
    ///
    /// `f` will then build the children of this [`Container`].
    fn column<F: FnOnce(&mut ChildBuilder)>(self, f: F) { self.into().column(f) }
    /// Spawn this [`Node`] as a [`Node::Container`] with children flowing horizontally.
    ///
    /// `f` will then build the children of this [`Container`].
    fn row<F: FnOnce(&mut ChildBuilder)>(self, f: F) { self.into().row(f) }

    /// Push children of this [`Node`] to the end of the main flow axis,
    /// the default is [`Distribution::Start`].
    ///
    /// > **Warning**: This [`Node`] **Must not** be [`Rule::Children`] on the main flow axis.
    #[must_use]
    fn distrib_end(self) -> Lec<'w, 's, 'a> {self.into().distrib_end()}
    /// Distribute the children of this [`Node`] to fill this [`Container`]'s main flow axis.
    /// the default is [`Distribution::Start`].
    ///
    /// > **Warning**: This [`Node`] **Must not** be [`Rule::Children`] on the main flow axis.
    #[must_use]
    fn fill_main_axis(self) -> Lec<'w, 's, 'a> {self.into().fill_main_axis()}

    /// Set this [`Container`]'s margin on the main flow axis.
    #[must_use]
    fn main_margin(self, pixels: f32) -> Lec<'w, 's, 'a> {self.into().main_margin(pixels)}

    /// Set this [`Container`]'s margin on the cross flow axis.
    #[must_use]
    fn cross_margin(self, pixels: f32) -> Lec<'w, 's, 'a> {self.into().cross_margin(pixels)}

    /// Set the width [`Rule`] of this [`Node`].
    #[must_use]
    fn width_rule(self, rule: Rule) -> Lec<'w, 's, 'a> {self.into().width_rule(rule)}
    /// Set the height [`Rule`] of this [`Node`].
    #[must_use]
    fn height_rule(self, rule: Rule) -> Lec<'w, 's, 'a> {self.into().height_rule(rule)}

    /// Use [`Alignment::Start`] for this [`Node`], the default is [`Alignment::Center`].
    #[must_use]
    fn align_start(self) -> Lec<'w, 's, 'a> {self.into().align_start()}
    /// Use [`Alignment::End`] for this [`Node`], the default is [`Alignment::Center`].
    #[must_use]
    fn align_end(self) -> Lec<'w, 's, 'a> {self.into().align_end()}

    /// Set this node as the [`ScreenRoot`], its size will follow that of the
    /// [`LayoutRootCamera`] camera.
    ///
    /// [`LayoutRootCamera`]: crate::LayoutRootCamera
    #[must_use]
    fn screen_root(self) -> Lec<'w, 's, 'a> {self.into().screen_root()}
    /// Set this node as a [`Root`].
    #[must_use]
    fn root(self) -> Lec<'w, 's, 'a> {self.into().root()}

    /// Set the name of this [`Node`]'s entity.
    #[must_use]
    fn named<N: Into<Cow<'static, str>>>(self, name: N) -> Lec<'w, 's, 'a> { self.into().named(name) }

}
impl<'w: 'a, 's: 'a, 'a, T: Into<Lec<'w, 's, 'a>>> LayoutCommandsExt<'w, 's, 'a> for T {}

impl<'w, 's, 'a> From<&'a mut Commands<'w, 's>> for LayoutCommands<'w, 's, 'a> {
    fn from(value: &'a mut Commands<'w, 's>) -> Self {
        LayoutCommands::new(value.spawn_empty())
    }
}
impl<'w, 's, 'a> From<EntityCommands<'w, 's, 'a>> for LayoutCommands<'w, 's, 'a> {
    fn from(value: EntityCommands<'w, 's, 'a>) -> Self {
        LayoutCommands::new(value)
    }
}

impl<'w, 's, 'a> From<&'a mut ChildBuilder<'w, 's, '_>> for LayoutCommands<'w, 's, 'a> {
    fn from(value: &'a mut ChildBuilder<'w, 's, '_>) -> Self {
        LayoutCommands::new(value.spawn_empty())
    }
}
