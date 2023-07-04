//! Smooth out integration of cuicui layout with bevy, simply using
//! render targets and a camera component marker.
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(clippy::use_self, clippy::redundant_pub_crate)]

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

/// Create a [`Root`] container as the screen root, its size will dyamically
/// follow the size of the viewport of camera marked iwth [`LayoutRootCamera`].
#[derive(Bundle)]
pub struct RootBundle {
    /// The [`cuicui_layout`] [`Root`] container parameters.
    pub node: Root,
    /// The [`RenderLayers`] in which this appears. Currently only used to
    /// associate a root to a camera with identical [`RenderLayers`].
    pub layer: RenderLayers,
    /// Set this to track the [`LayoutRootCamera`]'s size.
    pub screen_root: ScreenRoot,
}

/// Camera which screen boundaries are used as the size of the [`ScreenRoot`]
/// layout root.
///
/// Use [`UiCameraBundle::layer`] to limit this camera's rendering to entities
/// in the given [`RenderLayers`].
#[derive(Bundle)]
pub struct UiCameraBundle {
    /// The bevy camera components.
    pub camera: Camera2dBundle,
    /// Limit this camera's rendering to entities within given [`RenderLayers`].
    pub layer: RenderLayers,
    /// Use this camer'as logical size for the [`ScreenRoot`] container size.
    pub ui_camera: LayoutRootCamera,
}
impl UiCameraBundle {
    /// Create a camera limited to the provided [`Layer`].
    #[must_use]
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

/// System updating the [`ScreenRoot`] [`cuicui_layout`] [`Node`] with the
/// [`LayoutRootCamera`]'s viewport size, whenever it changes.
#[quick_sysfail]
#[allow(clippy::type_complexity)]
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
