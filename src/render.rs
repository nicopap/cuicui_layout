//! Smooth out integration of cuicui layout with bevy, simply using
//! render targets and a camera component marker.

use bevy::{
    prelude::*,
    render::view::{Layer, RenderLayers},
};
use bevy_mod_sysfail::quick_sysfail;

use super::Root;

#[derive(Component, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect, FromReflect))]
pub struct UiCamera;

#[derive(Bundle)]
pub struct RootBundle {
    pub node: Root,
    pub layer: RenderLayers,
}

#[derive(Bundle)]
pub struct UiCameraBundle {
    #[bundle]
    pub camera: Camera2dBundle,
    pub layer: RenderLayers,
    pub ui_camera: UiCamera,
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
            ui_camera: UiCamera,
        }
    }
}

#[quick_sysfail]
pub fn update_ui_camera_root(
    ui_cameras: Query<(&Camera, &RenderLayers), (With<UiCamera>, Changed<Camera>)>,
    mut roots: Query<(&mut Root, &RenderLayers)>,
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
