use bevy::prelude::*;

use crate::animate::AnimatedComponents;
use crate::colormix::color_lerp;
use crate::style::Fract;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub enum State {
    Shifted {
        initial_time: f64,
    },
    /// `.0` is last time this was shifted
    AtRest(f64),
}
impl Default for State {
    fn default() -> Self {
        Self::AtRest(0.)
    }
}
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Animation {
    pub rest_color: Color,
    pub active_color: Color,
    pub active_right_shift: u8,
    pub enable_speed: Fract,
    pub disable_speed: Fract,
}

// run_if:
// - resource_changed(OffsetActive)
// - any_offset_updated
#[allow(clippy::float_cmp, clippy::cast_possible_truncation)] // Of course clippy can't know the effect of `clamp`
pub(super) fn animate(
    mut offsets: Query<(AnimatedComponents, Ref<State>, Ref<Animation>)>,
    time: Res<Time>,
) {
    let current = time.elapsed_seconds_f64();
    for (components, state, animation) in &mut offsets {
        let Animation {
            rest_color,
            active_color,
            active_right_shift: active_left_shift,
            enable_speed,
            disable_speed,
        } = *animation;

        let (speed, initial_time, from_color, to_color) = match *state {
            State::Shifted { initial_time } => {
                (enable_speed, initial_time, rest_color, active_color)
            }
            State::AtRest(init) => (disable_speed, init, active_color, rest_color),
        };
        let speed = f64::from(speed);
        let lerp = ((current - initial_time) / speed).clamp(0., 1.);

        let requires_color = components.1.is_some() || components.2.is_some();
        let changed = state.is_changed() || animation.is_changed();
        let color = match () {
            () if lerp != 0. && lerp != 1. && requires_color => {
                color_lerp(from_color, to_color, lerp)
            }
            () if lerp == 1. && changed => to_color,
            () if lerp == 0. && changed => from_color,
            () => continue,
        };
        if let Some(mut ui_offset) = components.0 {
            let at_rest = matches!(*state, State::AtRest(_));
            let lerp = if at_rest { 1. - lerp } else { lerp };
            let x_offset = lerp as f32 * f32::from(active_left_shift);
            ui_offset.0 = Transform::from_xyz(x_offset, 0., 0.);
        }
        if let Some(mut bg_color) = components.2 {
            if components.1.is_none() {
                bg_color.0 = color;
            }
        }
        if let Some(mut text) = components.1 {
            text.sections[0].style.color = color;
        }
    }
}
