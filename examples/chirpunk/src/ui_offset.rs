use bevy::prelude::{Plugin as BevyPlugin, *};

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct UiOffset(pub Transform);

fn offset(mut query: Query<(&mut Transform, &UiOffset)>) {
    query.for_each_mut(|(mut transform, offset)| {
        *transform = transform.mul_transform(offset.0);
    });
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use bevy::transform::TransformSystem;
        use bevy::ui::UiSystem;

        app.add_systems(
            PostUpdate,
            offset.after(UiSystem::Layout).before(TransformSystem::TransformPropagate),
        );
    }
}
