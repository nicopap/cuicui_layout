use bevy::prelude::*;

use crate::animate::AnimatedComponents;
use crate::colormix::color_lerp;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Animation {
    pub offset: f64,
    pub period: f64,
    pub active_period: f64,
}

// run_if:
// - resource_changed(OffsetActive)
// - any_offset_updated
#[allow(clippy::float_cmp, clippy::cast_possible_truncation)] // Of course clippy can't know the effect of `clamp`
pub(super) fn animate(mut offsets: Query<(AnimatedComponents, &Animation)>, time: Res<Time>) {
    let current = time.elapsed_seconds_f64();
    for (components, animation) in &mut offsets {
        let Animation { offset, period, active_period } = *animation;
        let a_color = Color::WHITE;
        let b_color = Color::WHITE.with_a(0.0);

        let anim_offset = (current + offset) % period;
        let lerp = (anim_offset / active_period).clamp(0., 1.);
        let lerp = ((lerp - 0.5).abs() * 2.).clamp(0., 1.);
        let color = match () {
            () if lerp != 0. && lerp != 1. => color_lerp(a_color, b_color, lerp),
            () if lerp == 1. => b_color,
            () => a_color,
        };
        if let Some(mut text) = components.1 {
            text.sections[0].style.color = color;
        }
        if let Some(mut bg_color) = components.2 {
            bg_color.0 = color;
        }
    }
}
