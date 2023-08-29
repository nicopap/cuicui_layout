use bevy::prelude::*;
use cuicui_dsl::{dsl, EntityCommands};
use cuicui_layout::dsl_functions::{child, pct};

use super::BevypunkDsl;
use crate::style;

#[derive(Default, Debug)]
pub enum Element {
    #[default]
    None,
    TabButton,
    SettingsHeader,
    SettingsRow,
    MainMenuItem,
}
impl Element {
    pub fn spawn(&self, name: &str, cmds: &mut EntityCommands) {
        match self {
            Element::None => {}
            Element::TabButton => tab_button(name, cmds),
            Element::SettingsHeader => settings_header(name, cmds),
            Element::SettingsRow => settings_row(name, cmds),
            Element::MainMenuItem => main_menu_item(name, cmds),
        }
    }
}
fn tab_button(name: &str, cmds: &mut EntityCommands) {
    dsl! { @entity <BevypunkDsl> cmds,
        spawn(named name, focusable, rules(child(2.), child(2.))) {
            spawn(text name, style style::Element::TabButton);
        }
    };
}
fn settings_header(name: &str, cmds: &mut EntityCommands) {
    dsl! { @entity <BevypunkDsl> cmds,
        spawn(
            named name,
            main_margin 10.,
            width pct(90),
            style style::Element::SettingsHeader,
            row,
        ) {
            spawn(text name, style style::Element::SettingsHeaderText);
        }
    };
}
fn settings_row(name: &str, cmds: &mut EntityCommands) {
    dsl! { @entity <BevypunkDsl> cmds,
        spawn(
            named name,
            main_margin 10.,
            width pct(90),
            row,
        ) {
            spawn(text name, style style::Element::OptionEntry);
        }
    };
}
fn main_menu_item(name: &str, cmds: &mut EntityCommands) {
    dsl! { @entity <BevypunkDsl> cmds,
        spawn(
            named name,
            style style::Element::MainMenuItemButton,
            image &Handle::default(),
        ) {
            spawn(text name, style style::Element::MainMenuItemText);
        }
    };
}
