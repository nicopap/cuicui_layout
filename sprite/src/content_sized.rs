//! Systems to update a [`cuicui_layout::Node`]'s size based on a sprite's
//! and text's size, rather that fixed at spawn time.
//!
//! Size of `Handle2dMesh` is that of the AABB pre-transform bounds of the mesh.
//!
//! Sprite size is determined by either:
//! - [`bevy::sprite::Sprite::custom_size`] if it is `Some`.
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
#[cfg(feature = "sprite_text")]
use bevy::text::{Font, Text, Text2dBounds, TextPipeline};
use bevy::{
    ecs::prelude::*,
    ecs::{query::QueryItem, schedule::SystemSetConfig, system::SystemParam},
    math::Vec3Swizzles,
    prelude::{Assets, Handle, Image, Mesh, Vec2},
    sprite::Mesh2dHandle,
};
use cuicui_layout::{
    require_layout_recompute, ComputeContentParam, ComputeContentSize, ContentSizedComputeSystem,
    Node, Size,
};

#[derive(SystemParam)]
pub(crate) struct SpriteContentSize<'w> {
    #[cfg(feature = "sprite_text")]
    fonts: Res<'w, Assets<Font>>,
    images: Res<'w, Assets<Image>>,
    meshes: Res<'w, Assets<Mesh>>,
}
impl ComputeContentParam for SpriteContentSize<'static> {
    #[cfg(feature = "sprite_text")]
    type Components = AnyOf<(
        &'static Handle<Image>,
        &'static Mesh2dHandle,
        &'static Text,
        &'static Text2dBounds,
    )>;

    #[cfg(not(feature = "sprite_text"))]
    type Components = AnyOf<(&'static Handle<Image>, &'static Mesh2dHandle)>;

    fn condition(label: ContentSizedComputeSystem<Self>) -> SystemSetConfig {
        use bevy::ecs::schedule::common_conditions as cond;

        fn changed<C: Component>(q: Query<(), (Changed<C>, With<Node>)>) -> bool {
            !q.is_empty()
        }

        #[cfg(feature = "sprite_text")]
        let cond = cond::resource_changed::<Assets<Font>>()
            .or_else(cond::resource_changed::<Assets<Font>>())
            .or_else(changed::<Text>)
            .or_else(changed::<Text2dBounds>);
        #[cfg(not(feature = "sprite_text"))]
        let cond = || true;

        let cond = cond
            .or_else(changed::<Handle<Image>>)
            .or_else(changed::<Mesh2dHandle>);

        label.run_if(require_layout_recompute.or_else(cond))
    }
}
type OptSize = Size<Option<f32>>;
impl SpriteContentSize<'_> {
    #[cfg(feature = "sprite_text")]
    fn compute_text_size(&self, text: &Text, set_size: OptSize) -> Option<Size<f32>> {
        let inf = f32::INFINITY;
        let bounds = Vec2::new(
            set_size.width.unwrap_or(inf),
            set_size.height.unwrap_or(inf),
        );
        let measure = TextPipeline::default().create_text_measure(
            &self.fonts,
            &text.sections,
            // Seems like this requires an epsilon, otherwise text wraps poorly.
            1.01,
            text.alignment,
            text.linebreak_behavior,
        );
        Some(measure.ok()?.compute_size(bounds).into())
    }
    // TODO(perf): re-use AABB if present on entity
    // TODO(bug): preserve aspect ratio
    fn compute_mesh_size(&self, mesh: &Handle<Mesh>, _set_size: OptSize) -> Option<Size<f32>> {
        let mesh = self.meshes.get(mesh)?;
        let aabb = mesh.compute_aabb()?;
        let size = aabb.half_extents.xy() * 2.;
        Some(size.into())
    }
    // TODO(bug): Account for `Sprite::custom_size`, and all sprite fields generally.
    fn compute_image_size(&self, image: &Handle<Image>, set_size: OptSize) -> Option<Size<f32>> {
        let image = self.images.get(image)?;
        let size = image.size();
        let size = match (set_size.width, set_size.height) {
            (None, None) => size,
            (Some(width), None) => Vec2::new(width, width * size.y / size.x),
            (None, Some(height)) => Vec2::new(height * size.x / size.y, height),
            (Some(_), Some(_)) => unreachable!(
                "This is a bug in cuicui_layout, \
                the API promises that compute_content is never call with two set values"
            ),
        };
        Some(size.into())
    }
}
impl ComputeContentSize for SpriteContentSize<'_> {
    #[cfg(feature = "sprite_text")]
    type Components = AnyOf<(
        &'static Handle<Image>,
        &'static Mesh2dHandle,
        &'static Text,
        &'static Text2dBounds,
    )>;

    #[cfg(not(feature = "sprite_text"))]
    type Components = AnyOf<(&'static Handle<Image>, &'static Mesh2dHandle)>;

    fn compute_content(
        &self,
        components: QueryItem<Self::Components>,
        set_size: OptSize,
    ) -> Size<f32> {
        let size = match components {
            #[cfg(feature = "sprite_text")]
            (.., Some(text), Some(_)) => self.compute_text_size(text, set_size),
            (Some(image), ..) => self.compute_image_size(image, set_size),
            (_, Some(mesh), ..) => self.compute_mesh_size(&mesh.0, set_size),
            _ => unreachable!("This is a bevy bug"),
        };
        size.unwrap_or(Size::ZERO)
    }
}
