use bevy::ecs::query::WorldQuery;
use bevy::prelude::{Plugin as BevyPlugin, *};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct Swatch {
    swatch: &'static MenuSwatch,
    current: &'static mut SwatchBuilder,
}
impl SwatchItem<'_> {
    /// Set the index of the child to show.
    ///
    /// # Panics
    /// If the index is out of bound.
    pub fn set_shown(&mut self, index: usize) {
        assert!(self.swatch.children.len() > index);
        self.current.0 = index;
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
struct MenuSwatch {
    children: Vec<Entity>,
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct SwatchBuilder(usize);
impl SwatchBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

fn build_swatch(
    mut cmds: Commands,
    to_build: Query<(Entity, &Children), (With<SwatchBuilder>, Without<MenuSwatch>)>,
    mut visibility: Query<&mut Visibility>,
) {
    for (entity, old_children) in &to_build {
        let to_hide = old_children.iter().skip(1);
        let mut to_hide = visibility.iter_many_mut(to_hide);
        while let Some(mut vis) = to_hide.fetch_next() {
            *vis = Visibility::Hidden;
        }
        let children = old_children.to_vec();
        cmds.entity(entity).insert(MenuSwatch { children }).remove_children(&old_children[1..]);
    }
}
// Note: we assume MenuSwatch.children is immutable.
fn update_swatch(
    changed: Query<(Entity, &Children, &MenuSwatch, &SwatchBuilder), Changed<SwatchBuilder>>,
    mut cmds: Commands,
    mut visibility: Query<&mut Visibility>,
) {
    for (entity, current_children, swatch, new_swatch) in &changed {
        let new_child = swatch.children[new_swatch.0];
        let old_child = *current_children.first().unwrap();
        if old_child == new_child {
            continue;
        }
        if let Ok(mut vis) = visibility.get_mut(old_child) {
            *vis = Visibility::Hidden;
        }
        if let Ok(mut vis) = visibility.get_mut(new_child) {
            *vis = Visibility::Visible;
        }
        cmds.entity(old_child).remove_parent();
        cmds.entity(new_child).set_parent(entity);
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MenuSwatch>()
            .register_type::<SwatchBuilder>()
            .add_systems(Last, build_swatch)
            .add_systems(Update, update_swatch);
    }
}
