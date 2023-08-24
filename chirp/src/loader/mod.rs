//! Bevy [`AssetLoader`] for the chirp file format.
//!
//! Adds a Loader for the `.chirp` file format [`ChirpLoader`] and a global
//! "handles" registry [`WorldHandles`], accessible as a bevy [`Resource`].
//!
//! Handles are used for `code` statements in `.chirp` files.
//!
//! The [`crate::loader::Plugin`] defined in this module adds `ChirpLoader` as
//! an asset loader. Any [`Entity`] with a `Handle<Chirp>` **will be replaced**
//! by several entities, the one at the root of the `.chirp` file.

use std::{any::type_name, marker::PhantomData, sync::Arc, sync::RwLock, sync::TryLockError};

use anyhow::Result;
use bevy::app::{App, Plugin as BevyPlugin, PostUpdate};
use bevy::asset::{prelude::*, AssetLoader, LoadContext, LoadedAsset};
use bevy::ecs::{prelude::*, schedule::ScheduleLabel};
use bevy::log::error;
use bevy::reflect::{TypeRegistryArc, TypeRegistryInternal as TypeRegistry};
use bevy::scene::{scene_spawner_system, Scene};
use bevy::transform::TransformSystem;
use bevy::utils::get_short_name;
use thiserror::Error;

use crate::{interpret, ChirpReader, Handles, ParseDsl};
use spawn::{chirp_hook, Chirp, ChirpInstances};

pub(super) mod spawn;

struct InternalLoader<'a, 'w, 'h, 'r, D> {
    ctx: &'a mut LoadContext<'w>,
    registry: &'r TypeRegistry,
    handles: &'h Handles,
    _parse_dsl: PhantomData<fn(D)>,
}

/// Occurs when failing update the global chirp function registry [`WorldHandles`]
/// when [adding a function].
///
/// [adding a function]: WorldHandles::add_function
#[derive(Debug, Error)]
#[allow(missing_docs)] // Error messages already good documentation.
pub enum AddError {
    #[error("Failed to set function '{0}' in chirp handle registry: Lock poisoned")]
    Poisoned(String),
    #[error("Failed to set function '{0}' in chirp handle registry: Lock already taken")]
    WouldBlock(String),
}

/// Global [`ChirpLoader`] handle registry. Used in the `code` statements of the
/// chirp language.
#[derive(Resource)]
pub struct WorldHandles<D>(pub(crate) HandlesArc, PhantomData<fn(D)>);
type HandlesArc = Arc<RwLock<Handles>>;

impl<D> WorldHandles<D> {
    /// Associate `name` with `function` in `chirp` code statements.
    ///
    /// `function` may be called from a `chirp` file from a `code` statement if
    /// `name` is passed as argument.
    ///
    /// # Errors
    /// - When this operation would otherwise block (ie: a chirp file is loading)
    /// - When some other lock panicked.
    pub fn add_function(
        &mut self,
        name: String,
        function: impl Fn(&TypeRegistry, Option<&LoadContext>, &mut Commands, Option<Entity>)
            + Send
            + Sync
            + 'static,
    ) -> Result<(), AddError> {
        let mut handles = self.0.try_write().map_err(|err| match err {
            TryLockError::Poisoned(_) => AddError::Poisoned(name.clone()),
            TryLockError::WouldBlock => AddError::WouldBlock(name.clone()),
        })?;
        handles.add_function(name, function);
        drop(handles);
        Ok(())
    }
}

/// Loads a bevy [`Scene`] declared in a `chirp` file.
pub struct ChirpLoader<D> {
    registry: TypeRegistryArc,
    handles: HandlesArc,
    _dsl: PhantomData<fn(D)>,
}
impl<D: 'static> FromWorld for ChirpLoader<D> {
    fn from_world(world: &mut World) -> Self {
        let registry = world.resource::<AppTypeRegistry>().0.clone();
        let handles = HandlesArc::default();
        world.insert_resource(WorldHandles::<D>(Arc::clone(&handles), PhantomData));
        ChirpLoader { registry, handles, _dsl: PhantomData }
    }
}

impl<D: ParseDsl + 'static> AssetLoader for ChirpLoader<D> {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let registry = self.registry.internal.read();
            let Ok(handles) = self.handles.as_ref().read() else {
                let name = get_short_name(type_name::<D>());
                return Err(anyhow::anyhow!("Can't read handles in ChirpLoader<{name}>"));
            };
            InternalLoader::<D>::new(load_context, &registry, &handles).load(bytes)?;
            drop(registry);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["chirp"]
    }
}
impl<'a, 'w, 'h, 'r, D: ParseDsl + 'static> InternalLoader<'a, 'w, 'h, 'r, D> {
    fn new(ctx: &'a mut LoadContext<'w>, registry: &'r TypeRegistry, handles: &'h Handles) -> Self {
        Self { ctx, registry, _parse_dsl: PhantomData, handles }
    }

    fn load(&mut self, file: &[u8]) -> Result<(), interpret::Errors> {
        let scene = self.load_scene(file)?;
        let scene = self.ctx.set_labeled_asset("Scene", LoadedAsset::new(scene));
        self.ctx
            .set_default_asset(LoadedAsset::new(spawn::Chirp(scene)));
        Ok(())
    }
    fn load_scene(&mut self, file: &[u8]) -> Result<Scene, interpret::Errors> {
        let mut world = World::new();
        let mut chirp = ChirpReader::new(&mut world);
        let result = chirp.interpret::<D>(self.handles, Some(self.ctx), self.registry, file);
        if let Err(err) = &result {
            log_miette_error!(err);
        }
        result.map(|_| Scene::new(world))
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

/// The `SpawnChirp` schedule spawns chirp scenes between `Update` and `PostUpdate`.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpawnChirp;

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
        app.add_systems(
            PostUpdate,
            (chirp_hook.after(scene_spawner_system), apply_deferred)
                .chain()
                .before(TransformSystem::TransformPropagate),
        );
        app.init_resource::<ChirpInstances>()
            .add_asset::<Chirp>()
            .init_asset_loader::<ChirpLoader<D>>();
    }
}
