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
#![allow(clippy::needless_pass_by_value)]

use bevy::ecs::prelude::*;
use bevy::math::Vec3Swizzles;
use bevy::prelude::{App, Assets, Handle, Image, Mesh, Plugin, Update, Vec2};
use bevy::sprite::Mesh2dHandle;
#[cfg(feature = "sprite_text")]
use bevy::text::{Font, Text, Text2dBounds, TextPipeline};
use cuicui_layout::{ContentSized, ContentSizedSet, Node};

/// Adjust the [`cuicui_layout`] [`ContentSized`] component based on `bevy_sprite`
/// sizeable content.
pub struct SpriteContentSizePlugin;

impl Plugin for SpriteContentSizePlugin {
    fn build(&self, app: &mut App) {
        use bevy::ecs::schedule::common_conditions::resource_changed as res;

        fn changed<C: Component>(q: Query<(), (Changed<C>, With<Node>)>) -> bool {
            !q.is_empty()
        }
        #[cfg(feature = "sprite_text")]
        let text_cond = res::<Assets<Font>>()
            .or_else(changed::<Text>)
            .or_else(changed::<Text2dBounds>);

        app.add_systems(
            Update,
            (
                mesh_content_size.run_if(res::<Assets<Mesh>>().or_else(changed::<Mesh2dHandle>)),
                image_content_size.run_if(res::<Assets<Image>>().or_else(changed::<Handle<Image>>)),
                #[cfg(feature = "sprite_text")]
                text_content_size.run_if(text_cond),
            )
                .in_set(ContentSizedSet),
        );
    }
}

type QueryContentSize<'w, 's, T, F> = Query<
    'w,
    's,
    (&'static mut ContentSized, &'static T),
    (Or<(Added<ContentSized>, Changed<T>)>, F),
>;

fn mesh_content_size(
    meshes: Res<Assets<Mesh>>,
    mut query: QueryContentSize<Mesh2dHandle, Without<Handle<Image>>>,
) {
    for (mut content_size, mesh) in &mut query {
        let Some(mesh) = meshes.get(&mesh.0) else {
            continue;
        };
        // TODO(perf): re-use AABB if present on entity
        let Some(aabb) = mesh.compute_aabb() else {
            continue;
        };
        let size = aabb.half_extents.xy() * 2.;
        content_size.0 = size.into();
    }
}

// TODO(bug): Account for `Sprite::custom_size`, and all sprite fields generally.
fn image_content_size(
    images: Res<Assets<Image>>,
    mut query: QueryContentSize<Handle<Image>, Without<Mesh2dHandle>>,
) {
    for (mut content_size, image) in &mut query {
        let Some(image) = images.get(image) else {
            continue;
        };
        let size = image.size();
        content_size.0.width = size.x;
        content_size.0.height = size.y;
    }
}

#[cfg(feature = "sprite_text")]
fn text_content_size(
    fonts: Res<Assets<Font>>,
    mut query: QueryContentSize<Text, (Without<Mesh2dHandle>, Without<Handle<Image>>)>,
) {
    for (mut content_size, text) in &mut query {
        let set_size = content_size.0;
        let bounds = Vec2::from(set_size);
        let measure = TextPipeline::default().create_text_measure(
            &fonts,
            &text.sections,
            // Seems like this requires an epsilon, otherwise text wraps poorly.
            1.01,
            text.alignment,
            text.linebreak_behavior,
        );
        if let Ok(measure) = measure {
            content_size.0 = measure.compute_size(bounds).into();
        }
    }
}
