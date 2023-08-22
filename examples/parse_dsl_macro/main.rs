use std::{fmt, num::ParseIntError, str::FromStr};

use bevy::{
    ecs::{prelude::*, system::SystemState},
    log::Level,
    prelude::{App, BuildChildren, Deref, DerefMut, Parent, Plugin},
    reflect::{Reflect, TypeRegistryInternal as TypeRegistry},
    utils::HashMap,
};
use cuicui_chirp::{parse_dsl_impl, Chirp, Handles, ParseDsl};
use cuicui_dsl::{dsl, BaseDsl, DslBundle, EntityCommands, Name};
use pretty_assertions::assert_eq;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Reflect)]
enum Flow {
    #[default]
    None,
    Top,
    Left,
}
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Rule {
    Pct(i32),
    Px(i32),
    #[default]
    None,
}
impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rule::Pct(v) => write!(f, "{v}pct"),
            Rule::Px(v) => write!(f, "{v}px"),
            Rule::None => write!(f, "none"),
        }
    }
}
impl fmt::Debug for Pixels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} pixels", self.0)
    }
}
const fn pct(i: i32) -> Rule {
    Rule::Pct(i)
}
const fn px(i: i32) -> Rule {
    Rule::Px(i)
}
impl FromStr for Rule {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match () {
            () if s.starts_with("px(") => {
                let number = &s[3..s.len() - 1];
                Ok(Rule::Px(dbg!(number.parse())?))
            }
            () if s.starts_with("pct(") => {
                let number = &s[4..s.len() - 1];
                Ok(Rule::Pct(dbg!(number.parse())?))
            }
            () => Err("badnumber".parse::<i32>().unwrap_err()),
        }
    }
}
#[derive(Debug, Default, Deref, DerefMut)]
struct LayoutDsl<T = BaseDsl> {
    #[deref]
    inner: T,
    width: Rule,
    height: Rule,
    flow: Flow,
    px: Option<u16>,
}

#[derive(Debug, Clone, Component, PartialEq, Eq, PartialOrd, Ord)]
struct LayoutNode {
    width: Rule,
    height: Rule,
    flow: Flow,
}
#[derive(Clone, Component, PartialEq, Eq, PartialOrd, Ord)]
struct Pixels(u16);

fn show<T: Clone>(t: Option<&T>) -> Show<T> {
    Show(t.cloned())
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Sample {
    entity: Entity,
    pxls: Show<Pixels>,
    lay: Show<LayoutNode>,
    p: Show<Entity>,
    name: Show<String>,
}
fn sample(
    (entity, pxls, lay, p, name): (
        Entity,
        Option<&Pixels>,
        Option<&LayoutNode>,
        Option<&Parent>,
        Option<&Name>,
    ),
) -> Sample {
    Sample {
        entity,
        pxls: show(pxls),
        lay: show(lay),
        p: Show(p.map(Parent::get)),
        name: Show(name.map(|n| n.as_str().to_owned())),
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Show<T>(Option<T>);
impl<T: fmt::Debug> fmt::Debug for Show<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(v) => write!(f, "{v:?}"),
            None => write!(f, "xxxxxxx"),
        }
    }
}
impl<D: DslBundle> DslBundle for LayoutDsl<D> {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        cmds.insert(LayoutNode {
            width: self.width,
            height: self.height,
            flow: self.flow,
        });
        if let Some(px) = self.px {
            cmds.with_children(|cmds| {
                cmds.spawn(Pixels(px));
            });
        }
        self.inner.insert(cmds)
    }
}

#[parse_dsl_impl(
    set_params <D: ParseDsl + fmt::Debug>,
    delegate = inner,
    type_parsers(Rule = from_str),
)]
impl<D: DslBundle + fmt::Debug> LayoutDsl<D> {
    fn empty_px(&mut self, pixels: u16) {
        self.px = Some(pixels);
    }
    fn flow(&mut self, flow: Flow) {
        self.flow = flow;
    }
    fn column(&mut self) {
        self.flow(Flow::Top);
    }
    fn row(&mut self) {
        self.flow(Flow::Left);
    }
    fn rules(&mut self, width: Rule, height: Rule) {
        self.width = width;
        self.height = height;
    }

    // ...

    #[parse_dsl(ignore)]
    #[allow(clippy::needless_pass_by_value)] // false positive
    fn _spawn_ui<M>(&mut self, _ui_bundle: impl Into<M>, _cmds: &mut EntityCommands) -> Entity {
        todo!()
    }
}

fn main() {
    bevy::log::LogPlugin { level: Level::TRACE, ..Default::default() }.build(&mut App::new());
    let mut registry = TypeRegistry::new();
    registry.register::<Flow>();

    let mut world1 = World::new();
    let chirp = r#"
        row("first row", rules(px(10), pct(11))) {
            spawn(rules(pct(20), px(21)), "first child", empty_px 30);
            spawn(empty_px 31, "2");
        }
        column("second element", rules(px(40), pct(41))) {
            spawn(rules(pct(50), px(51)), empty_px 60, "child3");
            spawn(empty_px 61, "so called \"fourth\" child");
        }
"#;
    let handles: Handles = HashMap::new();
    let mut world_chirp = Chirp::new(&mut world1);
    world_chirp.interpret::<LayoutDsl>(&handles, None, &registry, chirp.as_bytes());

    let mut world2 = World::new();
    let mut state = SystemState::<Commands>::new(&mut world2);
    let mut cmds = state.get_mut(&mut world2);
    dsl! { <LayoutDsl> cmds,
        row("first row", rules(px(10), pct(11))) {
            spawn(rules(pct(20), px(21)), "first child", empty_px 30);
            spawn(empty_px 31, "2");
        }
        column("second element", rules(px(40), pct(41))) {
            spawn(rules(pct(50), px(51)), empty_px 60, "child3");
            spawn(empty_px 61, "so called \"fourth\" child");
        }
    };
    state.apply(&mut world2);

    let mut query = world1.query::<(
        Entity,
        Option<&Pixels>,
        Option<&LayoutNode>,
        Option<&Parent>,
        Option<&Name>,
    )>();
    let mut parse_entities = query.iter(&world1).map(sample).collect::<Vec<_>>();
    parse_entities.sort_unstable();

    let mut query = world2.query::<(
        Entity,
        Option<&Pixels>,
        Option<&LayoutNode>,
        Option<&Parent>,
        Option<&Name>,
    )>();
    let mut macro_entities = query.iter(&world2).map(sample).collect::<Vec<_>>();
    macro_entities.sort_unstable();

    assert_eq!(parse_entities, macro_entities);
}
#[test]
fn parse_dsl_macro_identical() {
    main();
}
