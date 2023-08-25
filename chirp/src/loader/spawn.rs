use bevy::asset::{Assets, Handle, HandleId};
use bevy::ecs::{prelude::*, query::Has, reflect::ReflectComponent};
use bevy::hierarchy::{BuildChildren, Parent};
use bevy::log::{error, trace, warn};
use bevy::prelude::AssetEvent;
use bevy::reflect::{Reflect, TypePath, TypeUuid};
use bevy::scene::{InstanceId, Scene, SceneSpawner};
use bevy::utils::HashMap;
use smallvec::SmallVec;

/// Controls loading and reloading of scenes with a hook.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum State {
    /// The scene's entites are not yet added to the `World`.
    Loading,
    /// The scene's entities are now in the `World` and its entities have the
    /// components added by the scene's [`Hook::hook`].
    Hooked,
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

struct ChirpInstance {
    id: InstanceId,
    parent: Option<Entity>,
    state: State,
    handle: Handle<Chirp>,
}

/// Control individual spawned chirp scenes.
#[derive(Resource, Default)]
pub struct ChirpInstances {
    // TODO(bug): if we want to spawn several instances of the same scene
    // HandleId is not the correct key to use…
    instances: HashMap<HandleId, ChirpInstance>,
    to_update: SmallVec<[HandleId; 2]>,
}
impl ChirpInstances {
    /// Schedule reload of provided `chirp` scene.
    ///
    /// The entities in the scene as when it was spawned will be removed and
    /// a new scene will be spawned with the same parent as the scene
    /// when it was spawned.
    pub fn set_reload(&mut self, chirp: &Handle<Chirp>) {
        trace!("Reloading: {chirp:?}");
        let id = chirp.id();
        let Some(chirp) = self.instances.get_mut(&id) else {
            error!("TODO(err): set_reload failed because instance does not exist");
            return;
        };
        self.to_update.push(id);
        chirp.state = State::MustReload;
    }
    /// set the provided chirp scene to be removed as soon as possible.
    ///
    /// The entities in the scene as when it was spawned will be removed.
    pub fn set_delete(&mut self, chirp: &Handle<Chirp>) {
        trace!("Deleting: {chirp:?}");
        let id = chirp.id();
        let Some(chirp) = self.instances.get_mut(&id) else {
            error!("TODO(err): set_delete failed because instance does not exist");
            return;
        };
        self.to_update.push(id);
        chirp.state = State::MustDelete;
    }
}

/// A `Chirp` scene. It's just a bevy [`Scene`].
///
/// Spawn a `Handle<Chirp>` to embed into the hierarchy a chirp scene.
#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "b954f251-c38a-4ede-a7dd-cbf9856c84c1"]
pub struct Chirp {
    /// The scene handle
    pub scene: Handle<Scene>,
    pub(crate) entity_count: usize,
}

#[derive(Debug, Reflect, Component, Clone, Copy)]
#[reflect(Component)]
pub struct FromChirp(HandleId);
impl Default for FromChirp {
    fn default() -> Self {
        FromChirp(HandleId::default::<Chirp>())
    }
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn update_asset_changed(
    mut asset_events: EventReader<AssetEvent<Chirp>>,
    mut chirp_instances: ResMut<ChirpInstances>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Created { .. } => {}
            AssetEvent::Modified { handle } => chirp_instances.set_reload(handle),
            AssetEvent::Removed { handle } => chirp_instances.set_delete(handle),
        }
    }
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn update_spawned(
    chirp_instances: ResMut<ChirpInstances>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut cmds: Commands,
    chirps: Res<Assets<Chirp>>,
    to_spawn: Query<(
        Entity,
        &Handle<Chirp>,
        Option<&Parent>,
        Has<ChirpSeedLogged>,
    )>,
    no_parents: Query<Without<Parent>>,
) {
    let mut instances = chirp_instances.map_unchanged(|i| &mut i.instances);
    for (entity, chirp, parent, already_logged) in &to_spawn {
        let Some(Chirp { scene, entity_count }) = chirps.get(chirp) else {
            if !already_logged {
                cmds.entity(entity).insert(ChirpSeedLogged);
                warn!("TODO(err): chirp {entity:?} not yet loaded, skipping");
            }
            continue;
        };
        trace!("Found chirp seed for chirp with {entity_count} entities");
        let instance = instances.entry(chirp.id()).or_insert_with(|| {
            let id = scene_spawner.spawn(scene.clone_weak());
            // TODO(bug): situations where the parent changes requires updating
            // this value (manual change after spawning
            let parent = parent.map(Parent::get);
            let handle = chirp.clone();
            ChirpInstance { id, parent, state: State::Loading, handle }
        });
        let is_ready = scene_spawner.instance_is_ready(instance.id);
        match instance.state {
            State::Loading if is_ready => {
                trace!(
                    "Instace {:?} is ready, hooking stuff up",
                    instance.handle.id()
                );
                instance.state = State::Hooked;
                let from_chirp = FromChirp(chirp.id());
                cmds.entity(entity).despawn();

                let entities = scene_spawner.iter_instance_entities(instance.id);
                let add_from_chirp = entities
                    .map(|entity| {
                        let scene_root = |_: &_| no_parents.contains(entity);
                        if let Some(parent) = parent.filter(scene_root) {
                            cmds.entity(entity).set_parent(parent.get());
                        };
                        (entity, from_chirp)
                    })
                    .collect::<Vec<_>>();
                cmds.insert_or_spawn_batch(add_from_chirp);
            }
            State::Loading => continue,
            State::Hooked | State::MustReload | State::MustDelete => unreachable!(),
        }
    }
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn update_marked(
    mut chirp_instances: ResMut<ChirpInstances>,
    mut cmds: Commands,
    scene_spawner: Res<SceneSpawner>,
) {
    let ChirpInstances { instances, to_update } = &mut *chirp_instances;
    for chirp_id in to_update.drain(..) {
        let Some(instance) = instances.get_mut(&chirp_id) else {
            todo!("TODO(err): Not sure what can trigger this");
        };
        match instance.state {
            State::MustReload => {
                trace!(
                    "Reloading instance {:?} marked as MustReload",
                    instance.handle.id()
                );
                instance.state = State::Loading;

                for entity in scene_spawner.iter_instance_entities(instance.id) {
                    cmds.entity(entity).despawn();
                }
                let mut seed = cmds.spawn(instance.handle.clone());
                if let Some(parent) = instance.parent {
                    seed.set_parent(parent);
                }
                instances.remove(&chirp_id);
            }
            State::MustDelete => {
                trace!(
                    "Deleting instance {:?} marked as MustDelete",
                    instance.handle.id()
                );
                for entity in scene_spawner.iter_instance_entities(instance.id) {
                    cmds.entity(entity).despawn();
                }
                instances.remove(&chirp_id);
            }
            State::Loading | State::Hooked => unreachable!(),
        }
    }
}
