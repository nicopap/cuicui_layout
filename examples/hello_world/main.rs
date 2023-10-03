//! The most simple way of using `cuicui_layout`
use bevy::prelude::*;
use cuicui_layout::{dsl, LayoutRootCamera};
use cuicui_layout_bevy_ui::UiDsl as Dsl;

fn main() {
    // Do not forget to add cuicui_layout_bevy_{ui,sprite}::Plugin
    App::new()
        .add_plugins((DefaultPlugins, cuicui_layout_bevy_ui::Plugin))
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands) {
    // Use LayoutRootCamera to mark a camera as the screen boundaries.
    commands.spawn((Camera2dBundle::default(), LayoutRootCamera));

    dsl! { &mut commands.spawn_empty(),
        // Use screen_root to follow the screen's boundaries
        Entity(row screen_root) {
            Entity(row margin(9.) border(5, Color::CYAN) bg(Color::NAVY)) {
                Entity(ui("Hello world!"))
            }
        }
    };
}
