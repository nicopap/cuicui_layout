//! Demonstrates how one could build a menu using `cuicui_layout` in
//! combination with `bevy_ui`.
#![allow(clippy::cast_precision_loss, clippy::wildcard_imports)]
use std::time::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use cuicui_dsl::dsl;
use cuicui_layout::{dsl_functions::*, LayoutRootCamera};
use cuicui_layout_bevy_ui::UiDsl as Dsl;

macro_rules! text {
    ($value:expr) => {
        Text::from_section($value, TextStyle {
            font_size: 30.0,
            ..default()
        })
    };
    ($($tail:tt)*) => {
        Text::from_section(format!($($tail)*), TextStyle {
            font_size: 30.0,
            ..default()
        })
    };
}

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
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

#[allow(clippy::needless_pass_by_value)]
fn setup(mut cmds: Commands, serv: Res<AssetServer>) {
    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));
    let title_card = serv.load::<Image, _>("logo.png");
    let menu_buttons = [
        "CONTINUE",
        "NEW GAME",
        "LOAD GAME",
        "SETTINGS",
        "ADDITIONAL CONTENT",
        "CREDITS",
        "QUIT GAME",
    ];
    let bg = serv.load("background.jpg");
    let board = serv.load("board.png");
    let button = serv.load("button.png");

    dsl! {
        &mut cmds,
        spawn(layout ">dSaS", screen_root, "root", main_margin 100., image &bg) {
            column("menu", rules(px(310), pct(100)), main_margin 40., image &board) {
                spawn(image &title_card, "Title card", width pct(100));
                spawn (ui title_card, "Title card 2", width pct(50));
                code(let cmds) {
                    for n in &menu_buttons {
                        let name = format!("{n} button");
                        dsl!(cmds, spawn (ui text!(*n), named name, image &button, height px(33)););
                    }
                }
            }
        }
    };
}
