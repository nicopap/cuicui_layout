use std::{fmt, num::ParseIntError, str::FromStr};

use bevy::{
    ecs::{prelude::*, system::SystemState},
    prelude::{BuildChildren, Deref, DerefMut, Parent},
    utils::HashMap,
};
use cuicui_chirp::{parse_dsl_impl, Chirp, Handles, ParseDsl};
use cuicui_dsl::{dsl, BaseDsl, DslBundle, EntityCommands};
use pretty_assertions::assert_eq;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Flow {
    #[default]
    Top,
    Left,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Rule {
    Pct(i32),
    Px(i32),
    #[default]
    None,
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
                Ok(Rule::Px(number.parse()?))
            }
            () if s.starts_with("pct(") => {
                let number = &s[4..s.len() - 1];
                Ok(Rule::Pct(number.parse()?))
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
}

#[derive(Debug, Clone, Component, PartialEq, Eq, PartialOrd, Ord)]
struct LayoutNode {
    width: Rule,
    height: Rule,
    flow: Flow,
}
#[derive(Debug, Clone, Component, PartialEq, Eq, PartialOrd, Ord)]
struct Pixels(u16);

impl<D: DslBundle> DslBundle for LayoutDsl<D> {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        cmds.insert(LayoutNode {
            width: self.width,
            height: self.height,
            flow: self.flow,
        });
        self.inner.insert(cmds)
    }
}

#[parse_dsl_impl(set_params <D: ParseDsl + fmt::Debug>, delegate = inner)]
impl<D: DslBundle + fmt::Debug> LayoutDsl<D> {
    #[parse_dsl(ignore)]
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

    #[parse_dsl(leaf_node)]
    fn empty_px(&mut self, pixels: u16, cmds: &mut EntityCommands) -> Entity {
        println!("{self:?}");
        cmds.with_children(|cmds| {
            cmds.spawn(Pixels(pixels));
        });
        cmds.id()
    }
    #[parse_dsl(ignore)]
    #[allow(clippy::needless_pass_by_value)] // false positive
    fn _spawn_ui<M>(&mut self, _ui_bundle: impl Into<M>, _cmds: &mut EntityCommands) -> Entity {
        todo!()
    }
}

fn main() {
    let mut world1 = World::new();
    let mut state = SystemState::<Commands>::new(&mut world1);
    let mut cmds = state.get_mut(&mut world1);
    let chirp = r#"
row rules="(px(10), pct(11))" {
    empty_px 30 rules="(pct(20), px(21))";
    empty_px 31;
}
column rules="(px(40), pct(41))" {
    empty_px 60 rules="(pct(50), px(51))";
    empty_px 61;
}"#;
    let parsed = Chirp::parse(chirp.as_bytes()).unwrap();
    let handles: Handles = HashMap::new();
    let cmds = cmds.spawn_empty();
    parsed.interpret::<LayoutDsl>(cmds, &handles).unwrap();
    state.apply(&mut world1);

    let mut world2 = World::new();
    let mut state = SystemState::<Commands>::new(&mut world2);
    let mut cmds = state.get_mut(&mut world2);
    cmds.spawn_empty().with_children(|cmds| {
        dsl! { <LayoutDsl> cmds,
            row(rules(px(10), pct(11))) {
                empty_px(30, rules(pct(20), px(21)));
                empty_px(31);
            }
            column(rules(px(40), pct(41))) {
                empty_px(60, rules(pct(50), px(51)));
                empty_px(61);
            }
        };
    });
    state.apply(&mut world2);

    let get = Parent::get;
    let mut query = world1.query::<(Option<&Pixels>, Option<&LayoutNode>, Option<&Parent>)>();
    let mut w1_entities = query
        .iter(&world1)
        .map(|(a, b, c)| (a.cloned(), b.cloned(), c.map(get)))
        .collect::<Vec<_>>();
    w1_entities.sort_unstable();

    let mut query = world2.query::<(Option<&Pixels>, Option<&LayoutNode>, Option<&Parent>)>();
    let mut w2_entities = query
        .iter(&world2)
        .map(|(a, b, c)| (a.cloned(), b.cloned(), c.map(get)))
        .collect::<Vec<_>>();
    w2_entities.sort_unstable();

    assert_eq!(w1_entities, w2_entities);
}
#[test]
fn parse_dsl_macro_identical() {
    main();
}
