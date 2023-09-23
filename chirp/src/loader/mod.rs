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

// ## Jargon
//
// A "chirp seed" is an entity with a `Handle<Chirp>` component.
// We call it a seed 🌱 because it grows into a full bevy hierarchy tree 🌳
//
// ## Architecture
//
// Due to how poorly bevy handle scene hot reloading, we need to work around it.
//
// the `spawn` module defines 3 systems, all dedicated to managing `Handle<Chirp>`s
//
// 1. `update_asset_changed`: Reacts to asset event and orders reloading of spawned
//    chirp scenes. Note that it is more powerful than the `Scene` system, as it actually
//    works with hot reloading.
// 2. `update_marked`: Reacts to chirp instances changed through the [`ChirpState`] component.
// 3. `consume_seeds`: Reacts to `Entity` spawned with a `Handle<Chirp>`, request
//    to `SceneSpawner` that the chirp's scene be loaded into the world, add
//    the instance's metadata to [`ChirpInstances`], and when loading is completed,
//    re-parent & add chirp metadata to spawned scene entities.

use std::{any::type_name, marker::PhantomData, sync::Arc, sync::RwLock, sync::TryLockError};

use anyhow::Result;
use bevy::app::{App, Plugin as BevyPlugin, PostUpdate};
use bevy::asset::{prelude::*, AssetLoader, LoadContext};
use bevy::ecs::{prelude::*, schedule::ScheduleLabel, system::EntityCommands};
use bevy::log::{error, info};
use bevy::reflect::{TypeRegistryArc, TypeRegistryInternal as TypeRegistry};
use bevy::scene::scene_spawner_system;
use bevy::transform::TransformSystem;
use bevy::utils::get_short_name;
use thiserror::Error;

use crate::{Handles, ParseDsl};

pub use spawn::{Chirp, ChirpState};

mod internal;
#[cfg(feature = "debug")]
pub mod print_hierarchy;
// mod remove_ids;
mod scene;
pub(super) mod spawn;

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

/// Components necessary to load chirp files.
#[derive(Bundle)]
pub struct ChirpBundle {
    /// The load state of the chirp file.
    pub state: ChirpState,
    /// The chirp scene.
    pub scene: Handle<Chirp>,
}
impl ChirpBundle {
    /// Load a new chirp scene.
    #[must_use]
    pub fn new(scene: Handle<Chirp>) -> Self {
        ChirpBundle { state: ChirpState::Loading, scene }
    }
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
        function: impl Fn(&TypeRegistry, Option<&LoadContext>, &mut EntityCommands)
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
///
/// [`Scene`]: bevy::scene::Scene
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
            internal::Loader::<D>::new(load_context, &registry, &handles).load(bytes);
            drop(registry);
            let path = load_context.path().to_string_lossy();
            info!("Complete loading of chirp: {path}");
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["chirp"]
    }
}
/// The chirp loader plugin. Enables loading scene `.chirp` files with the
/// bevy [`AssetLoader`].
///
/// The loader is specific to the DSL. This is what the `D` is here for.
///
/// Hot reloading should work out of the box.
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
        // TODO(perf): Run-condition to avoid useless apply_deferred
        let chirp_asset_systems = (
            spawn::update_asset_changed,
            spawn::manage_chirp_state,
            spawn::spawn_chirps::<D>,
            // print_hierarchy::show_spawned,
        )
            .chain()
            .after(scene_spawner_system);

        let chirp_asset_systems = chirp_asset_systems.before(TransformSystem::TransformPropagate);
        #[cfg(feature = "bevy/bevy_ui")]
        let chirp_asset_systems = chirp_asset_systems
            .before(bevy::ui::UiSystem::Layout)
            .before(bevy::ui::UiSystem::Focus)
            .before(bevy::ui::UiSystem::Stack);
        app.add_systems(PostUpdate, chirp_asset_systems);
        app.add_asset::<Chirp>()
            .register_type::<ChirpState>()
            .init_asset_loader::<ChirpLoader<D>>();
    }
}
