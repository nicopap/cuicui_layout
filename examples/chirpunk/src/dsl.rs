use bevy::prelude::*;
use cuicui_chirp::parse_dsl_impl;
use cuicui_dsl::DslBundle;
use cuicui_layout_bevy_ui::UiDsl;

#[derive(Default)]
enum ElementType {
    #[default]
    Panel,
    MainMenuItemButton,
    TabButton,
    OptionEntry,
}

#[derive(Deref, DerefMut, Default)]
struct BevypunkDsl {
    #[deref]
    inner: UiDsl,
    element: ElementType,
}
#[parse_dsl_impl(delegate = inner)]
impl BevypunkDsl {
    fn named(&mut self, name: &str) {
        self.inner.named(name.to_string());
    }
    fn main_menu_item(&mut self) {
        self.element = ElementType::MainMenuItemButton;
    }
}
impl DslBundle for BevypunkDsl {
    fn insert(&mut self, cmds: &mut cuicui_dsl::EntityCommands) -> Entity {
        self.inner.insert(cmds)
    }
}
