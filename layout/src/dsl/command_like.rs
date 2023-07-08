use bevy::{
    ecs::system::EntityCommands,
    prelude::{BuildChildren, ChildBuilder, Entity},
};

use crate::Size;

use super::{IntoUiBundle, UiBundle};

/// The type of insertion to do when calling [`MakeBundle::insert`].
///
/// You may want to implement different insertion logic for containers and
/// terminal UI nodes.
#[derive(Debug, Clone, Copy)]
pub enum InsertKind {
    /// Container UI node.
    Node,
    /// Terminal node.
    Leaf,
}

/// Can add `bundle`s and children to an [`Entity`] (Typically a [`EntityCommands`]).
pub trait MakeBundle: Default {
    /// Add given [`Bundle`](bevy::prelude::Bundle) to the entity.
    fn insert(self, insert: InsertKind, cmds: &mut EntityCommands) -> Entity;
    /// When spawning a leaf node, which axis are content-defined
    fn ui_content_axis(&self) -> Size<bool>;

    /// Spawn the entity as a container.
    fn node(self, mut cmds: EntityCommands, f: impl FnOnce(&mut ChildBuilder)) {
        self.insert(InsertKind::Node, &mut cmds);
        cmds.with_children(f);
    }

    /// Spawn the entity as a terminal UI node.
    fn spawn_ui<M>(self, mut cmds: EntityCommands, bundle: impl IntoUiBundle<M>) -> Entity {
        let content_defined = self.ui_content_axis();
        let target = self.insert(InsertKind::Leaf, &mut cmds);

        let mut bundle = bundle.into_ui_bundle();
        if content_defined.width {
            bundle.set_fixed_width();
        }
        if content_defined.height {
            bundle.set_fixed_height();
        }
        cmds.commands().entity(target).insert(bundle);
        target
    }
}
impl MakeBundle for () {
    fn insert(self, _: InsertKind, e: &mut EntityCommands) -> Entity {
        e.id()
    }
    fn ui_content_axis(&self) -> Size<bool> {
        Size::all(false)
    }
}

/// Convert this into an [`EntityCommands`].
///
/// This allows using the [`crate::layout!`] macro with common spawner types.
pub trait MakeSpawner<'w, 's, 'a>: Sized {
    /// Convert to [`EntityCommands`].
    fn make_spawner(self) -> EntityCommands<'w, 's, 'a>;
}
#[rustfmt::skip]
mod impls {
    use super::{MakeSpawner, ChildBuilder};
    use bevy::ecs::system::{EntityCommands, Commands};

    impl<'w, 's, 'a> MakeSpawner<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
        fn make_spawner(self) -> EntityCommands<'w, 's, 'a> { self } }
    impl<'w, 's, 'a> MakeSpawner<'w, 's, 'a> for &'a mut Commands<'w, 's> {
        fn make_spawner(self) -> EntityCommands<'w, 's, 'a> { self.spawn_empty() } }
    impl<'w, 's, 'a> MakeSpawner<'w, 's, 'a> for &'a mut ChildBuilder<'w, 's, '_> {
        fn make_spawner(self) -> EntityCommands<'w, 's, 'a> { self.spawn_empty() } }
}
