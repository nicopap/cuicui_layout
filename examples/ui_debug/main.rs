use bevy::prelude::*;
use cuicui_layout::{layout, LayoutRootCamera, Rule};
use cuicui_layout_bevy_ui::dsl_extension::{BevyUiCommandsExt, IntoUiCommands};

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
        // .add_startup_system(random_bg.in_base_set(StartupSet::PostStartup))
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(cuicui_layout_bevy_ui::Plug)
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
    let bg = serv.load("background.png");
    let board = serv.load("board.png");
    let button = serv.load("button.png");

    layout! {
        <> &mut cmds,
        row(screen_root, "root", main_margin 100., align_start, image &bg) {
            column("menu", width px 300, fill_main_axis, image &board) {
                spawn_ui(title_card, "Title card", height px 100, width %100);
                code(let cmds) {
                    for n in &menu_buttons {
                        let name = format!("{n} button");
                        layout!(<> cmds,
                            spawn_ui(text!(font, *n), named name, image &button, height px 30););
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
    let bg = serv.load("background.png");
    let board = serv.load("board.png");
    let button = serv.load("button.png");

    let width = Rule::Fixed(310.0);
    let t_height = Rule::Fixed(100.0);
    let b_height = Rule::Fixed(30.0);

    let aligned = cmds.lyout().image(&bg).align_start().named("root");

    aligned.main_margin(100.0).screen_root().row(|cmds| {
        let menu = cmds.lyout().image(&board).main_margin(40.0).width_rule(width);

        menu.fill_main_axis().named("menu").column(|cmds| {
            let title = cmds.lyout().width_rule(Rule::Parent(1.0)).height_rule(t_height);

            title.named("Title").spawn_ui(title_card);
            for n in &menu_buttons {
                let button = cmds.lyout().height_rule(b_height).image(&button);

                button.named(format!("{n} button")).spawn_ui(text!(font, *n));
            }
        });
    });
}
