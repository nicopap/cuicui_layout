use std::mem;

use bevy::asset::{AssetEvent, Assets, Handle};
use bevy::ecs::{prelude::*, reflect::ReflectComponent, system::SystemState};
use bevy::log::{error, trace};
use bevy::prelude::{Asset, Children};
use bevy::reflect::{Reflect, TypePath};
use bevy::scene::Scene;
use thiserror::Error;

use super::scene::{self, ChirpInstance};
use crate::interpret;

#[allow(missing_docs)] // allow: described by error message.
#[derive(Debug, Error)]
pub enum ReloadError {
    #[error("When inserting the root entity: {0}")]
    Root(#[from] scene::Error),
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
#[derive(Debug, TypePath, Asset)]
pub struct Chirp(pub(crate) Chirp_);

#[derive(Debug, TypePath)]
pub enum Chirp_ {
    /// The chirp file loaded successfully and holds the given [`Scene`].
    Loaded(Entity, Handle<Scene>),
    /// The chirp file failed to load with the given [`anyhow::Error`].
    ///
    /// Note: this exists because this enables us to use hot reloading even
    /// when loading the file failed.
    Error(interpret::Errors),
    LoadError,
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn update_asset_changed(
    mut asset_events: EventReader<AssetEvent<Chirp>>,
    mut chirp_instances: Query<(&mut ChirpState, &Handle<Chirp>), With<ChirpInstance>>,
) {
    use AssetEvent::{Added, LoadedWithDependencies, Modified, Removed};
    for event in asset_events.read() {
        for (mut state, instance_handle) in &mut chirp_instances {
            let instance_id = instance_handle.id();
            match event {
                Modified { id } if id == &instance_id => *state = ChirpState::MustReload,
                Removed { id } if id == &instance_id => *state = ChirpState::MustDelete,
                Added { .. } | Modified { .. } | Removed { .. } | LoadedWithDependencies { .. } => {
                }
            }
        }
    }
}

pub(super) struct SpawnRequest {
    target: Entity,
    source: Entity,
    scene_handle: Handle<Scene>,
}
type Chirps = (Entity, &'static mut ChirpState, &'static Handle<Chirp>);

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn spawn_chirps<D>(
    world: &mut World,
    mut to_load: Local<Vec<SpawnRequest>>,
    mut mark_state: Local<SystemState<(Res<Assets<Chirp>>, Query<Chirps, Without<ChirpInstance>>)>>,
) {
    to_load.extend(mark_loaded(mark_state.get_mut(world)));

    for SpawnRequest { target, source, scene_handle } in to_load.drain(..) {
        // A better impl would use `assets.remove(handle)` followed by `assets.insert` but currently
        // Assets::remove is broken, see: https://github.com/bevyengine/bevy/issues/10444
        let scene = change_scenes(world, |s| {
            Some(mem::take(&mut s.get_mut(&scene_handle)?.world))
        });
        let mut own_scene = Scene::new(scene.unwrap());

        let Some(instance) = spawn_scene::<D>(&mut own_scene, world, source, target) else {
            continue;
        };
        change_scenes(world, |s| {
            if let Some(scene) = s.get_mut(&scene_handle) {
                scene.world = own_scene.world;
            }
        });

        world.entity_mut(target).insert(instance);
    }
}

fn change_scenes<T>(world: &mut World, f: impl FnOnce(&mut Assets<Scene>) -> T) -> T {
    // SAFETY: we only call this function with a `Handle<Scene>` we got from same world.
    // Meaning there was a `Assets<Scene>`.
    let mut scenes = unsafe { world.get_resource_mut::<Assets<Scene>>().unwrap_unchecked() };

    // Bypass: we are NOT modifying the scene, we will be re-adding it in `reinsert_scene`
    f(scenes.bypass_change_detection())
}

fn spawn_scene<D>(
    scene: &mut Scene,
    target: &mut World,
    source_root: Entity,
    target_root: Entity,
) -> Option<ChirpInstance> {
    let type_registry = target.resource::<AppTypeRegistry>().clone();
    let type_registry = &*type_registry.read();
    // TODO(BUG): for some reasons the scene gets despawned with this, despite
    // the fact we make sure to avoid specifically dropping this (ie: the stash thing in scene.rs
    let handle = target.get::<Handle<Chirp>>(target_root).unwrap().clone();
    let instance =
        match scene::insert_on::<D>(type_registry, scene, target, source_root, target_root) {
            Ok(instance) => Some(instance),
            Err(err) => {
                error!("When spawning chirp file: {err}");
                None
            }
        };
    target.entity_mut(target_root).insert(handle);
    instance
}
// TODO(perf): Theoretically it _should_ be possible to implement this without cloning.
fn mark_loaded(
    (chirps, mut to_spawn): (Res<Assets<Chirp>>, Query<Chirps, Without<ChirpInstance>>),
) -> Vec<SpawnRequest> {
    let iter = to_spawn.iter_mut();
    let iter = iter.filter_map(|(target, mut state, handle)| {
        let Some(&Chirp(Chirp_::Loaded(source, ref scene))) = chirps.get(handle) else {
            return None;
        };
        matches!(*state, ChirpState::Loading).then(|| {
            trace!("Instance {target:?} is ready marking as loaded.");
            *state = ChirpState::Loaded;
            SpawnRequest { target, source, scene_handle: scene.clone() }
        })
    });
    iter.collect()
}

#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn manage_chirp_state(
    mut cmds: Commands,
    mut to_update: Query<(Chirps, &ChirpInstance), Changed<ChirpState>>,
) {
    for ((chirp_id, mut state, _), instance) in &mut to_update {
        match &*state {
            ChirpState::MustReload => {
                trace!("Reloading instance {chirp_id:?} marked as MustReload",);
                *state = ChirpState::Loading;

                // TODO(BUG): This also despawns the pre-existing components, which
                // is problematic.
                cmds.entity(chirp_id).remove::<(ChirpInstance, Children)>();
                instance.despawn_scene(chirp_id, &mut cmds);
            }
            ChirpState::MustDelete => {
                trace!("Deleting instance {chirp_id:?} marked as MustDelete",);
                instance.despawn_scene(chirp_id, &mut cmds);
                cmds.entity(chirp_id).despawn();
            }
            // This system doesn't need to do anything in this situations, also
            // currently this should never happen.
            ChirpState::Loading | ChirpState::Loaded => {}
        }
    }
}
