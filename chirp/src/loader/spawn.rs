use std::fmt;

use bevy::asset::{AssetEvent, Assets, Handle};
use bevy::core::Name;
use bevy::ecs::{prelude::*, query::Has, reflect::ReflectComponent, system::EntityCommands};
use bevy::hierarchy::{BuildChildren, Parent};
use bevy::log::{debug, error, trace, warn};
use bevy::reflect::{Reflect, TypePath, TypeUuid};
use bevy::scene::{InstanceId, Scene, SceneSpawner};
use thiserror::Error;

use crate::interpret;

pub use super::internal::RootInsertError;

#[allow(missing_docs)] // allow: described by error message.
#[derive(Debug, Error)]
pub enum ReloadError {
    #[error("When inserting the root entity")]
    Root(#[from] RootInsertError),
}

/// Controls loading and reloading of [`Chirp`] scenes within the main bevy [`World`].
#[derive(PartialEq, Eq, Clone, Copy, Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub enum ChirpState {
    /// The scene's entites are not yet added to the `World`.
    #[default]
    Loading,
    /// The scene's entities are now in the `World`.
    Loaded,
    /// Reload the scene next time the internal `Chirp` scene management systems run.
    MustReload,
    /// Remove the scene from the world next time the internal `Chirp` scene
    /// management systems run.
    MustDelete,
    // TODO(feat): MustSave
    // Would need to iter not only the get_instance_entities, but children
    // as well.
}

/// Added to "chirp seed entities" (entites with a `Handle<Chirp>` component)
/// when logging an error related to them.
///
/// This isn't part of the public API, you shouldn't use this. Changes
/// to `ChirpSeedLogged` are considered non-breaking.
#[derive(Component)]
#[component(storage = "SparseSet")]
#[doc(hidden)]
pub struct ChirpSeedLogged;

/// Added to the root of loaded chirp scenes.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ChirpLoaded;

#[derive(Debug, Component)]
pub(super) struct ChirpInstance {
    id: InstanceId,
}

/// Insert components on the root entity of a [`Chirp`] scene.
///
/// This is needed in order to insert in-place scenes, so that they are fully
/// part of the hierarchy without meaningless intermediary entities.
///
/// This is an advanced API, as an end-user **you are not meant to use this**.
/// But it's exposed to help people build on top of `cuicui_chirp` if they
/// want to.
pub struct InsertRoot(pub(super) Box<dyn Fn(&mut EntityCommands) + Send + Sync + 'static>);
impl InsertRoot {
    pub(super) fn new(f: impl Fn(&mut EntityCommands) + Send + Sync + 'static) -> Self {
        InsertRoot(Box::new(f))
    }
    /// Insert the components stored in this [`InsertRoot`] on the provided entity.
    pub fn insert(&self, cmds: &mut EntityCommands) {
        (self.0)(cmds);
        cmds.insert(ChirpLoaded);
        cmds.clear_children();
    }
}
impl fmt::Debug for InsertRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InsertRoot")
    }
}

/// A `Chirp` scene. It's very close to a bevy [`Scene`].
///
/// Unlike `Handle<Scene>`, `Handle<Chirp>` embeds inline the hierarchy of the scene,
/// so that the entity with a `Handle<Chirp>` becomes the single root entity
/// declared in the scene.
///
/// The root entity, once the `Chirp` spawned — in addition to the scene's root
/// components — will have a [`ChirpState`] component added.
///
/// Modify this component to control the scene state. It can be used to reload
/// the scene or despawn the scene.
#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "b954f251-c38a-4ede-a7dd-cbf9856c84c1"]
pub enum Chirp {
    /// The chirp file loaded successfully and holds the given [`Scene`].
    Loaded {
        /// The scene handle, without the root of the chirp scene.
        scene: Handle<Scene>,
        /// An [`InsertRoot`] to update any pre-existing entity with the
        /// chirp scene root node's components.
        root: InsertRoot,
    },
    /// The chirp file failed to load with the given [`anyhow::Error`].
    ///
    /// Note: this exists because this enables us to use hot reloading even
    /// when loading the file failed.
    Error(interpret::Errors),
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn update_asset_changed(
    mut asset_events: EventReader<AssetEvent<Chirp>>,
    mut chirp_instances: Query<(&mut ChirpState, &Handle<Chirp>), With<ChirpLoaded>>,
) {
    use AssetEvent::{Created, Modified, Removed};
    #[allow(clippy::explicit_iter_loop)]
    for event in asset_events.iter() {
        for (mut state, instance_handle) in &mut chirp_instances {
            match event {
                Modified { handle } if handle == instance_handle => *state = ChirpState::MustReload,
                Removed { handle } if handle == instance_handle => *state = ChirpState::MustDelete,
                Created { .. } | Modified { .. } | Removed { .. } => {}
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn consume_seeds(
    mut scene_spawner: ResMut<SceneSpawner>,
    mut cmds: Commands,
    chirps: Res<Assets<Chirp>>,
    mut to_spawn: Query<
        (
            Entity,
            &mut ChirpState,
            &Handle<Chirp>,
            Option<&ChirpInstance>,
            Has<ChirpSeedLogged>,
        ),
        Without<ChirpLoaded>,
    >,
) {
    for (chirp_id, mut state, handle, instance, already_logged) in &mut to_spawn {
        let Some(Chirp::Loaded { scene, root }) = chirps.get(handle) else {
            if !already_logged {
                cmds.entity(chirp_id)
                    .insert((ChirpSeedLogged, Name::new("Loading Chirp Seed")));
                warn!("Chirp {chirp_id:?} not yet loaded, skipping");
            }
            continue;
        };
        trace!("Chirp scene loaded, spawning instance…");
        let instance_id = instance.map_or_else(
            || {
                let id = scene_spawner.spawn_as_child(scene.clone_weak(), chirp_id);
                cmds.entity(chirp_id).insert(ChirpInstance { id });
                id
            },
            |instance| instance.id,
        );
        let is_ready = scene_spawner.instance_is_ready(instance_id);
        match *state {
            ChirpState::Loading if is_ready => {
                trace!("Instace {chirp_id:?} is ready, hooking stuff up");
                *state = ChirpState::Loaded;
                root.insert(&mut cmds.entity(chirp_id));
            }
            ChirpState::Loading => {
                debug!("Instance {chirp_id:?} not yet ready, maybe next frame!");
                continue;
            }
            ChirpState::Loaded | ChirpState::MustReload | ChirpState::MustDelete => unreachable!(),
        }
    }
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn update_marked(
    mut to_update: Query<
        (
            Entity,
            &mut ChirpState,
            &Handle<Chirp>,
            &ChirpInstance,
            Option<&Parent>,
        ),
        Changed<ChirpState>,
    >,
    mut cmds: Commands,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    for (chirp_id, mut state, handle, instance, parent) in &mut to_update {
        match &*state {
            ChirpState::MustReload => {
                trace!("Reloading instance {chirp_id:?} marked as MustReload",);
                *state = ChirpState::Loading;

                cmds.entity(chirp_id).despawn();
                scene_spawner.despawn_instance(instance.id);
                cmds.insert_or_spawn_batch([(chirp_id, handle.clone())]);
                if let Some(parent) = parent {
                    cmds.entity(chirp_id).set_parent(parent.get());
                }
            }
            ChirpState::MustDelete => {
                trace!("Deleting instance {chirp_id:?} marked as MustDelete",);
                cmds.entity(chirp_id).despawn();
                scene_spawner.despawn_instance(instance.id);
            }
            // This system doesn't need to do anything in this situations, also
            // currently this should never happen.
            ChirpState::Loading | ChirpState::Loaded => {}
        }
    }
}
