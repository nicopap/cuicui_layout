use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_navigation::prelude::{FocusState, Focusable, NavEvent, NavRequestSystem};

use crate::animate::button_shift;

#[derive(Reflect, PartialEq, Debug, Default)]
enum InputSource {
    Gamepad,
    #[default]
    Mouse,
}
#[derive(Reflect, Resource, Debug, Default)]
#[reflect(Resource)]
struct InputState {
    input: InputSource,
}
fn highlight_focused(
    time: Res<Time>,
    mut state: ResMut<InputState>,
    mut focus_change: Query<(&mut button_shift::State, &Focusable), Changed<Focusable>>,
) {
    use button_shift::State::{AtRest, Shifted};
    use FocusState::{Active, Blocked, Focused, Inert, Prioritized};

    let initial_time = time.elapsed_seconds_f64();
    for (mut anim, focusable) in &mut focus_change {
        match focusable.state() {
            Focused if !matches!(*anim, Shifted { .. }) => {
                if state.input != InputSource::Gamepad {
                    state.input = InputSource::Gamepad;
                }
                *anim = Shifted { initial_time };
            }
            Prioritized | Active | Blocked | Inert if !matches!(*anim, AtRest { .. }) => {
                if state.input != InputSource::Gamepad {
                    state.input = InputSource::Gamepad;
                }
                *anim = AtRest(initial_time);
            }
            _ => {}
        }
    }
}
fn highlight_hovered(
    time: Res<Time>,
    mut state: ResMut<InputState>,
    mut hover_change: Query<(&mut button_shift::State, &Interaction), Changed<Interaction>>,
) {
    use button_shift::State::{AtRest, Shifted};
    use Interaction::{Hovered, Pressed};

    let initial_time = time.elapsed_seconds_f64();
    for (mut anim, interaction) in &mut hover_change {
        if state.input != InputSource::Mouse {
            state.input = InputSource::Mouse;
        }

        match interaction {
            Pressed | Hovered if !matches!(*anim, Shifted { .. }) => {
                *anim = Shifted { initial_time };
            }
            Interaction::None if !matches!(*anim, AtRest { .. }) => {
                *anim = AtRest(initial_time);
            }
            _ => {}
        }
    }
}
fn clear_unused_input(
    time: Res<Time>,
    state: Res<InputState>,
    mut set: ParamSet<(
        Query<(&mut button_shift::State, &Interaction)>,
        Query<(&mut button_shift::State, &Focusable)>,
    )>,
) {
    use FocusState::Focused;
    use Interaction::{Hovered, Pressed};

    if !state.is_changed() {
        return;
    }
    let initial_time = time.elapsed_seconds_f64();
    if state.input == InputSource::Mouse {
        for (mut anim, focus) in &mut set.p1() {
            if focus.state() == Focused {
                *anim = button_shift::State::AtRest(initial_time);
            }
        }
    } else {
        for (mut anim, interaction) in &mut set.p0() {
            if matches!(interaction, Pressed | Hovered) {
                *anim = button_shift::State::AtRest(initial_time);
            }
        }
    }
}

#[allow(clippy::explicit_iter_loop)] // bad lint
fn print_events(mut nav_events: EventReader<NavEvent>) {
    for ev in nav_events.iter() {
        info!("{ev:?}");
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>();
        app.add_systems(Update, print_events.after(NavRequestSystem));
        app.register_type::<InputSource>().register_type::<InputState>().add_systems(
            Update,
            (
                (highlight_focused.after(NavRequestSystem), highlight_hovered),
                clear_unused_input,
            )
                .chain(),
        );
    }
}
