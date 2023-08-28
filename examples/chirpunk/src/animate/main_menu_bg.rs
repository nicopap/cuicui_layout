use bevy::prelude::*;

use crate::animate::AnimatedComponents;
use crate::colormix::color_lerp;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Animation {
    a_color: Color,
    b_color: Color,
    offset: f64,
    period: f64,
    active_period: f64,
}

// run_if:
// - resource_changed(OffsetActive)
// - any_offset_updated
#[allow(clippy::float_cmp, clippy::cast_possible_truncation)] // Of course clippy can't know the effect of `clamp`
pub(super) fn animate(mut offsets: Query<(AnimatedComponents, &Animation)>, time: Res<Time>) {
    let current = time.elapsed_seconds_f64();
    for (components, animation) in &mut offsets {
        let Animation { a_color, b_color, offset, period, active_period } = *animation;

        let anim_offset = (current + offset) % period;
        let lerp = (anim_offset / active_period).clamp(0., 1.);
        let lerp = ((lerp - 0.5).abs() * 2.).clamp(0., 1.);
        let color = match () {
            () if lerp != 0. && lerp != 1. => color_lerp(a_color, b_color, lerp),
            () if lerp == 1. => a_color,
            () => b_color,
        };
        if let Some(mut text) = components.1 {
            text.sections[0].style.color = color;
        }
        if let Some(mut bg_color) = components.2 {
            bg_color.0 = color;
        }
    }
}
