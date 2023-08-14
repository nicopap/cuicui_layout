//! Manage offseting text so that it doesn't overlap with other text.

use bevy::ecs::prelude::*;

#[derive(Component)]
pub(super) struct OverlayText;
