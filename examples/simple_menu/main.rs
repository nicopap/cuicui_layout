//! Demonstrates how one could build a menu using `cuicui_layout` in
//! combination with `bevy_ui`.
#![allow(clippy::cast_precision_loss, clippy::wildcard_imports)]

// ANCHOR: imports
use bevy::prelude::*;
use cuicui_dsl::{dsl, EntityCommands};
use cuicui_layout::{dsl_functions::*, LayoutRootCamera};
use cuicui_layout_bevy_ui::UiDsl;
// ANCHOR_END: imports

struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let primary_window = Some(Window { resolution: (640., 360.).into(), ..default() });
        let asset_folder = "../../assets".to_owned();
        bevy::prelude::DefaultPlugins
            .set(WindowPlugin { primary_window, ..default() })
            .set(AssetPlugin { asset_folder, ..default() })
            .add(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
    }
}

// ANCHOR: main
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Notice that we add the plugin here.
            cuicui_layout_bevy_ui::Plugin,
        ))
        .add_systems(Startup, setup)
        .run();
}
// ANCHOR_END: main

// ANCHOR: button_fn
fn button(cmds: &mut EntityCommands, button_bg: &Handle<Image>, button_text: &'static str) {
    dsl! {
        <UiDsl> cmds,
        Entity(text(button_text) named(button_text) image(button_bg) width(pct(80)))
    }
}
// ANCHOR_END: button_fn

#[allow(clippy::needless_pass_by_value)]
// ANCHOR: setup_signature
fn setup(mut cmds: Commands, serv: Res<AssetServer>) {
    // ANCHOR_END: setup_signature
    // ANCHOR: setup_camera
    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));
    // ANCHOR_END: setup_camera
    // ANCHOR: dsl
    // ANCHOR: menu_buttons_let
    let menu_buttons = [
        "CONTINUE",
        "NEW GAME",
        "LOAD GAME",
        "SETTINGS",
        "ADDITIONAL CONTENT",
        "CREDITS",
        "QUIT GAME",
    ];
    // ANCHOR_END: menu_buttons_let
    let button_bg = serv.load("button.png");
    // ANCHOR: dsl_handles
    let title_card = serv.load("logo.png");
    let bg = serv.load("background.jpg");
    let board = serv.load("board.png");
    // ANCHOR_END: dsl_handles

    // ANCHOR: dsl_start
    dsl! {
        <UiDsl>
        &mut cmds.spawn_empty(),
        // ANCHOR_END: dsl_start
        Root(screen_root row distrib_start main_margin(50.) image(&bg)) {
            Column(image(&board) rules(px(150), pct(100)) main_margin(10.) column) {
                TitleCard(width(pct(100)) image(&title_card))
                TitleCard2(width(pct(50)) ui(title_card))
                // ANCHOR: code_container
                code(let cmds) {
                    dsl! { <UiDsl> cmds,
                        ButtonContainer(column rules(pct(100), pct(60)))
                    };
                    cmds.with_children(|cmds| {
                        for text in menu_buttons {
                            button(&mut cmds.spawn_empty(), &button_bg, text);
                        }
                    });
                }
                // ANCHOR_END: code_container
                BottomSpacer(height(pct(15)))
            }
        }
    };
    // ANCHOR_END: dsl
}
