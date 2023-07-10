//! Systems to update a [`cuicui_layout::Node`]'s size based on a sprite's
//! and text's size, rather that fixed at spawn time.
//!
//! Size of `Handle2dMesh` is that of the AABB pre-transform bounds of the mesh.
//!
//! Sprite size is determined by either:
//! - [`Sprite::custom_size`] if it is `Some`.
//! - The `Handle<Image>` size of the same entity.
//!
//! Text size is determined by the [`Text2dBounds`] component.
//! You may update the component's `size` field and have `cuicui_layout`
//! follow it. But otherwise, `cuicui_layout` won't dynamically update
//! the text size based on its content.
//!
//! # Limitations
//!
//! Since the content size of meshes and textures depend on the asset's concret
//! value, it won't properly update if the underlying asset changes size but the
//! `Handle<T>` component doesn't change.
//!
//! Also, sizes do not take into account the `Transform` size. I'm not sure how
//! wishable as a feature this is, so please open an issue if you want it.
//!
//! This relies on the [`ContentSized`] component.
#[cfg(feature = "sprite_text")]
use bevy::text::Text2dBounds;
use bevy::{
    ecs::prelude::{AnyOf, Changed, Or, Query, Res, With},
    ecs::query::WorldQuery,
    math::Vec3Swizzles,
    prelude::{Assets, Handle, Image, Mesh, Vec2},
    sprite::{Mesh2dHandle, Sprite},
};
use cuicui_layout::{ContentSized, LeafRule, Node, Size};

/// [`WorldQuery`] for entities that can be sized based on [`bevy::sprite`]
/// components (such as [`Sprite`]).
#[derive(WorldQuery)]
pub struct SpriteSize {
    #[cfg(not(feature = "sprite_text"))]
    item: AnyOf<(
        &'static Mesh2dHandle,
        (&'static Sprite, &'static Handle<Image>),
    )>,
    #[cfg(feature = "sprite_text")]
    item: AnyOf<(
        &'static Mesh2dHandle,
        (&'static Sprite, &'static Handle<Image>),
        &'static Text2dBounds,
    )>,
}
impl SpriteSizeItem<'_> {
    #[allow(clippy::cast_precision_loss)] // We know texture sizes are bellow 10K, so casting to f32 is fine
    fn common_get(&self, meshes: &Assets<Mesh>, images: &Assets<Image>) -> Option<Vec2> {
        if let Some(mesh) = self.item.0 {
            let mesh = meshes.get(&mesh.0)?;
            let aabb = mesh.compute_aabb()?;
            return Some(aabb.half_extents.xy() * 2.);
        }
        let (sprite, image_handle) = self.item.1?;
        if let Some(custom_size) = sprite.custom_size {
            return Some(custom_size);
        }
        let image = images.get(image_handle)?;
        let size = image.texture_descriptor.size;
        Some(Vec2::new(size.width as f32, size.height as f32))
    }
    #[cfg(not(feature = "sprite_text"))]
    fn get(&self, meshes: &Assets<Mesh>, images: &Assets<Image>) -> Option<Vec2> {
        self.common_get(meshes, images)
    }
    #[cfg(feature = "sprite_text")]
    fn get(&self, meshes: &Assets<Mesh>, images: &Assets<Image>) -> Option<Vec2> {
        let text_bounds = self.item.2.map(|t| t.size);
        text_bounds.or_else(|| self.common_get(meshes, images))
    }
}

#[cfg(feature = "sprite_text")]
type SizeChange = Or<(
    Changed<Handle<Image>>,
    Changed<Mesh2dHandle>,
    Changed<Sprite>,
    Changed<Text2dBounds>,
)>;
#[cfg(not(feature = "sprite_text"))]
type SizeChange = Or<(
    Changed<Handle<Image>>,
    Changed<Mesh2dHandle>,
    Changed<Sprite>,
)>;

/// Update the [`cuicui_layout`] [`Node::Box`] [`LeafRule::Fixed`] values of
/// entities with a [`Sprite`], [`Mesh2dHandle`]  or `Text2dBound` components.
/// (the latter, when the `"sprite_text"` feature is enabled)
#[allow(clippy::needless_pass_by_value)] // systems trip clippy on this every time
pub fn update(
    images: Res<Assets<Image>>,
    meshes: Res<Assets<Mesh>>,
    mut query: Query<(&mut Node, SpriteSize), (SizeChange, With<ContentSized>)>,
) {
    for (mut node, size) in &mut query {
        let Some(size) = size.get(&meshes, &images) else { continue; };
        // TODO(bug): If only a signle axis of an image is Fixed, then use the image's aspect
        // ratio to "fix" the other axis.
        if let Node::Box(Size { width: LeafRule::Fixed(value), .. }) = &mut *node {
            *value = size.x;
        }
        if let Node::Box(Size { height: LeafRule::Fixed(value), .. }) = &mut *node {
            *value = size.y;
        }
    }
}
