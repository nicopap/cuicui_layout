use std::fmt;

use bevy::ecs::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Children, Name, Parent};
use bevy_mod_sysfail::FailureMode;
use thiserror::Error;

use crate::{direction::Axis, direction::Size};

#[doc(hidden)]
#[derive(SystemParam)]
pub struct AnalyzeParam<'w, 's> {
    names: Query<'w, 's, &'static Name>,
    parents: Query<'w, 's, &'static Parent>,
    children: Query<'w, 's, &'static Children>,
}
impl<'w, 's> AnalyzeParam<'w, 's> {
    fn handle(&self, entity: Entity) -> Handle {
        let unnamed = Handle::Unnamed(entity);
        let maybe_entity = self.names.get(entity);
        maybe_entity.map_or(unnamed, |n| Handle::Named(n.as_str()))
    }
    pub(crate) fn analyze(&self, error: LayoutEntityError) {
        use LayoutEntityErrorKind as Kind;
        match error.error_kind {
            Kind::InvalidRoot => todo!(),
            Kind::ChildlessContainer => todo!(),
            Kind::CyclicRule => todo!(),
            Kind::ContainerOverflow => todo!(),
            Kind::NegativeMargin => todo!(),
            Kind::TooMuchMargin => todo!(),
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct LayoutEntityError {
    pub(crate) entity: Entity,
    pub(crate) error_kind: LayoutEntityErrorKind,
}
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub(crate) enum LayoutEntityErrorKind {
    InvalidRoot,
    ChildlessContainer,
    CyclicRule,
    ContainerOverflow,
    NegativeMargin,
    TooMuchMargin,
}
impl LayoutEntityErrorKind {
    pub(crate) fn on(self, entity: Entity) -> LayoutEntityError {
        LayoutEntityError { entity, error_kind: self }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Handle<'a> {
    Unnamed(Entity),
    Named(&'a str),
}
impl<'a> Handle<'a> {
    pub(crate) fn of_entity(entity: Entity, names: &'a Query<&Name>) -> Self {
        names
            .get(entity)
            .map_or(Self::Unnamed(entity), |name| Self::Named(name.as_str()))
    }
}
impl fmt::Display for Handle<'_> {
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
pub(crate) enum Why<'a> {
    #[error("Both axes of a `Root` container must be `Rule::Fixed`! {this}'s {axis} is not!")]
    InvalidRoot { this: Handle<'a>, axis: Axis },
    #[error(
        "{0}'s `Node` is a `Container`, yet it has no children! Use `Node::Box` or `Node::Axis` \
        for terminal nodes!"
    )]
    ChildlessContainer(Handle<'a>),
    #[error(
        "Cyclic rule definition detected!\n\
        - {this} depends on PARENT {parent} on {axis}\n\
        - {parent} depends on CHILD {this} on {axis}\n\
        It's impossible to make sense of this circular dependency!   \
        Use different rules on {axis} for {parent} or {this} to fix this issue."
    )]
    CyclicRule {
        this: Handle<'a>,
        parent: Handle<'a>,
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
        this: Handle<'a>,
        size: Size<f32>,
        largest_child: Handle<'a>,
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
        this: Handle<'a>,
        axis: Axis,
        margin: f32,
    },
    #[error(
        "The margin of container {this} on axis {axis} is of {margin} pixels, \
        yet, {this} has a {axis} of {this_size} pixels! This would require \
        the content of {this} to have a negative size."
    )]
    TooMuchMargin {
        this: Handle<'a>,
        axis: Axis,
        margin: f32,
        this_size: f32,
    },
}

impl FailureMode for LayoutEntityError {
    type ID = Self;

    fn identify(&self) -> Self::ID {
        *self
    }
}
