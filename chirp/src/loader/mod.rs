//! Bevy [`AssetLoader`] for the chirp file format.
use std::marker::PhantomData;

use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::{AddAsset, App, Plugin as BevyPlugin, World},
    scene::Scene,
    utils::HashMap,
};

use crate::{Chirp, ParseDsl};

struct InternalLoader<'a, 'w, D> {
    ctx: &'a mut LoadContext<'w>,
    _parse_dsl: PhantomData<fn(D)>,
}

/// Loads a bevy [`Scene`] declared with a
pub struct ChirpLoader<D>(PhantomData<fn(D)>);
impl<D> Default for ChirpLoader<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<D: ParseDsl + 'static> AssetLoader for ChirpLoader<D> {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut loader = InternalLoader::<D>::new(load_context);
            loader.load(bytes);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["chirp"]
    }
}
impl<'a, 'w, D: ParseDsl + 'static> InternalLoader<'a, 'w, D> {
    fn new(ctx: &'a mut LoadContext<'w>) -> Self {
        Self { ctx, _parse_dsl: PhantomData }
    }

    fn load(&mut self, file: &[u8]) {
        let scene = self.load_scene(file);
        self.ctx.set_default_asset(LoadedAsset::new(scene));
    }
    fn load_scene(&mut self, file: &[u8]) -> Scene {
        let mut world = World::new();
        let mut chirp = Chirp::new(&mut world);
        let handles = HashMap::new();
        chirp.interpret::<D>(&handles, file);
        Scene::new(world)
    }
}

/// The chirp loader plugin. Enables loading scene `.chirp` files with the
/// bevy [`AssetLoader`].
///
/// The loader is specific to the DSL. This is what the `D` is here for.
///
/// To get proper hot-reloading, consider wrapping the scene into a
/// [`bevy-scene-hook::reload::SceneHook`].
///
/// [`bevy-scene-hook::reload::SceneHook`]: https://docs.rs/bevy-scene-hook/latest/bevy_scene_hook/reload/index.html
pub struct Plugin<D>(PhantomData<fn(D)>);

impl Plugin<()> {
    /// Create a [`Plugin`] that load chirp files specified by the `D` [DSL].
    ///
    /// [DSL]: [cuicui_dsl::dsl]
    #[must_use]
    pub fn new<D: ParseDsl + 'static>() -> Plugin<D> {
        Plugin(PhantomData)
    }
}
impl<D: ParseDsl + 'static> BevyPlugin for Plugin<D> {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<ChirpLoader<D>>();
    }
}
