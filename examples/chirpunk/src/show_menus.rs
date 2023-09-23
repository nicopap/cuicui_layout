use bevy::prelude::{Plugin as BevyPlugin, *};

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Swatch(usize);
impl Swatch {
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the index of the child to show.
    ///
    /// # Panics
    /// If the index is out of bound.
    pub fn set_shown(&mut self, index: usize) {
        self.0 = index;
    }
}

fn update_swatch(
    changed: Query<(&Children, &Swatch), Changed<Swatch>>,
    mut visibility: Query<&mut Visibility>,
) {
    use Visibility::{Hidden, Inherited};

    for (children, new_swatch) in &changed {
        for (i, child) in children.iter().enumerate() {
            let Ok(mut vis) = visibility.get_mut(*child) else {
                continue;
            };
            *vis = if i == new_swatch.0 { Inherited } else { Hidden };
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Swatch>().add_systems(Update, update_swatch);
    }
}
