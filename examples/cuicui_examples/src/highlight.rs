use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

type OnEnter = On<Pointer<Over>>;
type OnExit = On<Pointer<Out>>;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Highlight {
    highlight: Color,
    previous: Option<Color>,
}

impl Highlight {
    #[must_use]
    pub fn new(highlight: Color) -> Self {
        Self { highlight, previous: None }
    }
}

fn insert_events(mut cmds: Commands, query: Query<Entity, (With<Highlight>, Without<OnEnter>)>) {
    for entity in &query {
        cmds.entity(entity)
            .insert((OnEnter::run(highlight), OnExit::run(unhighlight)));
    }
}
fn highlight(
    event: Res<ListenerInput<Pointer<Over>>>,
    mut query: Query<(&mut Highlight, &mut BackgroundColor)>,
) {
    let Ok((mut high, mut bg)) = query.get_mut(event.listener()) else {
        return;
    };
    high.previous = Some(bg.0);
    bg.0 = high.highlight;
}
fn unhighlight(
    event: Res<ListenerInput<Pointer<Out>>>,
    mut query: Query<(&mut Highlight, &mut BackgroundColor)>,
) {
    let Ok((mut high, mut bg)) = query.get_mut(event.listener()) else {
        return;
    };
    let Some(prev) = high.previous else {
        return;
    };
    if high.highlight == bg.0 {
        bg.0 = prev;
    } else {
        high.previous = Some(bg.0);
    }
}

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Highlight>()
            .add_systems(Update, insert_events);
    }
}
