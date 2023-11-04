//! Demonstrates how one could build a menu using `cuicui_layout` in
//! combination with `bevy_ui` and `cuicui_chirp`.
#![allow(clippy::cast_precision_loss, clippy::wildcard_imports)]

use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;
use cuicui_layout::LayoutRootCamera;
use cuicui_layout_bevy_ui::UiDsl;

struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let primary_window = Some(Window { resolution: (640., 360.).into(), ..default() });
        bevy::prelude::DefaultPlugins.set(WindowPlugin { primary_window, ..default() })
        // .add(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
    }
}

fn main() {
    let file_path = "../../assets".to_owned();

    // ANCHOR: app
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin { file_path, ..default() }),
            cuicui_layout_bevy_ui::Plugin,
            // You still need to add manually the asset loader for UiDsl!
            cuicui_chirp::loader::Plugin::new::<UiDsl>(),
        ))
        .add_systems(Startup, setup)
        .run();
    // ANCHOR_END: app
}

#[allow(clippy::needless_pass_by_value)]
// ANCHOR: setup
fn setup(mut cmds: Commands, serv: Res<AssetServer>) {
    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));

    cmds.spawn(ChirpBundle::new(serv.load("chirp_menu.chirp")));
}
// ANCHOR_END: setup
