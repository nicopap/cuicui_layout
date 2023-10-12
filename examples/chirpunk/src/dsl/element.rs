use std::borrow::Cow;

use bevy::prelude::*;
use cuicui_dsl::{dsl, EntityCommands};
use cuicui_layout::dsl_functions::{child, pct, px};

use super::BevypunkDsl;
use crate::style;

/// A button that can cycle through multiple options.
///
/// Currently, the only way to achieve re-usability in through rust code, but
/// the aim is to completely replace this with the `fn` statement.
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
            Self::Choice(elems) => elems[0].clone().into(),
            Self::Toggle => "Enabled".into(),
            Self::Click => "Click".into(),
            Self::Increments(_) => "0".into(),
        }
    }
    fn choices(&self) -> usize {
        match self {
            Self::Choice(elems) => elems.len(),
            Self::Toggle => 2,
            Self::Click => 0,
            Self::Increments(count) => *count,
        }
    }
}

#[derive(Default, Debug)]
pub enum Element {
    #[default]
    None,
    SettingsRow,
}
impl Element {
    pub(super) fn spawn(
        &self,
        name: &str,
        cmds: &mut EntityCommands,
        options: Option<Box<SettingsOption>>,
    ) {
        let settings_err = "settings_row element MUST also have a `options` method call included, \
                            none were given.";
        match self {
            Self::None => {}
            Self::SettingsRow => settings_row(name, cmds, *options.expect(settings_err)),
        }
    }
}
fn box_mark(size: u16, cmds: &mut EntityCommands) {
    dsl! { <BevypunkDsl> cmds,
        Entity(rules(px(size), px(3)) style(style::Element::OptionTick)) {}
    };
}
fn settings_row(name: &str, cmds: &mut EntityCommands, options: SettingsOption) {
    use style::Element::{
        OptionBox, OptionBoxChoice, OptionBoxLArrow, OptionBoxRArrow, OptionEntry, OptionRow,
    };
    let default_choice_text = options.default_text();
    let choice_count = options.choices();

    dsl! { <BevypunkDsl> cmds,
        SettingsRow(rules(pct(100), child(1.)) row style(OptionRow)) {
            SettingsText(text(name) style(OptionEntry) width(pct(50)))
            SettingsBox(row rules(pct(45), child(1.5)) style(OptionBox) main_margin(10.)) {
                LArrow(style(OptionBoxLArrow) height(px(25)))
                BoxContent(column rules(child(1.), child(1.2))) {
                    BoxSelectedText(style(OptionBoxChoice) text(&default_choice_text))
                    code(let cmds) {
                        let mut dsl = BevypunkDsl::default();
                        dsl.named("BoxTicks");
                        dsl.row();
                        dsl.rules(child(1.3), child(1.));
                        dsl.node(cmds, |cmds| {
                            for _ in 0..choice_count {
                                let max_size = u16::try_from(350 / choice_count).unwrap();
                                let size = 20_u16.min(max_size);
                                box_mark(size, &mut cmds.spawn_empty());
                            }
                        });
                    }
                }
                RArrow(style(OptionBoxRArrow) height(px(25)))
            }
        }
    };
}
