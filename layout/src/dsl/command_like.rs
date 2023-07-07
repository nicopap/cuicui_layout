use bevy::{
    ecs::system::EntityCommands,
    prelude::{BuildChildren, Bundle, ChildBuilder, Commands, Entity},
};

use super::LayoutCommands;

/// Can add `bundle`s and children to an [`Entity`] (Typically a [`EntityCommands`]).
pub trait CommandLike {
    /// Add given [`Bundle`] to the entity.
    fn insert(&mut self, bundle: impl Bundle);
    /// The [`Entity`] on which this [`CommandLike`] inserts bundles.
    fn entity(&self) -> Entity;
    /// Creates a [`ChildBuilder`] with the given children built in the given closure
    fn with_children(&mut self, f: impl FnOnce(&mut ChildBuilder));
}
/// Add methods to various command types to make it easier to spawn layouts.
pub trait IntoLayoutCommands<'w, 's, 'a> {
    /// Convert to [`super::LayoutCommands`].
    fn lyout(self) -> LayoutCommands<EntityCommands<'w, 's, 'a>>;
}
impl<'w, 's, 'a> CommandLike for EntityCommands<'w, 's, 'a> {
    fn insert(&mut self, bundle: impl Bundle) {
        self.insert(bundle);
    }
    fn entity(&self) -> Entity {
        self.id()
    }
    fn with_children(&mut self, f: impl FnOnce(&mut ChildBuilder)) {
        <EntityCommands as BuildChildren>::with_children(self, f);
    }
}
impl<'w, 's, 'a> IntoLayoutCommands<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn lyout(self) -> LayoutCommands<Self> {
        LayoutCommands::new(self)
    }
}

impl<'w, 's, 'a> IntoLayoutCommands<'w, 's, 'a> for &'a mut Commands<'w, 's> {
    fn lyout(self) -> LayoutCommands<EntityCommands<'w, 's, 'a>> {
        self.spawn_empty().lyout()
    }
}
impl<'w, 's, 'a> IntoLayoutCommands<'w, 's, 'a> for &'a mut ChildBuilder<'w, 's, '_> {
    fn lyout(self) -> LayoutCommands<EntityCommands<'w, 's, 'a>> {
        self.spawn_empty().lyout()
    }
}
