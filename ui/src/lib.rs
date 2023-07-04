//! TODO
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(clippy::type_complexity, clippy::use_self, clippy::redundant_pub_crate)]

use bevy::ecs::prelude::*;
use bevy::prelude::{App, Camera, CoreSet, FromReflect, Plugin, Reflect, ReflectComponent, Style};
use bevy_mod_sysfail::quick_sysfail;
use content_sized::ContentSized;
use cuicui_layout::{PosRect, Root};

pub mod bundles;
pub mod content_sized;
mod debug;
mod dsl;
mod macros;

/// Traits defined by this crate.
pub mod traits {
    pub use crate::bundles::IntoUiBundle;
    pub use crate::dsl::LayoutCommandsExt;
}

/// Use this camera's logical size as the root fixed-size container for
/// `cuicui_layout`.
///
/// Note that it is an error to have more than a single camera with this
/// component.
#[derive(Component, Clone, Copy, Debug, Default, Reflect, FromReflect)]
#[reflect(Component)]
pub struct LayoutRootCamera;

/// Set this [`cuicui_layout::Root`] to track the [`LayoutRootCamera`]'s size.
#[derive(Component, Clone, Copy, Debug, Default, Reflect, FromReflect)]
#[reflect(Component)]
pub struct ScreenRoot;

/// System updating the [`ScreenRoot`] [`cuicui_layout::Node`] with the
/// [`LayoutRootCamera`]'s viewport size, whenever it changes.
#[quick_sysfail]
pub fn update_ui_camera_root(
    ui_cameras: Query<&Camera, (With<LayoutRootCamera>, Changed<Camera>)>,
    mut roots: Query<&mut Root>,
) {
    for cam in &ui_cameras {
        let size = cam.logical_viewport_size()?;
        for mut root in roots.iter_mut() {
            root.bounds.width = size.x;
            root.bounds.height = size.y;
        }
    }
}

/// Set the [`Style`]'s `{min_,max_,}size.{width,height}` and `position.{left,right}`
/// according to [`PosRect`]'s computed from [`cuicui_layout`].
pub fn set_layout_style(
    mut query: Query<(&mut Style, &PosRect), (Changed<PosRect>, Without<ContentSized>)>,
) {
    use bevy::ui::{PositionType, Val};
    query.for_each_mut(|(mut style, pos)| {
        style.position_type = PositionType::Absolute;
        style.position.left = Val::Px(pos.pos().x);
        style.position.top = Val::Px(pos.pos().y);

        let width = Val::Px(pos.size().width);
        style.min_size.width = width;
        style.max_size.width = width;
        style.size.width = width;

        let height = Val::Px(pos.size().height);
        style.min_size.height = height;
        style.max_size.height = height;
        style.size.height = height;
    });
}

/// Plugin managing position and size of UI elements using [`cuicui_layout`]
/// components.
///
/// What this does:
///
/// - **Manage size of text and image elements**: UI elements spawned through [`spawn_ui`]
///   (with the [`ContentSized`] component)
/// - **Manage size of the [`ScreenRoot`] container**
/// - **Set the [`Style`] flex parameters according to [`cuicui_layout`] computed values**
/// - **Compute [`cuicui_layout::Node`] layouts**
///
/// [`spawn_ui`]: dsl::LayoutCommandsExt::spawn_ui
/// [`ContentSized`]: content_sized::ContentSized
pub struct Plug;
impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        use bevy::ui::UiSystem;
        use cuicui_layout::Systems::ComputeLayout;
        use CoreSet::PostUpdate;

        app.add_plugin(cuicui_layout::Plug::new())
            .add_system(content_sized::update.before(ComputeLayout))
            .add_system(update_ui_camera_root.before(ComputeLayout))
            .add_system(content_sized::add.after(ComputeLayout))
            .add_system(content_sized::clear.after(ComputeLayout))
            .add_system(
                set_layout_style
                    .before(UiSystem::Flex)
                    .in_base_set(PostUpdate),
            );
    }
}
