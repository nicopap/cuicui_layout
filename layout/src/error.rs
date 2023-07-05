#![allow(clippy::module_name_repetitions)]
use std::fmt;

use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{Entity, Name, Query},
};
use bevy_mod_sysfail::FailureMode;
use thiserror::Error;

use crate::{direction::Size, layout::Layout};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Computed {
    ChildDefined(f32, Entity),
    Valid(f32),
}
impl Computed {
    pub(crate) fn with_child(&self, child_size: f32) -> f32 {
        match self {
            // TODO: margin
            Computed::ChildDefined(ratio, _) => *ratio * child_size,
            Computed::Valid(size) => *size,
        }
    }
}
impl From<f32> for Computed {
    fn from(value: f32) -> Self {
        Computed::Valid(value)
    }
}
impl fmt::Display for Computed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Computed::ChildDefined(_, _) => fmt::Display::fmt("<child_size>", f),
            Computed::Valid(value) => fmt::Display::fmt(value, f),
        }
    }
}

impl From<Size<f32>> for Size<Computed> {
    fn from(Size { width, height }: Size<f32>) -> Self {
        Size { width: width.into(), height: height.into() }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Handle {
    Unnamed(Entity),
    Named(Name),
}
impl Handle {
    pub(crate) fn of_entity(entity: Entity, names: &Query<&Name>) -> Self {
        names
            .get(entity)
            .map_or(Handle::Unnamed(entity), |name| Handle::Named(name.clone()))
    }
    pub(crate) fn of(queries: &Layout<impl ReadOnlyWorldQuery>) -> Self {
        Self::of_entity(queries.this, queries.names)
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
        "{0}'s `Node` is a `Container`, yet it has no children! Use `Node::Box` or `Node::Axis` \
        for terminal nodes!"
    )]
    ChildlessContainer(Handle),
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
        "Yo container {this} of size {size} contains more stuff than it possibly can!   \
         It has {node_children_count} items of total {dir_name} {child_size}.   \
         You gotta either make it larger or reduce the size of things within it."
    )]
    ContainerOverflow {
        this: Handle,
        size: Size<f32>,
        node_children_count: u32,
        dir_name: &'static str,
        child_size: f32,
    },
    #[error(
        "The margin of container {this} on axis {axis} has a negative value! ({margin}), \
        cuicui_layout doesn't support negative margins."
    )]
    NegativeMargin {
        this: Handle,
        axis: &'static str,
        margin: f32,
    },
    #[error(
        "The margin of container {this} on axis {axis} is of {margin} pixels, \
        yet, {this} has a {axis} of {this_size} pixels! This would require \
        the content of {this} to have a negative size."
    )]
    TooMuchMargin {
        this: Handle,
        axis: &'static str,
        margin: f32,
        this_size: f32,
    },
}

impl Why {
    pub(crate) fn bad_rule(
        axis: &'static str,
        parent: Entity,
        queries: &Layout<impl ReadOnlyWorldQuery>,
    ) -> Self {
        Why::ParentIsStretch {
            this: Handle::of(queries),
            parent: Handle::of_entity(parent, queries.names),
            axis,
        }
    }
}
/// An error caused by a bad layout.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct ComputeLayoutError(#[from] Why);

impl FailureMode for ComputeLayoutError {
    fn log_level(&self) -> bevy_mod_sysfail::LogLevel {
        bevy_mod_sysfail::LogLevel::Error
    }

    type ID = Handle;

    fn identify(&self) -> Self::ID {
        let (Why::ChildlessContainer(this)
        | Why::ParentIsStretch { this, .. }
        | Why::ContainerOverflow { this, .. }
        | Why::NegativeMargin { this, .. }
        | Why::TooMuchMargin { this, .. }) = &self.0;
        this.clone()
    }
    fn display(&self) -> Option<String> {
        Some(self.to_string())
    }
}
