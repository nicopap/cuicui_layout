//! Smooth out integration of cuicui layout with bevy, simply using
//! render targets and a camera component marker.

use bevy::{
    prelude::*,
    render::view::{Layer, RenderLayers},
};
use bevy_mod_sysfail::quick_sysfail;
use cuicui_layout::Root;

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

#[derive(Bundle)]
pub struct RootBundle {
    pub node: Root,
    pub layer: RenderLayers,
    pub screen_root: ScreenRoot,
}

#[derive(Bundle)]
pub struct UiCameraBundle {
    pub camera: Camera2dBundle,
    pub layer: RenderLayers,
    pub ui_camera: LayoutRootCamera,
}
impl UiCameraBundle {
    pub fn for_layer(order: isize, layer: Layer) -> Self {
        UiCameraBundle {
            camera: Camera2dBundle {
                projection: OrthographicProjection {
                    far: 1000.0,
                    viewport_origin: Vec2::new(0.0, 0.0),
                    ..default()
                },
                camera: Camera { order, ..default() },
                ..default()
            },
            layer: RenderLayers::none().with(layer),
            ui_camera: LayoutRootCamera,
        }
    }
}

#[quick_sysfail]
pub fn update_ui_camera_root(
    ui_cameras: Query<(&Camera, &RenderLayers), (With<LayoutRootCamera>, Changed<Camera>)>,
    mut roots: Query<(&mut Root, &RenderLayers), With<ScreenRoot>>,
) {
    for (cam, layers) in &ui_cameras {
        let size = cam.logical_viewport_size()?;
        let is_layer = |(r, l)| (l == layers).then_some(r);
        for mut root in roots.iter_mut().filter_map(is_layer) {
            root.bounds.width = size.x;
            root.bounds.height = size.y;
        }
    }
}
// TODO:
// Compute the Static (and likely offset) of sprites and meshes (including
// rotation)
