use bevy::app::AppExit;
use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::input::gamepad::GamepadButtonChangedEvent;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_navigation::prelude::*;
use cuicui_examples::{switchable_impl, SwitchPlugin};

use crate::animate::button_shift as bshift;

switchable_impl! {
    RootButton[Roots, SwitchRoot],
    TabButton[Tabs, SwitchTab],
}

#[derive(WorldQuery)]
#[world_query(mutable)]
struct StateItem {
    children: Option<&'static Children>,
    state: &'static mut bshift::State,
}
impl StateItemItem<'_> {
    fn set_state<F: ReadOnlyWorldQuery>(
        &mut self,
        child_anim: &mut Query<&mut bshift::State, F>,
        new_state: bshift::State,
    ) {
        *self.state = new_state;
        let children = self.children.into_iter().flatten();
        let mut iter = child_anim.iter_many_mut(children);
        while let Some(mut anim) = iter.fetch_next() {
            *anim = new_state;
        }
    }
}

#[derive(Reflect, Resource, PartialEq, Debug, Default)]
#[reflect(Resource)]
enum InputState {
    Gamepad,
    #[default]
    Mouse,
}

fn change_input_state(
    mut state: ResMut<InputState>,
    mut mouse: EventReader<MouseMotion>,
    mut gp_buttons: EventReader<GamepadButtonChangedEvent>,
    mut gp_axis: EventReader<GamepadButtonChangedEvent>,
) {
    if mouse.iter().next().is_some() {
        state.set_if_neq(InputState::Mouse);
    }
    if gp_buttons.iter().next().is_some() | gp_axis.iter().next().is_some() {
        state.set_if_neq(InputState::Gamepad);
    }
}

// TODO(feat): highlight selected tab
fn highlight_focused(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut child_anim: Query<&mut bshift::State, Without<Focusable>>,
    mut focus_change: Query<(StateItem, &Focusable), Changed<Focusable>>,
) {
    use bshift::State::{AtRest, Shifted};
    use FocusState::{Active, Blocked, Focused, Inert, Prioritized};

    let initial_time = time.elapsed_seconds_f64();
    let gp_input = matches!(*input_state, InputState::Gamepad);
    for (mut anim, focusable) in &mut focus_change {
        match focusable.state() {
            Focused if !matches!(*anim.state, Shifted { .. }) && gp_input => {
                anim.set_state(&mut child_anim, Shifted { initial_time });
            }
            Active => {
                anim.set_state(&mut child_anim, Shifted { initial_time });
            }
            Prioritized | Blocked | Inert if !matches!(*anim.state, AtRest { .. }) => {
                anim.set_state(&mut child_anim, AtRest(initial_time));
            }
            _ => {}
        }
    }
}
fn highlight_hovered(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut child_anim: Query<&mut bshift::State, Without<Interaction>>,
    mut hover_change: Query<(StateItem, &Interaction), Changed<Interaction>>,
) {
    use bshift::State::{AtRest, Shifted};
    use Interaction::{Hovered, Pressed};

    let initial_time = time.elapsed_seconds_f64();
    if !matches!(*input_state, InputState::Mouse) {
        return;
    }
    for (mut anim, interaction) in &mut hover_change {
        match interaction {
            Pressed | Hovered if !matches!(*anim.state, Shifted { .. }) => {
                anim.set_state(&mut child_anim, Shifted { initial_time });
            }
            Interaction::None if !matches!(*anim.state, AtRest { .. }) => {
                anim.set_state(&mut child_anim, AtRest(initial_time));
            }
            _ => {}
        }
    }
}
fn clear_unused_input(
    time: Res<Time>,
    state: Res<InputState>,
    mut set: ParamSet<(
        (
            Query<(StateItem, &Interaction)>,
            Query<&mut bshift::State, Without<Interaction>>,
        ),
        (
            Query<(StateItem, &Focusable)>,
            Query<&mut bshift::State, Without<Focusable>>,
        ),
    )>,
) {
    use bshift::State::{AtRest, Shifted};
    use FocusState::Focused;
    use Interaction::{Hovered, Pressed};

    if !state.is_changed() {
        return;
    }
    let initial_time = time.elapsed_seconds_f64();
    if *state == InputState::Mouse {
        let (mut focusables, mut child_anim) = set.p1();
        for (mut anim, focus) in &mut focusables {
            let focused = matches!(focus.state(), Focused);
            let animated = matches!(anim.state.as_ref(), &Shifted { .. });
            if focused && animated {
                anim.set_state(&mut child_anim, AtRest(initial_time));
            }
        }
    } else {
        let (mut interactions, mut child_anim) = set.p0();
        for (mut anim, interaction) in &mut interactions {
            let pressed = matches!(interaction, Pressed | Hovered);
            let animated = matches!(anim.state.as_ref(), &Shifted { .. });
            if pressed && animated {
                anim.set_state(&mut child_anim, AtRest(initial_time));
            }
        }
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct QuitGame;

#[allow(clippy::explicit_iter_loop)] // bad lint: https://github.com/bevyengine/bevy/pull/9583
fn quit_game_button(
    mut nav_events: EventReader<NavEvent>,
    quit_buttons: Query<(), With<QuitGame>>,
    mut quit_events: EventWriter<AppExit>,
) {
    for cancel in nav_events.nav_iter().with_request(NavRequest::Action) {
        if quit_buttons.contains(cancel) {
            info!("QUIT GAME button has been pressed / GAME WILL BE ENDED");
            quit_events.send(AppExit);
        }
    }
}
#[allow(clippy::explicit_iter_loop)] // bad lint: https://github.com/bevyengine/bevy/pull/9583
fn switch_swatch(
    mut nav_events: EventReader<NavEvent>,
    mut switch_tab: EventWriter<SwitchTab>,
    mut switch_root: EventWriter<SwitchRoot>,
    switch_root_buttons: Query<&RootButton>,
    switch_tab_buttons: Query<&TabButton>,
) {
    for event in nav_events.iter() {
        let NavEvent::FocusChanged { to, .. } = event else {
            continue;
        };
        let (head, trail) = to.split_first();
        if matches!(switch_root_buttons.get(*head), Ok(RootButton(1))) {
            switch_root.send(SwitchRoot(0));
        }
        if let Some(button) = switch_root_buttons.iter_many(trail).next() {
            switch_root.send(SwitchRoot(button.0));
        }
        if let Some(button) = switch_tab_buttons.iter_many(trail).next() {
            switch_tab.send(SwitchTab(button.0));
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SwitchPlugin::<RootButton>::new(),
            SwitchPlugin::<TabButton>::new(),
        ))
        .init_resource::<InputState>()
        .register_type::<InputState>()
        .register_type::<QuitGame>()
        .add_systems(
            Update,
            (
                (quit_game_button, switch_swatch.after(NavRequestSystem)),
                (
                    change_input_state,
                    (highlight_focused.after(NavRequestSystem), highlight_hovered),
                    clear_unused_input,
                )
                    .chain(),
            ),
        );
    }
}
