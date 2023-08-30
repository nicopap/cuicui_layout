use bevy::{
    app::AppExit,
    ecs::query::{ReadOnlyWorldQuery, WorldQuery},
    prelude::{Plugin as BevyPlugin, *},
};
use bevy_ui_navigation::prelude::*;

use crate::{animate::button_shift as bshift, show_menus::Swatch};

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

#[derive(Resource, Debug)]
struct Swatches {
    root: Entity,
    settings_submenu: Entity,
}

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

fn set_new_input_source(state: &mut ResMut<InputState>, new_source: InputSource) {
    if state.input != new_source {
        state.input = new_source;
    }
}

// TODO(feat): highlight selected tab
fn highlight_focused(
    time: Res<Time>,
    mut state: ResMut<InputState>,
    mut child_anim: Query<&mut bshift::State, Without<Focusable>>,
    mut focus_change: Query<(StateItem, &Focusable), Changed<Focusable>>,
) {
    use bshift::State::{AtRest, Shifted};
    use FocusState::{Active, Blocked, Focused, Inert, Prioritized};

    let initial_time = time.elapsed_seconds_f64();
    for (mut anim, focusable) in &mut focus_change {
        match focusable.state() {
            Focused | Active if !matches!(*anim.state, Shifted { .. }) => {
                set_new_input_source(&mut state, InputSource::Gamepad);
                anim.set_state(&mut child_anim, Shifted { initial_time });
            }
            Prioritized | Blocked | Inert if !matches!(*anim.state, AtRest { .. }) => {
                set_new_input_source(&mut state, InputSource::Gamepad);
                anim.set_state(&mut child_anim, AtRest(initial_time));
            }
            _ => {}
        }
    }
}
fn highlight_hovered(
    time: Res<Time>,
    mut state: ResMut<InputState>,
    mut child_anim: Query<&mut bshift::State, Without<Interaction>>,
    mut hover_change: Query<(StateItem, &Interaction), Changed<Interaction>>,
) {
    use bshift::State::{AtRest, Shifted};
    use Interaction::{Hovered, Pressed};

    let initial_time = time.elapsed_seconds_f64();
    for (mut anim, interaction) in &mut hover_change {
        set_new_input_source(&mut state, InputSource::Mouse);
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
    use FocusState::{Active, Focused};
    use Interaction::{Hovered, Pressed};

    if !state.is_changed() {
        return;
    }
    let initial_time = time.elapsed_seconds_f64();
    if state.input == InputSource::Mouse {
        let (mut focusables, mut child_anim) = set.p1();
        for (mut anim, focus) in &mut focusables {
            if matches!(focus.state(), Focused | Active) {
                anim.set_state(&mut child_anim, Shifted { initial_time });
            }
        }
    } else {
        let (mut interactions, mut child_anim) = set.p0();
        for (mut anim, interaction) in &mut interactions {
            if matches!(interaction, Pressed | Hovered) {
                anim.set_state(&mut child_anim, AtRest(initial_time));
            }
        }
    }
}

#[derive(Reflect, Default, Clone, Copy, Component, Debug)]
#[reflect(Component)]
pub enum SwatchMarker {
    #[default] // nonsensical, but required for ReflectComponent
    Root,
    SettingsSubmenu,
}

#[derive(Reflect, Default, Clone, Copy, Component, Debug)]
#[reflect(Component)]
pub enum SwatchTarget {
    #[default] // nonsensical, but required for ReflectComponent
    Root,
    Settings,
    Display,
    Sound,
    Tab3,
    Tab4,
}
impl SwatchTarget {
    // TODO(clean) bit hacky, basically hard-code the order.
    // This should get fixed with templating.
    const fn index(self) -> usize {
        use SwatchTarget::{Display, Root, Settings, Sound};
        match self {
            Root | Display => 0,
            Settings | Sound => 1,
            SwatchTarget::Tab3 => 2,
            SwatchTarget::Tab4 => 3,
        }
    }
}
fn update_swatches(
    mut cmds: Commands,
    swatches: Option<ResMut<Swatches>>,
    changed: Query<(Entity, &SwatchMarker), Changed<SwatchMarker>>,
) {
    if changed.is_empty() {
        return;
    }
    if let Some(mut swatches) = swatches {
        for (entity, swatch) in &changed {
            match swatch {
                SwatchMarker::Root => swatches.root = entity,
                SwatchMarker::SettingsSubmenu => swatches.settings_submenu = entity,
            }
        }
    } else {
        let placeholder = Entity::PLACEHOLDER;
        let mut swatches = Swatches { root: placeholder, settings_submenu: placeholder };
        for (entity, swatch) in &changed {
            match swatch {
                SwatchMarker::Root => swatches.root = entity,
                SwatchMarker::SettingsSubmenu => swatches.settings_submenu = entity,
            }
        }
        cmds.insert_resource(swatches);
    }
}

fn enable_swatch(
    target_path: &[Entity],
    swatches: &Swatches,
    swatches_query: &mut Query<Swatch>,
    swatch_targets: &Query<&SwatchTarget>,
) {
    use SwatchTarget::{Display, Root, Settings, Sound, Tab3, Tab4};

    if matches!(swatch_targets.get(target_path[0]), Ok(&Settings)) {
        let mut swatch = swatches_query.get_mut(swatches.root).unwrap();
        swatch.set_shown(0);
    }
    for target in swatch_targets.iter_many(&target_path[1..]) {
        let index = target.index();
        match target {
            Root | Settings => {
                let mut swatch = swatches_query.get_mut(swatches.root).unwrap();
                swatch.set_shown(index);
            }
            Display | Sound | Tab3 | Tab4 => {
                let mut swatch = swatches_query.get_mut(swatches.settings_submenu).unwrap();
                swatch.set_shown(index);
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
    for cancel in nav_events.nav_iter().with_request(NavRequest::Cancel) {
        if quit_buttons.contains(cancel) {
            info!("QUIT GAME button has been pressed / GAME WILL BE ENDED");
            quit_events.send(AppExit);
        }
    }
}
#[allow(clippy::explicit_iter_loop)] // bad lint: https://github.com/bevyengine/bevy/pull/9583
fn switch_swatch(
    mut nav_events: EventReader<NavEvent>,
    swatches: Res<Swatches>,
    mut swatches_query: Query<Swatch>,
    swatch_targets: Query<&SwatchTarget>,
) {
    for ev in nav_events.iter() {
        info!("event: {ev:?}");
        let NavEvent::FocusChanged { to, .. } = ev else {
            continue;
        };
        enable_swatch(to, &swatches, &mut swatches_query, &swatch_targets);
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .register_type::<SwatchTarget>()
            .register_type::<SwatchMarker>()
            .register_type::<InputSource>()
            .register_type::<InputState>()
            .register_type::<QuitGame>()
            .add_systems(
                Update,
                (
                    update_swatches,
                    quit_game_button,
                    switch_swatch.run_if(resource_exists::<Swatches>()).after(NavRequestSystem),
                    (
                        (highlight_focused.after(NavRequestSystem), highlight_hovered),
                        clear_unused_input,
                    )
                        .chain(),
                ),
            );
    }
}
