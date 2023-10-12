use std::{borrow::Cow, fmt, mem};

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_navigation::prelude::{Focusable, MenuBuilder, MenuSetting};
use cuicui_chirp::parse_dsl_impl;
use cuicui_dsl::{DslBundle, EntityCommands};
use cuicui_layout_bevy_ui::UiDsl;

use crate::animate::{bloom, main_menu_bg::Animation as BgAnimation};
use crate::style;
use crate::ui_event::{QuitGame, RootButton, TabButton, Tabs};
use element::{Element as DslElement, SettingsOption};

/// Elements (hierarchy of entities) used in [`BevypunkDsl`].
///
/// This allows dsl methods such as [`BevypunkDsl::settings_row`].
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

#[derive(Debug, Clone, Copy, Reflect)]
enum SwitchTarget {
    Roots,
    Tabs,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Deref, DerefMut, Default, Debug)]
pub struct BevypunkDsl {
    #[deref]
    inner: Box<UiDsl>,
    element: DslElement,
    style: Option<style::Element>,
    settings_option: Option<Box<SettingsOption>>,
    nav: Navigation,
    animation: Option<BgAnimation>,
    bloom: Option<bloom::Animation>,
    switch_index: Option<(u8, SwitchTarget)>,
    is_cancel: bool,
    is_settings_tabs: bool,
    is_hidden: bool,
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
    fn settings_row(&mut self) {
        self.element = DslElement::SettingsRow;
        self.focusable();
    }
    // Note that `parse_dsl_impl` automatically uses the RON deserializer
    // for `SettingsOption`, based on `Reflect`.
    fn options(&mut self, options: SettingsOption) {
        self.settings_option = Some(Box::new(options));
    }
    // Similarly, we piggyback on reflect deserialization here.
    // We could accept a `BgAnimation` instead of the three `f64` but templates
    // currently only supports parameter inlining when passed as "full" arguments.
    fn gyro(&mut self, offset: f64, period: f64, active_period: f64) {
        self.animation = Some(BgAnimation { offset, period, active_period });
    }

    fn style(&mut self, style: style::Element) {
        self.style = Some(style);
    }

    //
    // crate::show_menu methods
    //

    fn swatch_target(&mut self, index: u8, swatch: SwitchTarget) {
        self.switch_index = Some((index, swatch));
    }
    fn settings_tabs(&mut self) {
        self.is_settings_tabs = true;
    }
    fn hidden(&mut self) {
        self.is_hidden = true;
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
        self.is_cancel = true;
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
    #[allow(unused)]
    fn arbitrary(&mut self, bundle: impl Bundle) {
        self.arbitrary.0.push(Box::new(move |cmds| {
            cmds.insert(bundle);
        }));
    }
}
impl DslBundle for BevypunkDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) {
        let name = self.inner.name.clone().unwrap_or(Cow::Owned(String::new()));
        match self.switch_index {
            Some((i, SwitchTarget::Roots)) => cmds.insert(RootButton(i)),
            Some((i, SwitchTarget::Tabs)) => cmds.insert(TabButton(i)),
            None => cmds,
        };
        if self.is_settings_tabs {
            cmds.insert(Tabs);
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
        if self.is_cancel {
            cmds.insert(QuitGame);
        }
        for to_add in self.arbitrary.0.drain(..) {
            to_add(cmds);
        }
        self.element.spawn(&name, cmds, self.settings_option.take());
        self.nav.spawn(cmds);
        self.inner.insert(cmds);
        if self.is_hidden {
            cmds.insert(Visibility::Hidden);
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        // Required in order to parse the `options` method argument
        app.register_type::<SettingsOption>()
            .register_type::<Vec<String>>()
            .register_type::<SwitchTarget>();
    }
}
