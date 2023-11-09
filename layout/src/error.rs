use std::fmt;

use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::prelude::{Entity, Name, Query};
use bevy_mod_sysfail::FailureMode;
use thiserror::Error;

use crate::{direction::Axis, direction::Size, layout::Layout};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Computed {
    ChildDefined(f32, Entity),
    Valid(f32),
}
impl Computed {
    pub(crate) fn with_child(&self, child_size: f32) -> f32 {
        match self {
            // TODO: margin
            Self::ChildDefined(ratio, _) => *ratio * child_size,
            Self::Valid(size) => *size,
        }
    }
}
impl From<f32> for Computed {
    fn from(value: f32) -> Self {
        Self::Valid(value)
    }
}
impl fmt::Display for Computed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChildDefined(_, _) => fmt::Display::fmt("<child_size>", f),
            Self::Valid(value) => fmt::Display::fmt(value, f),
        }
    }
}

impl From<Size<f32>> for Size<Computed> {
    fn from(Size { width, height }: Size<f32>) -> Self {
        Self { width: width.into(), height: height.into() }
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
            .map_or(Self::Unnamed(entity), |name| Self::Named(name.clone()))
    }
    pub(crate) fn of(queries: &Layout<impl ReadOnlyWorldQuery>) -> Self {
        Self::of_entity(queries.this, queries.names)
    }
}
impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unnamed(entity) => write!(f, "<{entity:?}>"),
            Self::Named(name) => write!(f, "{name}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RelativeAxis {
    Main,
    Cross,
}

impl RelativeAxis {
    fn of(reference: Axis, axis: Axis) -> Self {
        match reference == axis {
            true => Self::Main,
            false => Self::Cross,
        }
    }
}

impl fmt::Display for RelativeAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Main => f.write_str("main"),
            Self::Cross => f.write_str("cross"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Relative {
    size: f32,
    axis: RelativeAxis,
    absolute: Axis,
}
impl Relative {
    pub(crate) fn of(reference: Axis, axis: Axis, size: f32) -> Self {
        Self {
            size,
            axis: RelativeAxis::of(reference, axis),
            absolute: reference,
        }
    }
}

impl fmt::Display for Relative {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.size > 0.5 {
            let larger = self.size > 1.0;
            write!(
                f,
                "- children have a total relative size on the parent's {} \
                axis of {:0}% of the parent's {}.{}",
                self.axis,
                self.size * 100.0,
                self.absolute,
                if larger { " This is larger than the parent!" } else { "" },
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Error)]
pub(crate) enum Why {
    #[error("Both axes of a `Root` container must be `Rule::Fixed`! {this}'s {axis} is not!")]
    InvalidRoot { this: Handle, axis: Axis },
    #[error(
        "{0}'s `Node` is a `Container`, yet it has no children! Use `Node::Box` or `Node::Axis` \
        for terminal nodes!"
    )]
    ChildlessContainer(Handle),
    #[error(
        "Cyclic rule definition detected!\n\
        - {this} depends on PARENT {parent} on {axis}\n\
        - {parent} depends on CHILD {this} on {axis}\n\
        It's impossible to make sense of this circular dependency!   \
        Use different rules on {axis} for {parent} or {this} to fix this issue."
    )]
    CyclicRule {
        this: Handle,
        parent: Handle,
        axis: Axis,
    },
    #[error(
        "Node {this}'s {axis} is overflowed by its children!\n\
        Notes:\n\
        - {this}'s inner size (excluding margins) is {size}\n\
        - There are {node_children_count} children of total {axis} {child_size}px.\n\
        - The largest child is {largest_child}\n\
        {child_relative_size}"
    )]
    ContainerOverflow {
        this: Handle,
        size: Size<f32>,
        largest_child: Handle,
        node_children_count: u32,
        axis: Axis,
        child_relative_size: Relative,
        child_size: f32,
    },
    #[error(
        "The margin of container {this} on axis {axis} has a negative value! ({margin}), \
        cuicui_layout doesn't support negative margins."
    )]
    NegativeMargin {
        this: Handle,
        axis: Axis,
        margin: f32,
    },
    #[error(
        "The margin of container {this} on axis {axis} is of {margin} pixels, \
        yet, {this} has a {axis} of {this_size} pixels! This would require \
        the content of {this} to have a negative size."
    )]
    TooMuchMargin {
        this: Handle,
        axis: Axis,
        margin: f32,
        this_size: f32,
    },
}

impl Why {
    pub(crate) fn bad_rule(
        axis: Axis,
        parent: Entity,
        queries: &Layout<impl ReadOnlyWorldQuery>,
    ) -> Self {
        Self::CyclicRule {
            this: Handle::of(queries),
            parent: Handle::of_entity(parent, queries.names),
            axis,
        }
    }

    pub(crate) fn invalid_root(axis: Axis, entity: Entity, names: &Query<&Name>) -> Self {
        Self::InvalidRoot { this: Handle::of_entity(entity, names), axis }
    }
}
/// An error caused by a bad layout.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct ComputeLayoutError(#[from] Why);

/// Uniquely identifies an error
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ErrorId {
    ChildlessContainer(Handle),
    CyclicRule(Handle),
    ContainerOverflow(Handle),
    NegativeMargin(Handle),
    InvalidRoot(Handle),
    TooMuchMargin(Handle),
}

impl FailureMode for ComputeLayoutError {
    type ID = ErrorId;

    fn identify(&self) -> Self::ID {
        match &self.0 {
            Why::ChildlessContainer(this) => ErrorId::ChildlessContainer(this.clone()),
            Why::CyclicRule { this, .. } => ErrorId::CyclicRule(this.clone()),
            Why::ContainerOverflow { this, .. } => ErrorId::ContainerOverflow(this.clone()),
            Why::NegativeMargin { this, .. } => ErrorId::NegativeMargin(this.clone()),
            Why::InvalidRoot { this, .. } => ErrorId::InvalidRoot(this.clone()),
            Why::TooMuchMargin { this, .. } => ErrorId::TooMuchMargin(this.clone()),
        }
    }
}
