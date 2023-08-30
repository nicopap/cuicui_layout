use std::{borrow::Cow, fmt, mem};

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_navigation::prelude::{Focusable, MenuBuilder, MenuSetting};
use cuicui_chirp::parse_dsl_impl;
use cuicui_dsl::{DslBundle, EntityCommands};
use cuicui_layout::dsl_functions::{child, pct};
use cuicui_layout_bevy_ui::UiDsl;

use crate::animate::{bloom, main_menu_bg::Animation as BgAnimation};
use crate::show_menus::SwatchBuilder;
use crate::style;
use crate::ui_event::{QuitGame, SwatchMarker, SwatchTarget};
use element::{Element as DslElement, SettingsOption};

/// Elements (hierarchy of entities) used in [`BevypunkDsl`].
///
/// This allows dsl methods such as [`BevypunkDsl::settings_header`] and
/// [`BevypunkDsl::settings_row`].
///
/// Currently, the only way to achieve re-usability in through rust code, but
/// the aim is to completely replace this with the `use` statement.
mod element;

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
    fn set_prioritized(&mut self) {
        *self = Navigation::Focusable(Focusable::new().prioritized());
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
                cmds.insert((focus, Interaction::default()));
            }
            Navigation::None => {}
        }
    }
}

#[derive(Default)]
struct Arbitrary(Vec<Box<dyn FnOnce(&mut EntityCommands)>>);
impl fmt::Debug for Arbitrary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.0.len();
        f.debug_tuple("Arbitrary").field(&format!("[{{closure}}; {n}]")).finish()
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
    animation: Option<BgAnimation>,
    bloom: Option<bloom::Animation>,
    swatch_target: Option<SwatchTarget>,
    swatch_name: Option<SwatchMarker>,
    cancel: bool,
    arbitrary: Arbitrary,
}
#[parse_dsl_impl(delegate = inner)]
impl BevypunkDsl {
    fn named(&mut self, name: &str) {
        self.inner.named(name.to_string());
    }
    fn bloom(&mut self, intensity: f32) {
        self.bloom = Some(bloom::Animation { intensity });
    }
    fn main_menu_item(&mut self) {
        self.element = DslElement::MainMenuItem;
        // Note: instead of repeating (focusable, row, main_margin 10., rules(70%, 1.5*), distrib_start)
        // for each button, we do this here.
        // This is a limitation of cuicui_chirp that may be lifted in the future.
        self.bloom(2.3);
        self.focusable();
        self.row();
        self.main_margin(10.);
        self.rules(pct(60), child(1.5));
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
    // Similarly, we piggyback on reflect deserialization here.
    fn gyro(&mut self, animation: BgAnimation) {
        self.animation = Some(animation);
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
    // crate::show_menu methods
    //

    fn swatch_target(&mut self, swatch: SwatchTarget) {
        self.swatch_target = Some(swatch);
    }

    fn swatch_name(&mut self, swatch: SwatchMarker) {
        self.swatch_name = Some(swatch);
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
        self.cancel = true;
    }
    fn root_menu(&mut self) {
        let menu = self.nav.get_or_init_menu();
        menu.reachable_from = None;
    }
    fn focusable(&mut self) {
        self.nav.set_focusable();
    }
    fn prioritized(&mut self) {
        self.nav.set_prioritized();
    }
    #[parse_dsl(ignore)]
    fn arbitrary(&mut self, bundle: impl Bundle) {
        self.arbitrary.0.push(Box::new(move |cmds| {
            cmds.insert(bundle);
        }));
    }
}
impl DslBundle for BevypunkDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        let name = self.inner.name.clone().unwrap_or(Cow::Owned(String::new()));
        if let Some(swatch_target) = self.swatch_target {
            cmds.insert(swatch_target);
        }
        if let Some(swatch_name) = self.swatch_name {
            cmds.insert((swatch_name, SwatchBuilder::new()));
        }
        if let Some(bloom) = self.bloom.take() {
            cmds.insert(bloom);
        }
        if let Some(animation) = self.animation.take() {
            cmds.insert(animation);
        }
        if let Some(style) = self.style.take() {
            style.insert(cmds);
        }
        if self.cancel {
            cmds.insert(QuitGame);
        }
        for to_add in self.arbitrary.0.drain(..) {
            to_add(cmds);
        }
        self.element.spawn(&name, cmds, self.settings_option.take());
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
