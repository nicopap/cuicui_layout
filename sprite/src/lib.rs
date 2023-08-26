//! Make [`cuicui_layout`] useable with bevy's 2D renderer (`bevy_sprite`).
//!
//! Import this crate's [`SpriteDsl`] and use [`cuicui_dsl::dsl!`] with
//! it to have a fully working UI library.
//!
//! Note that **unlike `cuicui_layout_bevy_ui`, this uses a Y axis down**
//! coordinate space, (like `bevy_sprite`)
//!
//! Therefore, if you happen to convert your layouts from `bevy_ui` to `bevy_sprite`
//! (or vis-versa) what was on top will be at the bottom and vis-versa.
//!
//! This might be changed in the future, so beware!
//!
//! [`Sprite`]: bevy::sprite::Sprite
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(
    clippy::use_self,
    clippy::redundant_pub_crate,
    clippy::module_name_repetitions
)]

use bevy::app::{App, Plugin as BevyPlugin};
use bevy::ecs::prelude::*;
use bevy::prelude::{Camera, Camera2dBundle, OrthographicProjection, Transform, Vec2};
use bevy::render::view::{Layer, RenderLayers};
use bevy::utils::default;
use bevy_mod_sysfail::quick_sysfail;
use cuicui_layout::{AppContentSizeExt, LayoutRect, LayoutRootCamera, Root, ScreenRoot};

pub use dsl::SpriteDsl;

pub mod content_sized;
pub mod dsl;

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
///
/// [`Node`]: cuicui_layout::Node
#[quick_sysfail]
pub fn update_layout_camera_root(
    ui_cameras: Query<(&Camera, &RenderLayers), (With<LayoutRootCamera>, Changed<Camera>)>,
    mut roots: Query<(&mut Root, &RenderLayers), With<ScreenRoot>>,
) {
    for (cam, layers) in &ui_cameras {
        let size = cam.logical_viewport_size()?;
        let is_layer = |(r, l)| (l == layers).then_some(r);
        for mut root in roots.iter_mut().filter_map(is_layer) {
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
/// This differs from [`update_layout_camera_root`] in that:
/// - `update_layout_camera_root` sets size for  **pre-existing roots** when **cameras change**
/// - `set_added_layout_camera_root` sets size for **newly added roots** on **pre-existing cameras**
#[quick_sysfail]
pub fn set_added_layout_camera_root(
    ui_cameras: Query<(&Camera, &RenderLayers), With<LayoutRootCamera>>,
    mut roots: Query<(&mut Root, &RenderLayers), Added<ScreenRoot>>,
) {
    for (mut root, layers) in &mut roots {
        let is_layer = |(c, l)| (l == layers).then_some(c);
        let Some(camera) = ui_cameras.iter().find_map(is_layer) else {
            continue;
        };
        let size = camera.logical_viewport_size()?;
        let bounds = root.size_mut();
        *bounds.width = size.x;
        *bounds.height = size.y;
    }
}
/// Set the [`Transform`]s according to [`LayoutRect`]'s computed from [`cuicui_layout`].
pub fn update_layout_transform(
    mut query: Query<(&mut Transform, &LayoutRect), Changed<LayoutRect>>,
) {
    query.for_each_mut(|(mut transform, rect)| {
        let z = transform.translation.z;
        transform.translation = rect.pos().extend(z);
    });
}

/// Plugin managing position and size of `bevy_sprite` renderable components
///  using [`cuicui_layout`] components.
///
/// What this does:
///
/// - Manage size of [`Sprite`], [`Mesh2dHandle`] and [`Text2dBundle`] components
///   based on their `cuicui_layout`-infered size.
/// - Manage the size of content-sized [`cuicui_layout::Node`].
/// - Manage size of the [`cuicui_layout::ScreenRoot`] container
/// - Set the [`Transform`] of entities with a [`cuicui_layout::Node`] component
///
/// [`Sprite`]: bevy::sprite::Sprite
/// [`Mesh2dHandle`]: bevy::sprite::Mesh2dHandle
/// [`Text2dBundle`]: bevy::text::Text2dBundle
pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use bevy::prelude::Update;
        use cuicui_layout::ComputeLayoutSet;

        app.add_plugins(cuicui_layout::Plugin)
            .add_content_sized::<content_sized::SpriteContentSize>()
            .add_systems(
                Update,
                (
                    (update_layout_camera_root, set_added_layout_camera_root)
                        .before(ComputeLayoutSet),
                    update_layout_transform.after(ComputeLayoutSet),
                ),
            );
    }
}
