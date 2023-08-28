use std::{borrow::Cow, mem};

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_navigation::prelude::{Focusable, MenuBuilder, MenuSetting};
use cuicui_chirp::parse_dsl_impl;
use cuicui_dsl::{DslBundle, EntityCommands};
use cuicui_layout::dsl_functions::{child, pct};
use cuicui_layout_bevy_ui::UiDsl;

use crate::style;
use element::Element as DslElement;

/// Elements (hierarchy of entities) used in [`BevypunkDsl`].
///
/// This allows dsl methods such as [`BevypunkDsl::settings_header`] and
/// [`BevypunkDsl::settings_row`].
///
/// Currently, the only way to achieve re-usability in through rust code, but
/// the aim is to completely replace this with the `use` statement.
mod element;

#[derive(Reflect, Debug)]
enum SettingsOption {
    Choice(Vec<String>),
    Toggle,
    Click,
    Increments(usize),
}

#[derive(Default, Debug)]
struct MenuData {
    setting: MenuSetting,
    reachable_from: Option<Box<str>>,
}

#[derive(Default, Debug)]
enum Navigation {
    Menu(MenuData),
    Focusable(Focusable),
    #[default]
    None,
}
impl Navigation {
    fn get_or_init_menu(&mut self) -> &mut MenuData {
        use Navigation::{Focusable, Menu};
        match self {
            Menu(menu) => menu,
            Focusable(_) | Navigation::None => {
                *self = Navigation::Menu(MenuData::default());
                if let Navigation::Menu(menu) = self {
                    menu
                } else {
                    unreachable!("We just set self to Menu")
                }
            }
        }
    }
    fn set_cancel(&mut self) {
        *self = Navigation::Focusable(Focusable::cancel());
    }
    fn set_focusable(&mut self) {
        *self = Navigation::Focusable(Focusable::new());
    }
    fn spawn(&mut self, cmds: &mut EntityCommands) {
        use MenuBuilder::{NamedParent, Root};

        let this = mem::take(self);
        match this {
            Navigation::Menu(data) => {
                let builder =
                    data.reachable_from.map_or(Root, |n| NamedParent(Name::new(n.into_string())));
                cmds.insert((data.setting, builder));
            }
            Navigation::Focusable(focus) => {
                cmds.insert(focus);
            }
            Navigation::None => {}
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Deref, DerefMut, Default, Debug)]
pub struct BevypunkDsl {
    #[deref]
    inner: UiDsl,
    element: DslElement,
    style: Option<style::Element>,
    settings_option: Option<Box<SettingsOption>>,
    nav: Navigation,
}
#[parse_dsl_impl(delegate = inner)]
impl BevypunkDsl {
    fn named(&mut self, name: &str) {
        self.inner.named(name.to_string());
    }
    fn main_menu_item(&mut self) {
        self.element = DslElement::MainMenuItem;
        // Note: instead of repeating (focusable, row, main_margin 10., rules(70%, 1.5*), distrib_start)
        // for each button, we do this here.
        // This is a limitation of cuicui_chirp that may be lifted in the future.
        self.focusable();
        self.row();
        self.main_margin(10.);
        self.rules(pct(70), child(1.5));
        self.distrib_start();
    }
    fn tab_button(&mut self) {
        self.element = DslElement::TabButton;
        self.focusable();
    }
    fn settings_row(&mut self) {
        self.element = DslElement::SettingsRow;
        self.focusable();
    }
    fn settings_header(&mut self) {
        self.element = DslElement::SettingsHeader;
    }
    // Note that `parse_dsl_impl` automatically uses the RON deserializer
    // for `SettingsOption`, based on `Reflect`.
    fn options(&mut self, options: SettingsOption) {
        self.settings_option = Some(Box::new(options));
    }

    fn style(&mut self, style: style::Element) {
        self.style = Some(style);
    }

    /// Set a node to fill entirely its parent. Useful for the main menu
    /// animation effects.
    fn full_screen(&mut self) {
        self.layout(">dSaS");
        self.rules(pct(100), pct(100));
    }

    //
    // bevy-ui-navigation methods
    //

    // bevy-ui-navigation provides a DSL, but since we depend on development cuicui_layout,
    // we can't depend on what bevy-ui-navigation provides

    fn menu(&mut self, reachable_from: &str) {
        let menu = self.nav.get_or_init_menu();
        menu.reachable_from = Some(reachable_from.into());
    }
    fn scope(&mut self) {
        let menu = self.nav.get_or_init_menu();
        menu.setting.scope = true;
    }
    fn wrap(&mut self) {
        let menu = self.nav.get_or_init_menu();
        menu.setting.wrapping = true;
    }
    fn cancel(&mut self) {
        self.nav.set_cancel();
    }
    fn root_menu(&mut self) {
        let menu = self.nav.get_or_init_menu();
        menu.reachable_from = None;
    }
    fn focusable(&mut self) {
        self.nav.set_focusable();
    }
}
impl DslBundle for BevypunkDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        let name = self.inner.name.clone().unwrap_or(Cow::Owned(String::new()));
        if let Some(style) = self.style.take() {
            style.insert(cmds);
        }
        self.element.spawn(&name, cmds);
        self.nav.spawn(cmds);
        self.inner.insert(cmds)
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        // Required in order to parse the `options` method argument
        app.register_type::<SettingsOption>().register_type::<Vec<String>>();
    }
}
