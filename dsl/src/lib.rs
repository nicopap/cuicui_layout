/*!
[`dsl!`]: dsl!
[`DslBundle`]: DslBundle
[`DslBundle::insert`]: DslBundle::insert
[`EntityCommands`]: EntityCommands
*/
#![doc = include_str!("../README.md")]

use std::borrow::Cow;

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
    fn insert(&mut self, cmds: &mut EntityCommands);

    /// Spawn the entity as a parent of other entities.
    fn node(&mut self, cmds: &mut EntityCommands, f: impl FnOnce(&mut ChildBuilder)) {
        self.insert(cmds);
        cmds.with_children(f);
    }
}
impl DslBundle for () {
    fn insert(&mut self, _: &mut EntityCommands) {}
}

impl DslBundle for BaseDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) {
        if let Some(name) = self.name.take() {
            cmds.insert(Name::new(name));
        }
    }
}
