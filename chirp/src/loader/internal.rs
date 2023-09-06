use std::marker::PhantomData;

use bevy::asset::{LoadContext, LoadedAsset};
use bevy::ecs::prelude::*;
use bevy::reflect::TypeRegistryInternal as TypeRegistry;
use bevy::scene::Scene;

use super::spawn::Chirp;
use crate::{interpret, ChirpReader, Handles, ParseDsl};

pub(super) struct Loader<'a, 'r, 'w, 'h, D> {
    ctx: &'a mut LoadContext<'w>,
    registry: &'r TypeRegistry,
    handles: &'h Handles,
    _dsl: PhantomData<fn(D)>,
}

impl<'a, 'r, 'w, 'h, D: ParseDsl + 'static> Loader<'a, 'r, 'w, 'h, D> {
    pub(super) fn new(ctx: &'a mut LoadContext<'w>, reg: &'r TypeRegistry, h: &'h Handles) -> Self {
        Self { ctx, registry: reg, handles: h, _dsl: PhantomData }
    }

    pub(super) fn load(&mut self, file: &[u8]) {
        let load = LoadedAsset::new;

        let chirp = match self.load_scene(file) {
            Ok((root, scene)) => {
                Chirp::Loaded(root, self.ctx.set_labeled_asset("Scene", load(scene)))
            }
            Err(errors) => {
                log_miette_error!(&errors);
                Chirp::Error(errors)
            }
        };
        self.ctx.set_default_asset(LoadedAsset::new(chirp));
    }
    fn load_scene(&mut self, file: &[u8]) -> Result<(Entity, Scene), interpret::Errors> {
        let mut world = World::new();
        let mut chirp = ChirpReader::new(&mut world);
        let result = chirp.interpret::<D>(self.handles, Some(self.ctx), self.registry, file);
        result.map(|root| (root, Scene::new(world)))
    }
}
