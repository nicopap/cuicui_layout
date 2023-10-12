//! Demonstrates how to define your own DSL.
#![allow(
    clippy::cast_precision_loss,
    clippy::wildcard_imports,
    clippy::needless_pass_by_value
)]

use std::time::Duration;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_mod_picking::prelude::{Click, On, Pointer};
use cuicui_chirp::ChirpBundle;
use cuicui_layout::LayoutRootCamera;

use dsl::BetterFactorioDsl;
use reflect_on_click::ReflectOnClick;

mod dsl;
mod reflect_on_click;

type OnClick = On<Pointer<Click>>;

struct DefaultPlugins;

fn bevy_log_plugin() -> LogPlugin {
    cuicui_examples::log_plugin(cfg!(feature = "advanced_logging"))
}

impl PluginGroup for DefaultPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let primary_window = Some(Window { resolution: (640., 360.).into(), ..default() });
        bevy::prelude::DefaultPlugins
            .set(WindowPlugin { primary_window, ..default() })
            .set(bevy_log_plugin())
        // .add(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
    }
}

fn main() {
    let asset_folder = "../../assets".to_owned();

    App::new()
        .add_plugins((
            DefaultPlugins.set({
                let delay = Duration::from_millis(200);
                let watch_for_changes = bevy::asset::ChangeWatcher::with_delay(delay);
                AssetPlugin { asset_folder, watch_for_changes }
            }),
            cuicui_layout_bevy_ui::Plugin,
            // ANCHOR: add_plugin
            cuicui_chirp::loader::Plugin::new::<BetterFactorioDsl>(),
            // ANCHOR_END: add_plugin
            dsl::DslPlugin,
            cuicui_examples::HighlightPlugin,
            // ANCHOR: mirror_plugin
            bevy_mod_picking::DefaultPickingPlugins,
            cuicui_examples::MirrorPlugin::<OnClick, ReflectOnClick>::new_from(),
            // ANCHOR_END: mirror_plugin
        ))
        .add_systems(Startup, setup)
        .run();
}

#[allow(clippy::needless_pass_by_value)]
fn setup(mut cmds: Commands, serv: Res<AssetServer>) {
    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));

    cmds.spawn(ChirpBundle::new(serv.load("better_factorio/menu.chirp")));
}
