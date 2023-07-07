use bevy::{
    ecs::system::EntityCommands,
    prelude::{BuildChildren, Bundle, ChildBuilder, Commands, Entity},
};

/// Can add `bundle`s and children to an [`Entity`] (Typically a [`EntityCommands`]).
pub trait CommandLike {
    /// Add given [`Bundle`] to the entity.
    fn insert(&mut self, bundle: impl Bundle);
    /// The [`Entity`] on which this [`CommandLike`] inserts bundles.
    fn entity(&self) -> Entity;
    /// Creates a [`ChildBuilder`] with the given children built in the given closure
    fn with_children(&mut self, f: impl FnOnce(&mut ChildBuilder));
}
/// Something that can be converted into a [`CommandLike`], and therefore can
/// use the [`LayoutCommandsExt`] dsl.
///
/// [`LayoutCommandsExt`]: super::LayoutCommandsExt
pub trait IntoCommandLike {
    /// The target type.
    type Cmd: CommandLike;
    /// Convert to [`Self::Cmd`].
    fn into_cmd(self) -> Self::Cmd;
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
impl<'w, 's, 'a> IntoCommandLike for EntityCommands<'w, 's, 'a> {
    type Cmd = Self;
    fn into_cmd(self) -> Self::Cmd {
        self
    }
}

impl<'w, 's, 'a> IntoCommandLike for &'a mut Commands<'w, 's> {
    type Cmd = EntityCommands<'w, 's, 'a>;
    fn into_cmd(self) -> Self::Cmd {
        self.spawn_empty()
    }
}
impl<'w, 's, 'a> IntoCommandLike for &'a mut ChildBuilder<'w, 's, '_> {
    type Cmd = EntityCommands<'w, 's, 'a>;
    fn into_cmd(self) -> Self::Cmd {
        self.spawn_empty()
    }
}
