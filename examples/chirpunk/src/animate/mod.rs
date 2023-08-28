//! Animations.
use bevy::prelude::{AnyOf, App, Plugin as BevyPlugin, Text, Update};

use crate::{BgColor, UiOffset};

pub mod button_shift;
pub mod main_menu_bg;

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
            .add_systems(Update, button_shift::animate);
    }
}
