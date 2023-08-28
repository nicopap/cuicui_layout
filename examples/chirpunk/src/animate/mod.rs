//! Animations.
use bevy::prelude::{AnyOf, App, Plugin as BevyPlugin, Text, Update};

use crate::{BgColor, UiOffset};

pub mod button_shift;
pub mod main_menu_bg;
// TODO(bug): broken because we are using bevy_ui
pub mod bloom;

type AnimatedComponents = AnyOf<(
    &'static mut UiOffset,
    &'static mut Text,
    &'static mut BgColor,
)>;

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<button_shift::State>()
            .register_type::<button_shift::Animation>()
            .register_type::<main_menu_bg::Animation>()
            .register_type::<bloom::Animation>()
            .add_systems(
                Update,
                (button_shift::animate, bloom::animate, main_menu_bg::animate),
            );
    }
}
