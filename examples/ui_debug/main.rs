use std::time::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cuicui_dsl::dsl;
use cuicui_layout::{
    dsl_functions::{pct, px},
    LayoutRootCamera,
};
use cuicui_layout_bevy_ui::UiDsl as Dsl;

macro_rules! text {
    ($handle:expr, $value:expr) => {
        Text::from_section($value, TextStyle {
            font: $handle.clone(),
            font_size: 30.0,
            ..Default::default()
        })
    };
    ($handle:expr, $($tail:tt)*) => {
        Text::from_section(format!($($tail)*), TextStyle {
            font: $handle.clone(),
            font_size: 30.0,
            ..Default::default()
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
                    cuicui_layout=trace,\
                    cuicui_layout_bevy_ui=trace,\
                    gilrs_core=info,\
                    gilrs=info,\
                    naga=info,\
                    wgpu=error,\
                    wgpu_hal=error\
                    "
                    .to_string(),
                }),
            cuicui_layout_bevy_ui::Plugin,
            WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, setup)
        // .add_startup_system(random_bg.in_base_set(StartupSet::PostStartup))
        .run();
}

#[allow(clippy::cast_precision_loss, clippy::unreadable_literal)]
fn _color_from_entity(entity: Entity) -> Color {
    use ahash::random_state::RandomState;
    const U64_TO_DEGREES: f32 = 360.0 / u64::MAX as f32;

    const STATE: RandomState =
        RandomState::with_seeds(5952553601252303067, 16866614500153072625, 0, 0);
    let hash = STATE.hash_one(entity);

    let hue = hash as f32 * U64_TO_DEGREES;
    Color::hsla(hue, 0.4, 0.9, 1.0)
}
fn _random_bg(mut query: Query<(Entity, &mut BackgroundColor)>) {
    for (entity, mut bg) in &mut query {
        bg.0 = _color_from_entity(entity);
    }
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
    let font = serv.load("adobe_sans.ttf");
    let bg = serv.load("background.png");
    let board = serv.load("board.png");
    let button = serv.load("button.png");

    dsl! {
        &mut cmds,
        row(screen_root, "root", main_margin 100., align_start, image &bg) {
            column("menu", width px(310), height pct(100), main_margin 40., fill_main_axis, image &board) {
                spawn_ui(title_card, "Title card", height px(100), width pct(100));
                code(let cmds) {
                    for n in &menu_buttons {
                        let name = format!("{n} button");
                        dsl!(cmds, spawn_ui(text!(font, *n), named name, image &button, height px(30)););
                    }
                }
            }
        }
    };
}
