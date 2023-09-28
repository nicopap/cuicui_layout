//! Demonstates how to use templates.
use std::time::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use cuicui_chirp::ChirpBundle;
use cuicui_layout::LayoutRootCamera;
use cuicui_layout_bevy_ui::UiDsl;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    asset_folder: "../../assets".to_owned(),
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "\
                    cuicui_chirp=trace,\
                    cuicui_layout_bevy_ui=info,\
                    gilrs_core=info,\
                    gilrs=info,\
                    naga=info,\
                    wgpu=error,\
                    wgpu_hal=error\
                    "
                    .to_string(),
                }),
            cuicui_chirp::loader::Plugin::new::<UiDsl>(),
            cuicui_layout_bevy_ui::Plugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

#[allow(clippy::needless_pass_by_value)]
fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));
    cmds.spawn(ChirpBundle::from(assets.load("templates.chirp")));
}
