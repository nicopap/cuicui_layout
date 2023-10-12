//! Vendored [`bevy_mod_component_mirror`], with explicit splitting of To and From logic.
//!
//! [`bevy_mod_component_mirror`]: https://github.com/devildahu/bevy_mod_component_mirror
#![allow(clippy::type_repetition_in_bounds)]
use std::marker::PhantomData;

use bevy::{prelude::*, reflect::GetTypeRegistration};

pub trait FromMirror<T> {
    fn to_target(&self) -> T;
}
pub trait ToMirror<T>: for<'a> From<&'a T> {}

impl<U, T> FromMirror<T> for U
where
    for<'a> &'a U: Into<T>,
{
    fn to_target(&self) -> T {
        self.into()
    }
}
impl<U: for<'a> From<&'a T>, T> ToMirror<T> for U {}

fn add_orphaned_mirror<T: Component, U: FromMirror<T> + Component>(
    mut cmds: Commands,
    orphan_mirrors: Query<(Entity, &U), Without<T>>,
) {
    for (entity, orphan) in &orphan_mirrors {
        cmds.entity(entity).insert(orphan.to_target());
    }
}

fn add_orphaned_target<T: Component, U: ToMirror<T> + Component>(
    mut cmds: Commands,
    orphan_targets: Query<(Entity, &T), Without<U>>,
) {
    for (entity, orphan) in &orphan_targets {
        cmds.entity(entity).insert(U::from(orphan));
    }
}

fn update_changed_mirror<T: Component, U: FromMirror<T> + Component>(
    mut changed: Query<(&mut T, &U), Changed<U>>,
) {
    for (mut to_update, changed) in &mut changed {
        *to_update = changed.to_target();
    }
}

fn update_changed_target<T: Component, U: ToMirror<T> + Component>(
    mut updated: Query<(&T, &mut U), Changed<T>>,
) {
    for (updated, mut to_update) in &mut updated {
        *to_update = updated.into();
    }
}

/// Systems added by the [`MirrorPlugin`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum MirrorSystems {
    /// When the mirror component is updated, during [`First`].
    Update,
    /// When components get added to entites with the mirror component they
    /// mirror (if not already present), in [`Last`].
    Add,
}

pub enum ToPlugin {}
pub enum FromPlugin {}
pub enum BiPlugin {}

pub struct MirrorPlugin<T, U, Dir = ()>(PhantomData<(T, U, Dir)>);

#[rustfmt::skip]
impl<T: Component, U: Component + GetTypeRegistration> MirrorPlugin<T, U> {
    #[must_use]
    pub fn new_from() -> MirrorPlugin<T, U, FromPlugin> where U: FromMirror<T> {
        MirrorPlugin(PhantomData)
    }
    #[must_use]
    pub fn new_to() -> MirrorPlugin<T, U, ToPlugin> where U: ToMirror<T> {
        MirrorPlugin(PhantomData)
    }
    #[must_use]
    pub fn new() -> MirrorPlugin<T, U, BiPlugin> where U: ToMirror<T> + FromMirror<T> {
        MirrorPlugin(PhantomData)
    }
}

impl<T: Component, U: Component> Plugin for MirrorPlugin<T, U, FromPlugin>
where
    U: FromMirror<T> + GetTypeRegistration,
{
    fn build(&self, app: &mut App) {
        app.register_type::<U>()
            .add_systems(Last, add_orphaned_mirror::<T, U>.in_set(MirrorSystems::Add))
            .add_systems(
                First,
                update_changed_mirror::<T, U>.in_set(MirrorSystems::Update),
            );
    }
}

impl<T: Component, U: Component> Plugin for MirrorPlugin<T, U, ToPlugin>
where
    U: ToMirror<T> + GetTypeRegistration,
{
    fn build(&self, app: &mut App) {
        app.register_type::<U>()
            .add_systems(Last, add_orphaned_target::<T, U>.in_set(MirrorSystems::Add))
            .add_systems(
                First,
                update_changed_target::<T, U>.in_set(MirrorSystems::Update),
            );
    }
}

impl<T: Component, U: Component> Plugin for MirrorPlugin<T, U, BiPlugin>
where
    U: FromMirror<T> + ToMirror<T> + GetTypeRegistration,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MirrorPlugin::<T, U>::new_from(),
            MirrorPlugin::<T, U>::new_to(),
        ));
    }
}
