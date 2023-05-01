use std::fmt;

use bevy::prelude::{Entity, Name, Query};
use bevy_mod_sysfail::FailureMode;
use thiserror::Error;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(super) enum Handle {
    Unnamed(Entity),
    Named(Name),
}
impl Handle {
    pub(super) fn of(entity: Entity, names: &Query<&Name>) -> Self {
        match names.get(entity) {
            Ok(name) => Handle::Named(name.clone()),
            Err(_) => Handle::Unnamed(entity),
        }
    }
}
impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Handle::Unnamed(entity) => write!(f, "<{entity:?}>"),
            Handle::Named(name) => write!(f, "{name}"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Error)]
pub(super) enum Why {
    #[error(
        "{this} needs to know its {axis}, \
        but {parent}, an ancestor of {this}, doesn't have a defined {axis}.   \
        Try specifying the {axis} of any container between {parent} and {this} \
        (included)"
    )]
    ParentIsStretch {
        this: Handle,
        parent: Handle,
        axis: &'static str,
        // TODO: include a "because Stretch/Ratio" explanation
    },
    #[error(
        "Yo container {this} of size {bounds} contains more stuff than it possibly can!   \
         It has {node_children_count} items of total {dir_name} {child_size}.   \
         You gotta either make it larger or reduce the size of things within it."
    )]
    ContainerOverflow {
        this: Handle,
        bounds: super::Bounds,
        node_children_count: u32,
        dir_name: &'static str,
        child_size: f32,
    },
}
impl FailureMode for Why {
    fn log_level(&self) -> bevy_mod_sysfail::LogLevel {
        bevy_mod_sysfail::LogLevel::Error
    }

    type ID = Handle;

    fn identify(&self) -> Self::ID {
        match self {
            Why::ParentIsStretch { this, .. } | Why::ContainerOverflow { this, .. } => this.clone(),
        }
    }
    fn display(&self) -> Option<String> {
        Some(self.to_string())
    }
}
pub(super) fn parent_is_stretch(
    axis: &'static str,
    this: Entity,
    parent: Entity,
    query: &Query<&Name>,
) -> Why {
    Why::ParentIsStretch {
        this: Handle::of(this, query),
        parent: Handle::of(parent, query),
        axis,
    }
}
