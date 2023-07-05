use bevy::prelude::*;
use cuicui_layout::Rule;
use cuicui_layout_bevy_ui::{layout, traits::LayoutCommandsExt, LayoutRootCamera};

macro_rules! text {
    ($handle:expr, $value:expr) => {
        Text::from_section($value, TextStyle {
            font: $handle.clone(),
            font_size: 32.0,
            ..Default::default()
        })
    };
    ($handle:expr, $($tail:tt)*) => {
        Text::from_section(format!($($tail)*), TextStyle {
            font: $handle.clone(),
            font_size: 32.0,
            ..Default::default()
        })
    };
}

fn main() {
    use bevy_inspector_egui::quick::WorldInspectorPlugin;
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            asset_folder: "../../assets".to_owned(),
            watch_for_changes: true,
        }))
        .add_startup_system(setup)
        .add_startup_system(random_bg.in_base_set(StartupSet::PostStartup))
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(cuicui_layout_bevy_ui::Plug)
        .run();
}

#[allow(clippy::cast_precision_loss, clippy::unreadable_literal)]
fn color_from_entity(entity: Entity) -> Color {
    use ahash::random_state::RandomState;
    const U64_TO_DEGREES: f32 = 360.0 / u64::MAX as f32;

    const STATE: RandomState =
        RandomState::with_seeds(5952553601252303067, 16866614500153072625, 0, 0);
    let hash = STATE.hash_one(entity);

    let hue = hash as f32 * U64_TO_DEGREES;
    Color::hsla(hue, 0.8, 0.5, 1.0)
}
fn random_bg(mut query: Query<(Entity, &mut BackgroundColor)>) {
    for (entity, mut bg) in &mut query {
        bg.0 = color_from_entity(entity);
    }
}
#[allow(clippy::needless_pass_by_value)]
fn _setup_with_macro(mut cmds: Commands, serv: Res<AssetServer>) {
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

    layout! {
        &mut cmds,
        row(screen_root, "root", main_margin 100, align_start) {
            column("menu", width px 300, fill_main_axis) {
                spawn_ui(title_card, "Title card", height px 100, width %100);
                code(let cmds) {
                    for n in &menu_buttons {
                        let name = format!("{n} button");
                        layout!(cmds, spawn_ui(text!(font, *n), named name, height px 30););
                    }
                }
            }
        }
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

    let width = Rule::Fixed(300.0);
    let t_height = Rule::Fixed(100.0);
    let b_height = Rule::Fixed(30.0);
    cmds.align_start().main_margin(100.0).named("root").screen_root().row(|cmds| {
        cmds.fill_main_axis().main_margin(40.0).width_rule(width).named("menu").column(|cmds| {
            cmds.width_rule(Rule::Parent(1.0))
                .height_rule(t_height)
                .named("Title")
                .spawn_ui(title_card);
            for n in &menu_buttons {
                cmds.height_rule(b_height).named(format!("{n} button")).spawn_ui(text!(font, *n));
            }
        });
    });
}
