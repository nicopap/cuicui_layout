#![doc = include_str!("../README.md")]
#![allow(clippy::needless_pass_by_value)]

use std::time::Duration;

use bevy::log::LogPlugin;
use bevy::{asset::ChangeWatcher, prelude::*};
use cuicui_chirp::Chirp;
use cuicui_layout::LayoutRootCamera;

use animate::button_shift;
use show_menus::SwatchBuilder;
use ui_offset::UiOffset;

/// Animate stuff.
///
/// - Main menu bg police car strobe lights
/// - Main menu bg "camera shift" effect
/// - Highlight of selected entry
mod animate;
/// Methods for mixing bevy colors using hsluv 💞
mod colormix;
/// Extensions to the DSL required for the menu.
mod dsl;
/// Handle Showing & hidding menus & submenus
mod show_menus;
/// Runtime style controls.
///
/// Basically all color, fonts and animations are defined in this module. Since
/// they are defined as a resource, it's perfectly imaginable to load them and
/// update them at runtime.
mod style;
/// React to UI events such as hover, click or ui-nav focus changes.
mod ui_event;
/// Move `bevy_ui` nodes AFTER layouting, for "offset effects".
mod ui_offset;

type BgColor = BackgroundColor;

#[cfg(feature = "advanced_logging")]
fn bevy_log_plugin() -> LogPlugin {
    LogPlugin {
        level: bevy::log::Level::TRACE,
        filter: "\
          cuicui_layout=info,cuicui_layout_bevy_ui=info,\
          cuicui_chirp=debug,\
          gilrs_core=info,gilrs=info,\
          naga=info,wgpu=error,wgpu_hal=error,\
          bevy_app=info,bevy_render::render_resource::pipeline_cache=info,\
          bevy_render::view::window=info,bevy_ecs::world::entity_ref=info"
            .to_string(),
    }
}
#[cfg(not(feature = "advanced_logging"))]
fn bevy_log_plugin() -> LogPlugin {
    default()
}
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy_log_plugin()).set(AssetPlugin {
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                ..default()
            }),
            (style::Plugin, animate::Plugin, dsl::Plugin),
            (ui_offset::Plugin, ui_event::Plugin, show_menus::Plugin),
            cuicui_layout_bevy_ui::Plugin,
            cuicui_chirp::loader::Plugin::new::<dsl::BevypunkDsl>(),
            bevy_ui_navigation::DefaultNavigationPlugins,
            bevy_framepace::FramepacePlugin,
            #[cfg(feature = "inspector")]
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}
#[allow(clippy::needless_pass_by_value)] // false positive
fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    use ui_event::SwatchMarker::Root;

    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));

    // TODO(feat): This is a workaround not having single-chirp-entity &
    // not being able to refer to other chirp files within chirp files.
    // This is so bad, it makes me angry.
    cmds.spawn((Root, SwatchBuilder::new(), NodeBundle::default())).with_children(|cmds| {
        let mut spawn_menu = |file: &str| {
            cmds.spawn(NodeBundle::default()).with_children(|cmds| {
                cmds.spawn((NodeBundle::default(), assets.load::<Chirp, _>(file)));
            });
        };
        spawn_menu("menus/main.chirp");
        spawn_menu("menus/settings.chirp");
    });
}