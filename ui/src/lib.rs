#![allow(clippy::type_complexity)]
use std::marker::PhantomData;

use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*};
use bevy_mod_sysfail::quick_sysfail;
use content_size::ContentSize;
use cuicui_layout::{PosRect, Root};

pub mod bundles;
pub mod content_size;
mod debug;
mod into_ui_bundle;
mod layout_dsl;

pub mod traits {
    pub use crate::into_ui_bundle::IntoUiBundle;
    pub use crate::layout_dsl::LayoutCommandsExt;
}

/// Use this camera's logical size as the root fixed-size container for
/// `cuicui_layout`.
#[derive(Component, Clone, Copy, Debug, Default, Reflect, FromReflect)]
#[reflect(Component)]
pub struct LayoutRootCamera;

/// Set this [`cuicui_layout::Root`] to track the [`LayoutRootCamera`]'s size.
#[derive(Component, Clone, Copy, Debug, Default, Reflect, FromReflect)]
#[reflect(Component)]
pub struct ScreenRoot;

#[derive(Bundle)]
pub struct RootBundle {
    pub node: Root,
    pub screen_root: ScreenRoot,
}

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

/// Set the [`Sytle`]'s `{min_,max_,}size.{width,height}` and `position.{left,right}`
/// according to [`PosRect`]'s computed from [`cuicui_layout`].
pub fn set_layout_style(
    mut query: Query<(&mut Style, &PosRect), (Changed<PosRect>, Without<ContentSize>)>,
) {
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
    })
}

/// Plugin managing position and size of UI elements using [`cuicui_layout`]
/// components.
///
/// See [`set_layout_style`] for details.
///
/// The `F` type parameter is the additional filters to use.
pub struct Plug<F = ()>(PhantomData<fn(F)>);
impl Plug<()> {
    pub const fn new() -> Self {
        Plug(PhantomData)
    }
    pub const fn filter<F: ReadOnlyWorldQuery + 'static>() -> Plug<F> {
        Plug(PhantomData)
    }
}

impl<F: ReadOnlyWorldQuery + 'static> Plugin for Plug<F> {
    fn build(&self, app: &mut App) {
        use bevy::ui::UiSystem;
        use CoreSet::PostUpdate;

        app.add_plugin(cuicui_layout::Plug::filter::<F>())
            .add_system(content_size::update_node.before(cuicui_layout::Systems::ComputeLayout))
            .add_system(update_ui_camera_root.before(cuicui_layout::Systems::ComputeLayout))
            .add_system(content_size::add_content_size.after(cuicui_layout::Systems::ComputeLayout))
            .add_system(
                content_size::clear_content_size.after(cuicui_layout::Systems::ComputeLayout),
            )
            .add_system(
                set_layout_style
                    .before(UiSystem::Flex)
                    .in_base_set(PostUpdate),
            );
    }
}
