//! Demonstrates how one could build a menu using `cuicui_layout` in
//! combination with `bevy_ui` and `cuicui_chirp`.
#![allow(clippy::cast_precision_loss, clippy::wildcard_imports)]

use std::time::Duration;

use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;
use cuicui_layout::LayoutRootCamera;
use cuicui_layout_bevy_ui::UiDsl;

fn main() {
    let asset_folder = "../../assets".to_owned();

    // ANCHOR: app
    App::new()
        .add_plugins((
            DefaultPlugins.set({
                let delay = Duration::from_millis(200);
                let watch_for_changes = bevy::asset::ChangeWatcher::with_delay(delay);
                AssetPlugin { asset_folder, watch_for_changes }
            }),
            cuicui_layout_bevy_ui::Plugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
            // You still need to add manually the chirp loaded for UiDsl!
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
