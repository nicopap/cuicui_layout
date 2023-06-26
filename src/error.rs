use std::fmt;

use bevy::prelude::{Entity, Name, Query};
use bevy_mod_sysfail::FailureMode;
use thiserror::Error;

use crate::direction::{Oriented, Rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Bounds(pub(crate) Rect<Bound>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MaybeDirectionalBound {
    name: &'static str,
    bound: Bound,
}
impl Bounds {
    pub(crate) const fn on(&self, direction: Oriented) -> MaybeDirectionalBound {
        let name = direction.orient("width", "height");
        MaybeDirectionalBound { bound: self.0.on(direction), name }
    }
}
impl MaybeDirectionalBound {
    pub(crate) fn map(self, f: impl FnOnce(Bound) -> Bound) -> Self {
        Self { bound: f(self.bound), name: self.name }
    }
    pub(crate) fn why(self, this: Entity, names: &Query<&Name>) -> Result<f32, Why> {
        self.bound.why(self.name, this, names)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BadEntity(pub(crate) Entity);

pub(crate) type Bound = Result<f32, BadEntity>;

impl fmt::Display for Bounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.width {
            Ok(v) => write!(f, "{v}×")?,
            Err(_) => write!(f, "?×")?,
        };
        match self.0.height {
            Ok(v) => write!(f, "{v}"),
            Err(_) => write!(f, "?"),
        }
    }
}

pub(crate) trait ResultBadEntityExt<T> {
    fn why(self, name: &'static str, this: Entity, names: &Query<&Name>) -> Result<T, Why>;
}
impl<T> ResultBadEntityExt<T> for Result<T, BadEntity> {
    fn why(self, name: &'static str, this: Entity, names: &Query<&Name>) -> Result<T, Why> {
        self.map_err(|e| parent_is_stretch(name, this, e, names))
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(crate) enum Handle {
    Unnamed(Entity),
    Named(Name),
}
impl Handle {
    pub(crate) fn of(entity: Entity, names: &Query<&Name>) -> Self {
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
pub(crate) enum Why {
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
        bounds: Bounds,
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
pub(crate) fn parent_is_stretch(
    axis: &'static str,
    this: Entity,
    parent: BadEntity,
    query: &Query<&Name>,
) -> Why {
    Why::ParentIsStretch {
        this: Handle::of(this, query),
        parent: Handle::of(parent.0, query),
        axis,
    }
}
