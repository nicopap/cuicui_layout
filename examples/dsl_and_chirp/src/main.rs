//! Demonstrates the equivalence between the `dsl!` macro and the `.chirp` file
//! format.
//!
//! Also used as a test to make sure it is trully equivalent.
use std::{fmt, num::ParseIntError, str::FromStr};

use bevy::app::{App, Plugin};
use bevy::ecs::{prelude::*, system::SystemState};
use bevy::log::Level;
use bevy::prelude::{BuildChildren, Deref, DerefMut, Parent};
use bevy::reflect::{Reflect, TypeRegistryInternal as TypeRegistry};
use cuicui_chirp::{parse_dsl_impl, ChirpReader, Handles, ParseDsl};
use cuicui_dsl::{dsl, BaseDsl, DslBundle, EntityCommands, Name};
use pretty_assertions::assert_eq;

/* Additional syntax to test
// ----- Invalid Syntax -----
// 1. Named entity without either method or children
EntityName
// 2. Literal-style named entity without either method or children
"Entity Name"
// ----- Valid Syntax -----
// 1. Without methods, but children
EntityName {  }
// 2. With methods only
EntityName ()
// 3. single entity is valid
Entity
*/

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
            Self::Pct(v) => write!(f, "{v}pct"),
            Self::Px(v) => write!(f, "{v}px"),
            Self::None => write!(f, "none"),
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
                Ok(Self::Px(number.parse()?))
            }
            () if s.starts_with("pct(") => {
                let number = &s[4..s.len() - 1];
                Ok(Self::Pct(number.parse()?))
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
    fn insert(&mut self, cmds: &mut EntityCommands) {
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
        self.inner.insert(cmds);
    }
}

#[parse_dsl_impl(
    set_params <D: ParseDsl + fmt::Debug>,
    delegate = inner,
    type_parsers(Rule = args::from_str),
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
}
fn inner_children(cmds: &mut EntityCommands) {
    let menu_buttons = ["CONTINUE", "NEW GAME"];
    cmds.with_children(|cmds| {
        for (i, name) in menu_buttons.iter().enumerate() {
            let pixels = u16::try_from(i).unwrap();
            cmds.spawn((Name::new(format!("{name} inner")), Pixels(pixels)));
        }
    });
}
fn outer_children(cmds: &mut EntityCommands) {
    let menu_buttons = ["CONTINUE", "NEW GAME"];
    cmds.with_children(|cmds| {
        for (i, name) in menu_buttons.iter().enumerate() {
            let pixels = u16::try_from(i).unwrap();
            cmds.spawn((Name::new(format!("{name} outer")), Pixels(pixels + 70)));
        }
    });
}
fn main() {
    bevy::log::LogPlugin { level: Level::TRACE, ..Default::default() }.build(&mut App::new());
    let mut registry = TypeRegistry::new();
    registry.register::<Flow>();

    let mut world1 = World::new();
    let chirp = r#"
        // Some comments
        RootEntity(column) {
            "first row"(
                // demonstrating
                rules(px(10), pct(11))
                row
            ) { // that it is possible
                code(inner_children)
                FirstChild(rules(pct(20), px(21)) empty_px(30)) // to
                code(inner_children)
                2(empty_px(31)) // add comments
                code(inner_children)
            }
            code(outer_children)
            // To a chirp file
            "second element"(rules(px(40),pct(41)) column) {
                child3(rules(pct(50), px(51)) empty_px(60))
                "so called \"fourth\" child"(empty_px(61))
            }
        }
"#;
    let mut handles: Handles = Handles::new();
    handles.add_function("inner_children", |_, _, cmds| inner_children(cmds));
    handles.add_function("outer_children", |_, _, cmds| outer_children(cmds));

    let mut world_chirp = ChirpReader::new(&mut world1);
    assert!(world_chirp.interpret_logging::<LayoutDsl>(
        &handles,
        None,
        &registry,
        chirp.as_bytes()
    ));

    let mut world2 = World::new();
    let mut state = SystemState::<Commands>::new(&mut world2);
    let mut cmds = state.get_mut(&mut world2);
    dsl! { <LayoutDsl> &mut cmds.spawn_empty(),
        // Some comments
        RootEntity(column) {
            "first row"(
                // demonstrating
                rules(px(10), pct(11))
                row
            ) { // that it is possible
                code(let cmds) {
                    inner_children(cmds);
                }
                FirstChild(rules(pct(20), px(21)) empty_px(30)) // to
                code(let cmds) {
                    inner_children(cmds);
                }
                2(empty_px(31)) // add comments
                code(let cmds) {
                    inner_children(cmds);
                }
            }
            code(let cmds) {
                outer_children(cmds);
            }
            // To a chirp file
            "second element"(rules(px(40), pct(41)) column) {
                child3(rules(pct(50), px(51)) empty_px(60))
                "so called \"fourth\" child"(empty_px(61))
            }
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
