use std::{fmt, time::Duration};

use bevy::{asset::ChangeWatcher, prelude::*};
use cuicui_chirp::Chirp;
use cuicui_layout::LayoutRootCamera;
use cuicui_layout_bevy_ui::UiDsl;

impl fmt::Debug for Pixels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} pixels", self.0)
    }
}

#[derive(Clone, Component, PartialEq, Eq, PartialOrd, Ord)]
struct Pixels(u16);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    asset_folder: "../../assets".to_owned(),
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::TRACE,
                    filter: "\
                      cuicui_layout=info,\
                      cuicui_layout_bevy_ui=info,\
                      gilrs_core=info,\
                      gilrs=info,\
                      naga=info,\
                      wgpu=error,\
                      wgpu_hal=error\
                      "
                    .to_string(),
                }),
            cuicui_layout_bevy_ui::Plugin,
            cuicui_chirp::loader::Plugin::new::<UiDsl>(),
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}
#[allow(clippy::needless_pass_by_value)] // false positive
fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));
    cmds.spawn(assets.load::<Chirp, _>("trivial.chirp"));
}
