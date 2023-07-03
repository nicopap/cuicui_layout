use std::borrow::Cow;

use bevy::{
    ecs::system::EntityCommands,
    prelude::{BuildChildren, ChildBuilder, Commands, Entity, Name},
};
use cuicui_layout::{Alignment, Container, Distribution, Flow, LeafRule, Oriented, Rule, Size};

use crate::{
    bundles::{BoxBundle, FlowBundle, RootBundle},
    into_ui_bundle::{IntoUiBundle, UiBundle},
    ScreenRoot,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Layout {
    // Default to center alignment.
    pub align: Alignment,
    // Default to Start Distribution
    pub distrib: Distribution,
    // NOTE: margin is incompatible with Distribution::FillParent. When
    // margin is set, and FillParent, must spawn an outer container in the
    // form [spacer, inner, spacer] with Distribution::Start and THEN
    // spawn inner as a container with Distribution::FillParent.
    // TODO: forbid several children with FillParent.
    // TODO: check that single FillParent with sibling Rule::Parent(%) works.
    // TODO: Oriented<LeafRule> / margin: Size
    pub margin: Oriented<f32>,
    pub size: Size<Option<Rule>>,
}

enum RootKind {
    ScreenRoot,
    Root,
    None,
}
// TODO(feat): The `layout!` macro that is a thin wrapper around LayoutEntityCommands
// TODO(feat): Use similar compilation checks as in layout/src/typed.rs
pub struct LayoutEntityCommands<'w, 's, 'a> {
    name: Option<Cow<'static, str>>,
    inner: EntityCommands<'w, 's, 'a>,
    layout: Layout,
    root: RootKind,
}
impl<'w, 's, 'a> LayoutEntityCommands<'w, 's, 'a> {
    fn new(inner: EntityCommands<'w, 's, 'a>) -> Self {
        LayoutEntityCommands {
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
            size: self.layout.size.map(|r| r.unwrap_or(Rule::Parent(1.0))),
        }
    }
    fn flow(mut self, flow: Flow, f: impl FnOnce(&mut ChildBuilder)) {
        let container = self.container(flow);
        let root_bundle = || RootBundle::new(flow, self.layout.align, self.layout.distrib);
        let cmds = &mut self.inner;
        match self.root {
            RootKind::ScreenRoot => cmds.insert(root_bundle()),
            RootKind::Root => cmds.insert(root_bundle()).remove::<ScreenRoot>(),
            RootKind::None => cmds.insert(FlowBundle::new(container)),
        };
        let main_margin = (self.layout.margin.main != 0.0).then_some(Oriented {
            main: LeafRule::Fixed(self.layout.margin.main),
            cross: LeafRule::Fixed(0.0),
        });
        let f = |cmds: &mut ChildBuilder| {
            if let Some(main_margin) = main_margin {
                //TODO: nest if distribution is FillParent
                let mut entity = cmds.spawn(BoxBundle::axis(main_margin));
                if let Some(name) = &self.name {
                    entity.insert(Name::new(format!("{name} start margin")));
                }
            }
            f(cmds);
            if let Some(main_margin) = main_margin {
                //TODO: nest if distribution is FillParent
                let mut entity = cmds.spawn(BoxBundle::axis(main_margin));
                if let Some(name) = &self.name {
                    entity.insert(Name::new(format!("{name} end margin")));
                }
            }
        };
        if let Some(name) = &self.name {
            cmds.insert(Name::new(name.clone()));
        }
        cmds.with_children(f);
    }
    fn column(self, f: impl FnOnce(&mut ChildBuilder)) {
        self.flow(Flow::Vertical, f);
    }
    fn row(self, f: impl FnOnce(&mut ChildBuilder)) {
        self.flow(Flow::Horizontal, f);
    }
    fn distrib_end(mut self) -> Self {
        self.layout.distrib = Distribution::End;
        self
    }
    fn fill_main_axis(mut self) -> Self {
        self.layout.distrib = Distribution::FillMain;
        self
    }

    fn main_margin(mut self, pixels: f32) -> Self {
        self.layout.margin.main = pixels;
        self
    }
    // fn cross_margin(mut self, pixels: f32) -> Self {
    //     self.layout.margin.cross = pixels;
    //     self
    // }
    // fn main_margin_pct(mut self, percent: f32) -> Self;
    // fn cross_margin_pct(mut self, percent: f32) -> Self;

    fn width_rule(mut self, rule: Rule) -> Self {
        self.layout.size.width = Some(rule);
        self
    }
    fn height_rule(mut self, rule: Rule) -> Self {
        self.layout.size.height = Some(rule);
        self
    }

    fn align_start(mut self) -> Self {
        self.layout.align = Alignment::Start;
        self
    }
    fn align_end(mut self) -> Self {
        self.layout.align = Alignment::End;
        self
    }

    fn screen_root(mut self) -> Self {
        self.root = RootKind::ScreenRoot;
        self
    }
    fn root(mut self) -> Self {
        self.root = RootKind::Root;
        self
    }
    fn named(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(name.into());
        self
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
            let size = self.layout.size.map(|r| r.unwrap_or(Rule::Children(1.0)));
            let container = Container { size, ..Container::compact(Flow::Horizontal) };
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
}

pub type Lec<'w, 's, 'a> = LayoutEntityCommands<'w, 's, 'a>;

// Note that the `fn method() { self.into().method() }` always calls the implementation
// of the inherent `.method()` impl in LayoutEntityCommands (the previous `impl` block).
//
// Since the trait has a `Into<Lec<'w, 's, 'a>>` bound, `self.into()` becomes `Lec`,
// and rust calls the inherent impl for the method. Even if `Lec` itself implements
// `LayoutCommandsExt`
//
// See <https://users.rust-lang.org/t/method-call-resolution-behaviour/59492/5>

/// Add methods to various command types to make it easier to spawn layouts.
#[rustfmt::skip]
pub trait LayoutCommandsExt<'w, 's, 'a> : Into<Lec<'w, 's, 'a>> where 'w: 'a, 's: 'a {
    fn column<F: FnOnce(&mut ChildBuilder)>(self, f: F) { self.into().column(f) }
    fn row<F: FnOnce(&mut ChildBuilder)>(self, f: F) { self.into().row(f) }

    fn distrib_end(self) -> Lec<'w, 's, 'a> {self.into().distrib_end()}
    fn fill_main_axis(self) -> Lec<'w, 's, 'a> {self.into().fill_main_axis()}

    fn main_margin(self, pixels: f32) -> Lec<'w, 's, 'a> {self.into().main_margin(pixels)}
    // fn cross_margin(self, pixels: f32) -> Lec<'w, 's, 'a> {self.into().cross_margin(pixels)}
    // fn main_margin_pct(self, percent: f32) -> Lec<'w, 's, 'a>;
    // fn cross_margin_pct(self, percent: f32) -> Lec<'w, 's, 'a>;

    fn width_rule(self, rule: Rule) -> Lec<'w, 's, 'a> {self.into().width_rule(rule)}
    fn height_rule(self, rule: Rule) -> Lec<'w, 's, 'a> {self.into().height_rule(rule)}

    fn align_start(self) -> Lec<'w, 's, 'a> {self.into().align_start()}
    fn align_end(self) -> Lec<'w, 's, 'a> {self.into().align_end()}

    fn screen_root(self) -> Lec<'w, 's, 'a> {self.into().screen_root()}
    fn root(self) -> Lec<'w, 's, 'a> {self.into().root()}

    fn named<N: Into<Cow<'static, str>>>(self, name: N) -> Lec<'w, 's, 'a> { self.into().named(name) }

    fn spawn_ui<B: IntoUiBundle>(self, bundle: B) -> Entity {self.into().spawn_ui(bundle)}
}
impl<'w: 'a, 's: 'a, 'a, T: Into<Lec<'w, 's, 'a>>> LayoutCommandsExt<'w, 's, 'a> for T {}

impl<'w, 's, 'a> From<&'a mut Commands<'w, 's>> for LayoutEntityCommands<'w, 's, 'a> {
    fn from(value: &'a mut Commands<'w, 's>) -> Self {
        LayoutEntityCommands::new(value.spawn_empty())
    }
}
impl<'w, 's, 'a> From<EntityCommands<'w, 's, 'a>> for LayoutEntityCommands<'w, 's, 'a> {
    fn from(value: EntityCommands<'w, 's, 'a>) -> Self {
        LayoutEntityCommands::new(value)
    }
}

impl<'w, 's, 'a> From<&'a mut ChildBuilder<'w, 's, '_>> for LayoutEntityCommands<'w, 's, 'a> {
    fn from(value: &'a mut ChildBuilder<'w, 's, '_>) -> Self {
        LayoutEntityCommands::new(value.spawn_empty())
    }
}
