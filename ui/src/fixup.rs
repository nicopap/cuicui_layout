//! Workaround some `bevy_text` components not being properly registered

use bevy::ecs::prelude::*;
use bevy::text::{Text, TextLayoutInfo};
use bevy::ui::widget::{TextFlags, UiImageSize};
use bevy::ui::{ContentSize, UiImage};
use bevy::utils::default;

/// Add [`TextLayoutInfo`] and [`TextFlags`] to UI text without such components.
///
/// Those components can't be spawned within a scene because they are not `Reflect`,
/// but we still need them to show text on screen.
///
/// This system is a workaround to add the components after-the-fact in the main
/// world.
#[allow(clippy::needless_pass_by_value)]
pub fn add_text_components(
    to_add: Query<
        Entity,
        (
            With<Text>,
            With<ContentSize>,
            Without<TextLayoutInfo>,
            Without<TextFlags>,
        ),
    >,
    mut cmds: Commands,
) {
    let to_add: Vec<(Entity, (TextLayoutInfo, TextFlags))> =
        to_add.iter().map(|e| (e, default())).collect();
    cmds.insert_or_spawn_batch(to_add);
}
/// This adds [`UiImageSize`] to UI image nodes.
#[allow(clippy::needless_pass_by_value)]
pub fn add_image_components(
    to_add: Query<Entity, (With<UiImage>, Without<ContentSize>, Without<UiImageSize>)>,
    mut cmds: Commands,
) {
    let to_add: Vec<(Entity, (ContentSize, UiImageSize))> =
        to_add.iter().map(|e| (e, default())).collect();
    cmds.insert_or_spawn_batch(to_add);
}
