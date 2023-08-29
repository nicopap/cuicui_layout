use std::borrow::Cow;

use bevy::prelude::*;
use cuicui_dsl::{dsl, EntityCommands};
use cuicui_layout::dsl_functions::child;

use super::BevypunkDsl;
use crate::style;

#[derive(Reflect, Debug)]
pub(super) enum SettingsOption {
    Choice(Vec<String>),
    Toggle,
    Click,
    Increments(usize),
}
impl SettingsOption {
    fn default_text(&self) -> Cow<'static, str> {
        match self {
            SettingsOption::Choice(elems) => elems[0].clone().into(),
            SettingsOption::Toggle => "Enabled".into(),
            SettingsOption::Click => "Click".into(),
            SettingsOption::Increments(_) => "0".into(),
        }
    }
    fn choices(&self) -> usize {
        match self {
            SettingsOption::Choice(elems) => elems.len(),
            SettingsOption::Toggle => 2,
            SettingsOption::Click => 1,
            SettingsOption::Increments(count) => *count,
        }
    }
}

fn spawn(options: SettingsOption, cmds: &mut EntityCommands) {
    let default_choice_text = options.default_text();
    let choice_count = options.choices();

    dsl! {
        @entity <BevypunkDsl> cmds,
        spawn(rules(child(1.), child(1.)), row) {
            spawn(style style::Element::OptionBoxLArrow, focusable);
            column(rules(child(1.), child(1.))) {
                spawn(style style::Element::OptionBoxChoice, text &default_choice_text);
                row(rules(child(1.), child(1.))) {
                    code(let cmds) {
                        for _ in 0..choice_count {
                            let mut dsl = BevypunkDsl::default();
                            // dsl.box_mark();
                            dsl.insert(&mut cmds.spawn_empty());
                        }
                    }
                }
            }
            spawn(style style::Element::OptionBoxRArrow, focusable);
        }
    };
}
