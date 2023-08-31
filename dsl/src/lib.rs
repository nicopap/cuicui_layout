/*!
[`dsl!`]: dsl
[`DslBundle`]: DslBundle
*/
#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(clippy::use_self, clippy::module_name_repetitions)]

use std::borrow::Cow;

use bevy::prelude::Entity;

pub use bevy::prelude::{BuildChildren, ChildBuilder};
pub use bevy::{core::Name, ecs::system::EntityCommands};

/// This exports the dummy impls we make to test the documentation on the macro.
#[doc(hidden)]
pub mod macros;

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
    fn node(&mut self, cmds: &mut EntityCommands, f: impl FnOnce(&mut ChildBuilder)) {
        let target_entity = self.insert(cmds);
        cmds.commands().entity(target_entity).with_children(f);
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
