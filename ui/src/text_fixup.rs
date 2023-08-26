//! Workaround some `bevy_text` components not being properly registered

use bevy::ecs::prelude::*;
use bevy::text::{Text, TextLayoutInfo};
use bevy::ui::{widget::TextFlags, ContentSize};
use bevy::utils::default;

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
