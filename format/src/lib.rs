#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(
    clippy::use_self,
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate
)]

mod interpret;
pub mod parse;

use std::str;

use bevy::prelude::BuildChildren;
use cuicui_dsl::EntityCommands;
use kdl::KdlDocument;

use interpret::DslInterpret;

pub use interpret::{Handles, InterpError};
pub use parse::{DslError, ParseDsl};

/// Deserialized `dsl!` object.
///
/// Use [`DslFormat::parse`] to create a `DslFormat` from arbitrary byte slices.
///
/// Use [`DslFormat::interpret`] to interpret the `DslFormat` and add it to the
/// world with provided `cmds` as root.
pub struct DslFormat {
    document: KdlDocument,
}
impl DslFormat {
    /// Create a [`DslFormat`] from arbitrary byte slices.
    ///
    /// Currently, UTF-8 encoded KDL is expected.
    pub fn parse(input: &[u8]) -> Result<Self, DslError> {
        let input_utf8 = str::from_utf8(input)?;
        let document = input_utf8.parse()?;
        Ok(DslFormat { document })
    }
    /// Spawns UI according to KDL spec of this, using the `D` [`ParseDsl`].
    pub fn interpret<D: ParseDsl>(
        &self,
        mut cmds: EntityCommands,
        handles: &Handles,
    ) -> Result<(), InterpError> {
        let mut err = Ok(());
        cmds.with_children(|cmds| {
            err = DslInterpret::<D>::new(handles).statements(&self.document, cmds);
        });
        err
    }
}

#[cfg(all(test, feature = "test_and_doc"))]
mod tests {
    use crate::parse::InterpretMethodCtx;

    use super::*;
    use bevy::{ecs::system::SystemState, prelude::*, utils::HashMap};

    use cuicui_dsl::{
        dsl,
        macros::__doc_helpers::{self as dsl, px},
        BaseDsl,
    };

    impl<C: ParseDsl> ParseDsl for dsl::DocDsl<C> {
        fn method(&mut self, data: InterpretMethodCtx) -> Result<(), anyhow::Error> {
            use crate::parse::quick;
            let InterpretMethodCtx { name, args, .. } = data;
            match name {
                "column" => {
                    let () = quick::arg0(args)?;
                    self.column();
                    Ok(())
                }
                "main_margin" => {
                    let arg = quick::arg1(args)?;
                    self.main_margin(arg);
                    Ok(())
                }
                "distrib_start" => {
                    let () = quick::arg0(args)?;
                    self.distrib_start();
                    Ok(())
                }
                "align_start" => {
                    let () = quick::arg0(args)?;
                    self.align_start();
                    Ok(())
                }
                "image" => {
                    // let args = quick::arg0(args)?;
                    self.image(&());
                    Ok(())
                }
                "row" => {
                    let () = quick::arg0(args)?;
                    self.row();
                    Ok(())
                }
                "width" => {
                    let arg = quick::arg1(args)?;
                    self.width(arg);
                    Ok(())
                }
                "height" => {
                    let arg = quick::arg1(args)?;
                    self.height(arg);
                    Ok(())
                }
                "screen_root" => {
                    let () = quick::arg0(args)?;
                    self.screen_root();
                    Ok(())
                }
                "fill_main_axis" => {
                    let () = quick::arg0(args)?;
                    self.fill_main_axis();
                    Ok(())
                }
                "color" => {
                    let arg = quick::arg1(args)?;
                    self.color(arg);
                    Ok(())
                }
                "amplitude" => {
                    let arg = quick::arg1(args)?;
                    self.amplitude(arg);
                    Ok(())
                }
                "frequency" => {
                    let arg = quick::arg1(args)?;
                    self.frequency(arg);
                    Ok(())
                }
                _ => self.inner.method(data),
            }
        }
        fn leaf_node(
            &mut self,
            mut data: parse::InterpretLeafCtx,
        ) -> Result<Entity, anyhow::Error> {
            use crate::parse::quick;
            let parse::InterpretLeafCtx { name, leaf_arg, cmds, .. } = &mut data;
            match *name {
                "button" => {
                    let arg: String = quick::arg1(leaf_arg)?;
                    Ok(self.button(&arg, cmds))
                }
                "spawn_ui" => {
                    let arg: String = quick::arg1(leaf_arg)?;
                    Ok(self.spawn_ui(&arg, cmds))
                }

                _ => self.inner.leaf_node(data),
            }
        }
    }

    const TITLE_CARD_DSL: &str = r#"
row "screen_root" named="root" main_margin="100." "distrib_start" "align_start" image="&bg" {
    column named="menu" width="px(310)" height="pct(100)" main_margin="40." image="&board" {
        spawn image="&title_card" named="Title card" width="pct(100)" ;
        spawn_ui "title_card" named="Title card 2" width="pct(50)" ;
        code "spawn_menu_buttons";
    }
}
"#;
    #[test]
    fn parse_main_menu() {
        let menu_buttons = [
            "CONTINUE",
            "NEW GAME",
            "LOAD GAME",
            "SETTINGS",
            "ADDITIONAL CONTENT",
            "CREDITS",
            "QUIT GAME",
        ];
        let mut world = World::new();
        let mut state = SystemState::<Commands>::new(&mut world);
        let mut cmds = state.get_mut(&mut world);
        let cmds = cmds.spawn_empty();
        let parsed = DslFormat::parse(TITLE_CARD_DSL.as_bytes()).unwrap();
        let spawn_menu_buttons = |cmds: &mut ChildBuilder| {
            let button = ();
            for n in &menu_buttons {
                let name = format!("{n} button");
                dsl!(<dsl::DocDsl> cmds, spawn_ui(*n, named name, image &button, height px(33)););
            }
        };
        let mut handles: Handles = HashMap::new();
        handles.insert("spawn_menu_buttons".to_owned(), &spawn_menu_buttons);
        parsed
            .interpret::<dsl::DocDsl<BaseDsl>>(cmds, &handles)
            .unwrap();
        state.apply(&mut world);
    }
}
