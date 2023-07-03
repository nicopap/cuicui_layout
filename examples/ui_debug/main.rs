use bevy::prelude::*;
use cuicui_layout::Rule;
use cuicui_layout_bevy_ui::{traits::LayoutCommandsExt, LayoutRootCamera};

macro_rules! text {
    ($handle:expr, $value:expr) => {
        Text::from_section($value, TextStyle {
            font: $handle.clone(),
            ..Default::default()
        })
    };
    ($handle:expr, $($tail:tt)*) => {
        Text::from_section(format!($($tail)*), TextStyle {
            font: $handle.clone(),
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
        .add_plugin(cuicui_layout_bevy_ui::Plug::new())
        .run();
}

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
    let mut menu_entites = Vec::with_capacity(menu_buttons.len());
    let font = serv.load("adobe_sans.ttf");

    cmds.screen_root().named("root").main_margin(100.0).align_start().row(|cmds| {
        cmds.named("menu").width_rule(Rule::Fixed(300.0)).fill_main_axis().column(|cmds| {
            cmds.named("Title card").height_rule(Rule::Fixed(100.0)).spawn_ui(title_card.clone());

            menu_entites.extend(menu_buttons.iter().map(|n| {
                cmds.named(format!("{n} button"))
                    .height_rule(Rule::Fixed(30.0))
                    .spawn_ui(text!(font, *n))
            }));
        })
    });
    // layout!{
    //     cmds,
    //     row(screen_root, main_margin 100, align_start) {
    //         column(width 300, main_margin 40, fill_main_axis) {
    //             spawn_ui(title_card.clone());
    //
    //             code(|cmds| {
    //                 menu_entites.extend(menu_buttons.iter().map(|n| cmds.spawn_ui(*n)));
    //             });
    //         }
    //     }
    // }
}
