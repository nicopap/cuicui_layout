use std::any;
use std::marker::PhantomData;

use bevy::asset::{LoadContext, LoadedAsset};
use bevy::ecs::{prelude::*, query::QuerySingleError, system::EntityCommand};
use bevy::hierarchy::Parent;
use bevy::log::error;
use bevy::reflect::{Reflect, TypeRegistryArc, TypeRegistryInternal as TypeRegistry};
use bevy::scene::Scene;
use cuicui_dsl::EntityCommands;
use thiserror::Error;

use super::spawn::{self, InsertRoot};
use crate::{interpret, ChirpReader, Handles, ParseDsl};

#[derive(Debug, Error)]
pub enum RootInsertError {
    #[error(
        "The chirp file had 0 or more than 1 root entity, this should be impossible! \
        Consider reporting a bug to the cuicui repo!"
    )]
    NoSingleRoot(#[from] QuerySingleError),
    #[error(
        "The chirp DSL for {1} tries to spawn a component of type {0} that isn't \
        registered. Try registering it with `app.register_type::<{0}>()`"
    )]
    UnregisteredType(Box<str>, &'static str),
}

pub(super) struct Loader<'a, 'w, 'h, D> {
    ctx: &'a mut LoadContext<'w>,
    registry: TypeRegistryArc,
    handles: &'h Handles,
    _dsl: PhantomData<fn(D)>,
}

impl<'a, 'w, 'h, D: ParseDsl + 'static> Loader<'a, 'w, 'h, D> {
    pub(super) fn new(ctx: &'a mut LoadContext<'w>, reg: TypeRegistryArc, h: &'h Handles) -> Self {
        Self { ctx, registry: reg, handles: h, _dsl: PhantomData }
    }

    pub(super) fn load(&mut self, file: &[u8]) {
        let chirp = match self.load_scene(file) {
            Ok(scene) => {
                let root_cmd = match root_cmds::<D>(&self.registry.read(), &mut scene.world) {
                    Ok(cmd) => cmd,
                    Err(err) => {
                        error!("{err}");
                        return;
                    }
                };
                let scene = self.ctx.set_labeled_asset("Scene", LoadedAsset::new(scene));
                let root = Box::new(move |c| insert_root(&root_cmd, c));
                spawn::Chirp::Loaded { scene, root: InsertRoot(root) }
            }
            Err(errors) => {
                log_miette_error!(&errors);
                spawn::Chirp::Error(errors)
            }
        };
        self.ctx.set_default_asset(LoadedAsset::new(chirp));
    }
    fn load_scene(&mut self, file: &[u8]) -> Result<Scene, interpret::Errors> {
        let mut world = World::new();
        let mut chirp = ChirpReader::new(&mut world);
        let reg = self.registry.read();
        let result = chirp.interpret::<D>(self.handles, Some(self.ctx), &reg, file);
        result.map(|_| Scene::new(world))
    }
}

struct InsertReflect {
    components: Box<[Box<dyn Reflect>]>,
}
impl Clone for InsertReflect {
    fn clone(&self) -> Self {
        let components = self.components.iter().map(|reflect| reflect.clone_value());
        Self { components: components.collect() }
    }
}

impl EntityCommand for InsertReflect {
    fn apply(self, entity: Entity, world: &mut World) {
        world.resource_scope(|world, registry: Mut<AppTypeRegistry>| {
            let Some(mut entity) = world.get_entity_mut(entity) else {
                return;
            };
            let registry = registry.read();
            for component in self.components.iter() {
                let type_info = component.type_name();
                let Some(registration) = registry.get_with_name(type_info) else {
                    return;
                };
                let Some(component_data) = registration.data::<ReflectComponent>() else {
                    return;
                };
                component_data.insert(&mut entity, &**component);
            }
        });
    }
}

fn insert_root(insert_cmd: &InsertReflect, cmds: &mut EntityCommands) {
    cmds.add(insert_cmd.clone());
}

fn root_cmds<D>(
    type_registry: &TypeRegistry,
    source: &mut World,
) -> Result<InsertReflect, RootInsertError> {
    let root = source
        .query_filtered::<Entity, Without<Parent>>()
        .get_single(source)?;
    let archetype = source.entity(root).archetype();
    let get_info = |id| source.components().get_info(id);
    let mut components = Vec::new();
    for component_id in archetype.components() {
        let info = get_info(component_id).unwrap();
        let unregistered_type = || {
            let dsl = any::type_name::<D>();
            RootInsertError::UnregisteredType(info.name().into(), dsl)
        };
        let reflect_component = type_registry
            .get(info.type_id().unwrap())
            .ok_or_else(unregistered_type)?
            .data::<ReflectComponent>()
            .ok_or_else(unregistered_type)?;
        let w_entity = source.entity(root);
        // SAFETY: component is registered & entity contains component
        let source_component = unsafe { reflect_component.reflect(w_entity).unwrap_unchecked() };
        components.push(source_component.clone_value());
    }
    source.despawn(root);
    Ok(InsertReflect { components: components.into() })
}
