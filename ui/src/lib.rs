//! Make [`cuicui_layout`] useable with bevy's UI library (`bevy_ui`).
//!
//! Import this crate's [`UiDsl`] and use [`cuicui_dsl::dsl!`] with
//! it to have a fully working UI library.
//!
//! It contains:
//! * A [`dsl`] to use with the [`cuicui_dsl::dsl!`] macro.
//!
//! Note that **unlike `cuicui_layout_bevy_ui`, this uses a Y axis down**
//! coordinate space, (like `bevy_sprite`)
//!
//! Therefore, if you happen to convert your layouts from `bevy_ui` to `bevy_sprite`
//! (or vis-versa) what was on top will be at the bottom and vis-versa.
//!
//! # Example
//!
//! ```
//! use bevy::prelude::*;
//! use cuicui_layout::{dsl, LayoutRootCamera, Rule};
//! // dsl! will use this crate's extensions to DslBundle
//! // if you import this      vvvvvvvvvvvv
//! use cuicui_layout_bevy_ui::UiDsl as Dsl;
//! use cuicui_layout::dsl_functions::{px, pct};
//!
//! # fn setup(mut cmds: Commands, serv: Res<AssetServer>) {
//! cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));
//!
//! let title_card = serv.load::<Image>("logo.png");
//! let menu_buttons = [ "CONTINUE", "NEW GAME" ];
//! let bg = serv.load("background.png");
//! let board = serv.load("board.png");
//! let button = serv.load("button.png");
//!
//! dsl! {
//!     &mut cmds.spawn_empty(),
//!     // Notice the `image` argument                     vvvvvvvvvv
//!     Root(row screen_root main_margin(100.) align_start image(&bg)) {
//!         Menu(column width(px(310)) main_margin(40.) fill_main_axis image(&board)) {
//!             TitleCard(ui(title_card) height(px(100)) width(pct(100)))
//!             code(let cmds) {
//!                 for n in &menu_buttons {
//!                     let name = format!("{n} button");
//!                     dsl!(cmds, Entity(ui(*n) named(name) image(&button) height(px(30))))
//!                 }
//!             }
//!         }
//!     }
//! };
//! # }
//! ```
//!
//! [`DslBundle`]: cuicui_dsl::DslBundle

use bevy::app::{App, Plugin as BevyPlugin};
use bevy::ecs::prelude::*;
use bevy::render::camera::Camera;
use bevy::ui::Style;
use bevy_mod_sysfail::quick_sysfail;
use cuicui_layout::content_sized::AppContentSizeExt;
use cuicui_layout::{LayoutRect, LayoutRootCamera, Root, ScreenRoot};

pub use dsl::UiDsl;

mod fixup;

pub mod content_sized;
pub mod dsl;

#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
pub struct TestWorkspaceReadme;

/// System updating the [`ScreenRoot`] [`cuicui_layout::Node`] with the
/// [`LayoutRootCamera`]'s viewport size, whenever it changes.
#[quick_sysfail]
pub fn update_ui_camera_root(
    ui_cameras: Query<&Camera, (With<LayoutRootCamera>, Changed<Camera>)>,
    mut roots: Query<&mut Root, With<ScreenRoot>>,
) {
    for cam in &ui_cameras {
        let size = cam.logical_viewport_size()?;
        for mut root in &mut roots {
            let bounds = root.size_mut();
            *bounds.width = size.x;
            *bounds.height = size.y;
        }
    }
}
// Note: if root is spawned but there isn't yet a camera associated with it,
// `update_layout_camera_root will take care of it when camera is added.
/// System setting the size of newly added [`ScreenRoot`] nodes.
///
/// This differs from [`update_ui_camera_root`] in that:
/// - `update_ui_camera_root` sets size for  **pre-existing roots** when **cameras change**
/// - `set_added_camera_root` sets size for **newly added roots** on **pre-existing cameras**
#[quick_sysfail]
pub fn set_added_camera_root(
    ui_cameras: Query<&Camera, With<LayoutRootCamera>>,
    mut roots: Query<&mut Root, Added<ScreenRoot>>,
) {
    for mut root in &mut roots {
        let Some(camera) = ui_cameras.iter().next() else {
            continue;
        };
        let size = camera.logical_viewport_size()?;
        let bounds = root.size_mut();
        *bounds.width = size.x;
        *bounds.height = size.y;
    }
}

/// Set the [`Style`]'s `{min_,max_,}size.{width,height}` and `position.{left,right}`
/// according to [`LayoutRect`]'s computed from [`cuicui_layout`].
pub fn set_layout_style(mut query: Query<(&mut Style, &LayoutRect), Changed<LayoutRect>>) {
    use bevy::ui::{PositionType, Val};
    query.for_each_mut(|(mut style, pos)| {
        style.position_type = PositionType::Absolute;
        style.left = Val::Px(pos.pos().x);
        style.top = Val::Px(pos.pos().y);

        let width = Val::Px(pos.size().width);
        style.min_width = width;
        style.max_width = width;
        style.width = width;

        let height = Val::Px(pos.size().height);
        style.min_height = height;
        style.max_height = height;
        style.height = height;
    });
}

/// Plugin managing position and size of UI elements using [`cuicui_layout`]
/// components.
///
/// What this does:
///
/// - **Manage size of text and image elements**
/// - **Manage size of the [`cuicui_layout::ScreenRoot`] container**
/// - **Set the [`Style`] flex parameters according to [`cuicui_layout`] computed values**
/// - **Compute [`cuicui_layout::Node`] layouts**
///
/// [`spawn_ui`]: cuicui_layout::dsl::LayoutDsl::spawn_ui
/// [`ContentSized`]: cuicui_layout::ContentSized
pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use bevy::prelude::{Last, PostUpdate, Update};
        use bevy::ui::UiSystem;
        use cuicui_layout::ComputeLayoutSet;

        app.add_plugins(cuicui_layout::Plugin)
            .add_content_sized::<content_sized::UiContentSize>()
            .add_systems(
                Update,
                (update_ui_camera_root, set_added_camera_root).before(ComputeLayoutSet),
            )
            .add_systems(PostUpdate, set_layout_style.before(UiSystem::Layout))
            .add_systems(
                Last,
                (fixup::add_text_components, fixup::add_image_components),
            );
    }
}
