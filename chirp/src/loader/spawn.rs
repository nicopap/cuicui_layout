use bevy::asset::{Assets, Handle, HandleId};
use bevy::ecs::{prelude::*, query::Has, reflect::ReflectComponent};
use bevy::hierarchy::{BuildChildren, Parent};
use bevy::log::{error, warn};
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
pub struct Chirp(pub Handle<Scene>);

#[derive(Debug, Reflect, Component, Clone, Copy)]
#[reflect(Component)]
pub struct FromChirp(HandleId);
impl Default for FromChirp {
    fn default() -> Self {
        FromChirp(HandleId::default::<Chirp>())
    }
}

// TODO(bug)TODO(feat): React to `AssetEvent::Changed<Chirp>` it should
// indicate users trying to modify `Handle<Chirp>`, which means we need to update
// the actual scene, or the hot reloading system having updated the chirp
// based on file change.
#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub fn chirp_hook(
    to_spawn: Query<(
        Entity,
        &Handle<Chirp>,
        Option<&Parent>,
        Has<ChirpSeedLogged>,
    )>,
    chirps: Res<Assets<Chirp>>,
    mut chirp_instances: ResMut<ChirpInstances>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut cmds: Commands,
) {
    let ChirpInstances { instances, to_update } = &mut *chirp_instances;
    for chirp_id in to_update.drain(..) {
        let Some(instance) = instances.get_mut(&chirp_id) else {
            todo!("TODO(err): Not sure what can trigger this");
        };
        match instance.state {
            State::MustReload => {
                instance.state = State::Loading;

                for entity in scene_spawner.iter_instance_entities(instance.id) {
                    cmds.entity(entity).despawn();
                }
                let seed = cmds.spawn(instance.handle.clone()).id();
                if let Some(parent) = instance.parent {
                    cmds.entity(parent).add_child(seed);
                }
            }
            State::MustDelete => {
                for entity in scene_spawner.iter_instance_entities(instance.id) {
                    cmds.entity(entity).despawn();
                }
                instances.remove(&chirp_id);
            }
            State::Loading | State::Hooked => unreachable!(),
        }
    }
    for (entity, chirp, parent, already_logged) in &to_spawn {
        let Some(Chirp(scene)) = chirps.get(chirp) else {
            if !already_logged {
                cmds.entity(entity).insert(ChirpSeedLogged);
                warn!("TODO(err): chirp {entity:?} not yet loaded, skipping");
            }
            continue;
        };
        let instance = instances.entry(chirp.id()).or_insert_with(|| {
            let id = scene_spawner.spawn(scene.clone_weak());
            // TODO(bug): situations where the parent changes requires updating
            // this value (manual change after spawning
            // TODO(bug): only set parent the things that do not already have one,
            // otherwise we are flattening everything.
            let parent = parent.map(Parent::get);
            let handle = chirp.clone();
            ChirpInstance { id, parent, state: State::Loading, handle }
        });
        let is_ready = scene_spawner.instance_is_ready(instance.id);
        match instance.state {
            State::Loading if is_ready => {
                instance.state = State::Hooked;
                let from_chirp = FromChirp(chirp.id());
                cmds.entity(entity).despawn();

                let entities = scene_spawner.iter_instance_entities(instance.id);
                let add_from_chirp = entities
                    .map(|entity| {
                        if let Some(parent) = parent {
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
