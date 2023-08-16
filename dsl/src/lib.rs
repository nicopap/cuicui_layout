/*!
[`dsl!`]: dsl
[`DslBundle`]: DslBundle
*/
#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(clippy::use_self, clippy::module_name_repetitions)]

/// This exports the dummy impls we make to test the documentation on the macro.
#[doc(hidden)]
pub mod macros;

use bevy::prelude::{BuildChildren, ChildBuilder, Entity};
use std::borrow::Cow;

pub use bevy::{core::Name, ecs::system::EntityCommands};

/// Convert this into an [`EntityCommands`].
///
/// This allows using the [`dsl!`] macro with common spawner types.
pub trait IntoEntityCommands<'w, 's, 'a>: Sized {
    /// Convert to [`EntityCommands`].
    fn to_cmds(self) -> EntityCommands<'w, 's, 'a>;
}
#[rustfmt::skip]
mod impls {
    use super::{IntoEntityCommands, ChildBuilder};
    use bevy::ecs::system::{EntityCommands, Commands};

    impl<'w, 's, 'a> IntoEntityCommands<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
        fn to_cmds(self) -> EntityCommands<'w, 's, 'a> { self } }
    impl<'w, 's, 'a> IntoEntityCommands<'w, 's, 'a> for &'a mut Commands<'w, 's> {
        fn to_cmds(self) -> EntityCommands<'w, 's, 'a> { self.spawn_empty() } }
    impl<'w, 's, 'a> IntoEntityCommands<'w, 's, 'a> for &'a mut ChildBuilder<'w, 's, '_> {
        fn to_cmds(self) -> EntityCommands<'w, 's, 'a> { self.spawn_empty() } }
}

/// The base [`DslBundle`] for the [`crate::dsl!`] macro.
///
/// This only implements the [`BaseDsl::named`] method to easily give a name to your entities.
#[derive(Debug, Clone, Default)]
pub struct BaseDsl {
    /// The name to give to the `Entity`.
    pub name: Option<Cow<'static, str>>,
}
impl BaseDsl {
    /// Give name `name` to this statement's `Entity`.
    pub fn named(&mut self, name: impl Into<Cow<'static, str>>) {
        self.name = Some(name.into());
    }
}

/// The type used in a [`dsl!`] macro to create bundles to spawn when creating
/// the entity tree.
///
/// [`Default`] is used as the initial value for each entity.
pub trait DslBundle: Default {
    /// Add given [`Bundle`](bevy::prelude::Bundle) to the entity.
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity;

    /// Spawn the entity as a container.
    fn node(&mut self, mut cmds: EntityCommands, f: impl FnOnce(&mut ChildBuilder)) {
        self.insert(&mut cmds);
        cmds.with_children(f);
    }
}
impl DslBundle for () {
    fn insert(&mut self, e: &mut EntityCommands) -> Entity {
        e.id()
    }
}

impl DslBundle for BaseDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        if let Some(name) = self.name.take() {
            cmds.insert(Name::new(name));
        }
        cmds.id()
    }
}
