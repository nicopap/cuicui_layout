//! A modification of bevy's [`Scene::write_to_world_with`] to allow specifying
//! a root entity in the target world.

use std::any;
use std::collections::BTreeSet;

use bevy::ecs::component::ComponentId;
use bevy::ecs::prelude::*;
use bevy::ecs::{entity::EntityMap, query::QuerySingleError, reflect::ReflectMapEntities};
use bevy::log::{info, trace, warn};
use bevy::reflect::TypeRegistryInternal as TypeRegistry;
use bevy::scene::Scene;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "The chirp file had 0 or more than 1 root entity, this should be impossible! \
        Consider reporting a bug at https://github.com/nicopap/cuicui_layout/issues"
    )]
    NoSingleRoot(#[from] QuerySingleError),
    #[error(
        "The chirp DSL for {1} tries to spawn a component of type {0} that isn't \
        registered. Try registering it with `app.register_type::<{0}>()`"
    )]
    UnregisteredType(Box<str>, &'static str),
}

#[derive(Debug, Component, Default)]
#[component(storage = "SparseSet")]
pub(super) struct ChirpInstance {
    pub(super) map: EntityMap,
    pub(super) root_reserved: Box<[ComponentId]>,
}
impl ChirpInstance {
    pub(super) fn despawn_scene(&self, root: Entity, cmds: &mut Commands<'_, '_>) {
        for e in self.map.values().filter(|e| *e != root) {
            cmds.entity(e).despawn();
        }
    }
}

fn components(entity: Entity, world: &World) -> BTreeSet<ComponentId> {
    world.entity(entity).archetype().components().collect()
}

// 1. Track which components the target root has
// 2. when spawning scene:
//   1. Remove from target root all those components
//   2. Remove from that list all components that are overwritten by the source scene root
//   3. Apply `EntityMap`.
//   4. Insert back the target root components left on the list.
// 3. Store in the `ChirpInstace` that list
// 4. When respawning:
//   1. Iterate over `Archetype`'s component_id, removing the ones from the instance list
//   2. Use the innexisting `remove_by_ids` method to remove those TODO(BUG)¹

// ¹ bug: Say we insert a `NodeBundle`, now our root entity doesn't have a node bundle
// anymore, so we may be getting the "non-UI child of UI" panick, which sucks.

// We might want to track separately overwritten component and discard that information
// at the end of the scene spawning routine.
pub(super) fn insert_on<D>(
    reg: &TypeRegistry,
    source_scene: &mut Scene,
    target: &mut World,
    source_root: Entity,
    target_root: Entity,
) -> Result<ChirpInstance, Error> {
    info!("Stashing root components");
    let stash = stash_components(reg, target, &mut source_scene.world, target_root);

    let source = &source_scene.world;
    let get_info = |id| source.components().get_info(id);
    let dsl = any::type_name::<D>();
    let mut entity_map = EntityMap::default();
    entity_map.insert(source_root, target_root);

    trace!("Applying scene to target world");
    for archetype in source.archetypes().iter() {
        for s_entity in archetype.entities() {
            let s_entity = s_entity.entity();
            if s_entity == stash {
                trace!("skipping stashed entity");
                continue;
            }
            let w_entity = *entity_map
                .entry(s_entity)
                .or_insert_with(|| target.spawn_empty().id());

            for component_id in archetype.components() {
                let info = get_info(component_id).unwrap();
                let unregistered = || Error::UnregisteredType(info.name().into(), dsl);
                let entry = info
                    .type_id()
                    .map_or_else(|| reg.get_with_name(info.name()), |id| reg.get(id))
                    .ok_or_else(unregistered)?;

                let reflect_component =
                    entry.data::<ReflectComponent>().ok_or_else(unregistered)?;
                reflect_component.copy(source, target, s_entity, w_entity);
            }
        }
    }
    trace!("Applying entity map to target world");
    for entry in reg.iter() {
        if let Some(map) = entry.data::<ReflectMapEntities>() {
            map.map_all_entities(target, &mut entity_map);
        }
    }

    trace!("Overwritting changes to pre-existing components");
    let source = &mut source_scene.world;
    unstash_components(reg, target, source, target_root, stash);

    // TODO(BUG)
    let root_reserved = Vec::new().into();
    Ok(ChirpInstance { map: entity_map, root_reserved })
}
fn copy_components(
    reg: &TypeRegistry,
    source_world: &World,
    target_world: &mut World,
    from: Entity,
    to: Entity,
) {
    let get_id = |id, _: &mut _| source_world.components().get_info(id).unwrap();
    let entity = source_world.entity(from);
    for component_id in entity.archetype().components() {
        let info = get_id(component_id, target_world);
        let id = info.type_id().unwrap();
        trace!("{} {from:?}->{to:?}", info.name());
        let Some(entry) = reg.get(id) else {
            warn!("{} {from:?}->{to:?} not reflected", info.name());
            continue;
        };
        let Some(reflect_component) = entry.data::<ReflectComponent>() else {
            warn!("{} {from:?}->{to:?} not reflected (2)", info.name());
            continue;
        };
        reflect_component.copy(source_world, target_world, from, to);
    }
}
fn stash_components(
    reg: &TypeRegistry,
    world: &World,
    stash: &mut World,
    to_copy: Entity,
) -> Entity {
    let stash_entity = stash.spawn_empty().id();
    copy_components(reg, world, stash, to_copy, stash_entity);
    stash_entity
}
fn unstash_components(
    reg: &TypeRegistry,
    world: &mut World,
    stash: &mut World,
    target_entity: Entity,
    stash_entity: Entity,
) {
    copy_components(reg, stash, world, stash_entity, target_entity);
    stash.despawn(stash_entity);
}
