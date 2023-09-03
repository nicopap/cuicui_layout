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

#[derive(Debug, Error)]
pub enum ReloadError {
    #[error("This isn't a valid chirp seed, no chirp file were spawned for this entity: {0:?}")]
    NotAvailable(Entity),
    #[error("Cannot reload a not-yet-loaded chirp")]
    NotYetLoaded,
    #[error("When inserting the root entity")]
    Root(#[from] RootInsertError),
}

/// Controls loading and reloading of scenes with a hook.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub enum ChirpState {
    /// The scene's entites are not yet added to the `World`.
    #[default]
    Loading,
    /// The scene's entities are now in the `World` and its entities have the
    /// components added by the scene's [`Hook::hook`].
    Loaded,
    /// The scene's entities, whether they are its direct children or were
    /// unparented are to be despawned next time [`run_hooks`] runs, to be
    /// reloaded, running [`Hook::hook`] again.
    ///
    /// The spawned scene is loaded using [`Hook::file_path`].
    MustReload,
    /// The scene's entities, whether they are its direct children or were
    /// unparented are to be despawned next time [`run_hooks`] runs, the scene
    /// entity itself will also be deleted.
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
struct ChirpInstance {
    id: InstanceId,
}

pub struct InsertRoot(pub Box<dyn Fn(&mut EntityCommands) + Send + Sync + 'static>);
impl fmt::Debug for InsertRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InsertRoot")
    }
}

impl InsertRoot {
    fn insert(&self, cmds: &mut EntityCommands) {
        (self.0)(cmds);
        cmds.insert(ChirpLoaded);
        cmds.clear_children();
    }
}

/// A `Chirp` scene. It's just a bevy [`Scene`].
///
/// Unlike `Handle<Scene>`, `Handle<Chirp>` embeds inline the hierarchy of the scene,
/// so that all root entities become sibbling of the entity with a `Handle<Chirp>`.
/// Note that the `Handle<Chirp>` entity gets despawned once the scene is spawned.
///
/// You may keep around the `Entity` you used to spawn the chirp scene in order to
/// later refer to it in [`ChirpInstances`] and control reloading/deletion of
/// individual scene instances.
#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "b954f251-c38a-4ede-a7dd-cbf9856c84c1"]
pub enum Chirp {
    /// The chirp file loaded successfully and holds the given [`Scene`].
    Loaded {
        scene: Handle<Scene>,
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
    use AssetEvent::{Modified, Removed};
    for event in asset_events.iter() {
        for (mut state, instance_handle) in &mut chirp_instances {
            match event {
                AssetEvent::Created { .. } => {}
                Modified { handle } if handle == instance_handle => *state = ChirpState::MustReload,
                Removed { handle } if handle == instance_handle => *state = ChirpState::MustDelete,
                Modified { .. } | Removed { .. } => {}
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn consume_seeds(
    mut scene_spawner: ResMut<SceneSpawner>,
    mut cmds: Commands,
    chirps: Res<Assets<Chirp>>,
    no_parents: Query<Without<Parent>>,
    mut to_spawn: Query<
        (
            Entity,
            &mut ChirpState,
            &Handle<Chirp>,
            Option<&ChirpInstance>,
            Option<&Parent>,
            Has<ChirpSeedLogged>,
        ),
        Without<ChirpLoaded>,
    >,
) {
    for (chirp_id, mut state, handle, instance, parent, already_logged) in &mut to_spawn {
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
    scene_spawner: Res<SceneSpawner>,
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
