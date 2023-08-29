use bevy::prelude::*;

use crate::BgColor;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Animation {
    pub intensity: f32,
}

// run_if:
// - resource_changed(OffsetActive)
// - any_offset_updated
#[allow(clippy::float_cmp, clippy::cast_possible_truncation)] // Of course clippy can't know the effect of `clamp`
pub(super) fn animate(mut offsets: Query<(&mut BgColor, &Animation)>, _: Res<Time>) {
    for (mut bg_color, animation) in &mut offsets {
        if bg_color.0.a() >= 1.0 {
            bg_color.0.set_a(animation.intensity);
        }
    }
}
