//! Demonstrates how one could build a menu using `cuicui_layout` in
//! combination with `bevy_ui`.
#![allow(clippy::cast_precision_loss, clippy::wildcard_imports)]

// ANCHOR: imports
use bevy::prelude::*;
use cuicui_dsl::dsl;
use cuicui_layout::{dsl_functions::*, LayoutRootCamera};
use cuicui_layout_bevy_ui::UiDsl as Dsl;
// ANCHOR_END: imports

struct DefaultPlugins;
impl PluginGroup for DefaultPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        DefaultPlugins.set(AssetPlugin {
            asset_folder: "../../assets".to_owned(),
            ..default()
        })
    }
}

// ANCHOR: main
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Notice that we add the plugin here.
            cuicui_layout_bevy_ui::Plugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}
// ANCHOR_END: main

#[allow(clippy::needless_pass_by_value)]
// ANCHOR: setup_signature
fn setup(mut cmds: Commands, serv: Res<AssetServer>) {
    // ANCHOR_END: setup_signature
    // ANCHOR: setup_camera
    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));
    // ANCHOR_END: setup_camera
    // ANCHOR: dsl
    let menu_buttons = [
        "CONTINUE",
        "NEW GAME",
        "LOAD GAME",
        "SETTINGS",
        "ADDITIONAL CONTENT",
        "CREDITS",
        "QUIT GAME",
    ];
    let text = |value| Text::from_section(value, TextStyle { font_size: 30.0, ..default() });
    // ANCHOR: dsl_start
    let title_card = serv.load("logo.png");
    let bg = serv.load("background.jpg");
    let board = serv.load("board.png");
    let button = serv.load("button.png");

    dsl! {
        &mut cmds.spawn_empty(),
        Root(layout(">dSaS") screen_root main_margin(100.) image(&bg)) {
            Menu(rules(px(310), pct(100)) main_margin(40.) image(&board) column) {
                TitleCard(image(&title_card) width(pct(100)))
                TitleCard2(ui(title_card) width(pct(50)))
    // ANCHOR_END: dsl_start
                code(let cmds) {
                    dsl!(cmds, Buttons(column height(child(2.)) width(pct(100))));
                    cmds.with_children(|cmds|{
                        for n in &menu_buttons {
                            let name = format!("{n} button");
                            dsl!(
                                &mut cmds.spawn_empty(),
                                Entity(ui(text(*n)) named(name) image(&button) height(px(33)))
                            );
                        }
                    });
                }
            }
        }
    };
    // ANCHOR_END: dsl
}
