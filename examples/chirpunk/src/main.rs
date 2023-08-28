/*!
An end-to-end example of a complex menu, using the `.chirp` file format.

This is a carbon-copy of: <https://github.com/IDEDARY/bevy-lunex-cyberpunk/tree/main>

The goal is to demonstrate the full capabilities of `cuicui_chirp`. Specifically:

- **Flexibility**: Define your own components/behavior, use them in `cuicui_chirp`
  seamlessly
- **Tersness**: Minimal effort involved in integrating your behaviors with `cuicui_chirp`
- **Speed of development**: Going from 0 to fully working prototype should be fast.
  This is accomplished thanks to (1) Hot reloading, (2) simple layouting algorithm,
  (3) full integration with pre-existing stuff.

## Setup

The cyberpunk assets are **NOT CHECKED OUT**. Check them out as follow:

- Your PWD (working directory) should be the root of this repositry.
- If you have `make` installed, running `make checkout-cyberpunk` should be enough
- If not, the code to setup is as follow (still with the repositroy root as working directory):

```sh
git clone --no-checkout --depth=1 --filter=tree:0 \
    https://github.com/IDEDARY/bevy-lunex-cyberpunk.git \
    examples/chirpunk/lunex-cyberpunk-assets
cd examples/chirpunk/lunex-cyberpunk-assets
git sparse-checkout set --no-cone assets
git checkout
cd ..
../../scripts/x_platform_ln.sh lunex-cyberpunk-assets/assets assets
cd lunex-cyberpunk-assets/assets
../../../../scripts/x_platform_ln.sh ../../menus menus
cd ../../../..
```

## Limitations

`cuicui_layout_bevy_sprite` is currently not good enough for this, so as a temporary
measure we use `cuicui_layout_bevy_ui`. Which has its own limitations:

- Image scale with layout size. So we get wonky distorsion on the background
  in non-16:9 window resolutions.

## Architecture

We define a few components for the visual effects

- [`ui_offset::UiOffset`]
- [`button_shift::Animation`]

*/
#![allow(clippy::needless_pass_by_value)]

use std::time::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use cuicui_chirp::Chirp;
use cuicui_layout::LayoutRootCamera;
use cuicui_layout_bevy_ui::UiDsl;

use animate::button_shift;
use ui_offset::UiOffset;

/// Animate stuff.
///
/// - Main menu bg police car strobe lights
/// - Main menu bg "camera shift" effect
/// - Highlight of selected entry
mod animate;
/// Extensions to the DSL required for the menu.
mod dsl;
/// Runtime style controls.
///
/// Basically all color, fonts and animations are defined in this module. Since
/// they are defined as a resource, it's perfectly imaginable to load them and
/// update them at runtime.
mod style;
/// Move `bevy_ui` nodes AFTER layouting, for "offset effects".
mod ui_offset;

/// Methods for mixing bevy colors using hsluv 💞
mod colormix;

type BgColor = BackgroundColor;
type StyleComponents = AnyOf<(
    &'static mut Style,
    &'static mut Text,
    &'static mut BgColor,
    &'static mut button_shift::Animation,
)>;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct MainMenuItem;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::TRACE,
                    filter: "\
                      cuicui_layout=trace,cuicui_layout_bevy_ui=trace,\
                      gilrs_core=info,gilrs=info,\
                      naga=info,wgpu=error,wgpu_hal=error"
                        .to_string(),
                }),
            style::Plugin,
            animate::Plugin,
            ui_offset::Plugin,
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
    cmds.spawn(assets.load::<Chirp, _>("menus/main.chirp"));
    cmds.spawn(assets.load::<Chirp, _>("menus/settings.chirp"));
}
