#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(
    clippy::use_self,
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate
)]
// TODO(clean): move the parser to an independent crate.

macro_rules! log_miette_error {
    ($err: expr) => {
        #[cfg(feature = "fancy_errors")]
        let message = {
            let mut s = String::new();
            miette::GraphicalReportHandler::new()
                .with_context_lines(2)
                .with_width(90)
                .with_footer("\n".into())
                .render_report(&mut s, $err)
                .unwrap();
            s
        };
        #[cfg(not(feature = "fancy_errors"))]
        let message = $err;
        bevy::log::error!("{message:#}");
    };
}

use bevy::asset::LoadContext;
use bevy::ecs::{prelude::*, system::SystemState};
use bevy::reflect::TypeRegistryInternal as TypeRegistry;

use crate::interpret::Interpreter;

pub use anyhow;
#[cfg(feature = "macros")]
pub use cuicui_chirp_macros::parse_dsl_impl;
pub use interpret::{Handles, InterpError};
pub use load_asset::LoadAsset;
pub use loader::spawn::{Chirp, ChirpState};
pub use parse_dsl::ParseDsl;
pub use reflect::ReflectDsl;

mod load_asset;
mod parser;

pub mod interpret;
pub mod loader;
pub mod parse_dsl;
pub mod reflect;

#[doc(hidden)]
pub mod bevy_types {
    pub use bevy::prelude::Entity;
}

/// Deserialized `dsl!` object.
///
/// Use [`ChirpReader::new`] to create a `ChirpReader` that will spawn stuff into
/// the provided [`World`]. Note that you may create a bevy `Scene` and pass the
/// `Scene`'s world instead of re-using the app world.
///
/// Use [`ChirpReader::interpret`] to interpret the `Chirp` file/text and add it to the
/// world.
pub struct ChirpReader<'a> {
    /// The scene read from the provided input.
    pub world: &'a mut World,
}
impl<'a> ChirpReader<'a> {
    /// Create a new `ChirpReader` that will write to the provided world.
    ///
    /// Note that you may create a temporary world instead of using the main
    /// app world.
    pub fn new(world: &'a mut World) -> Self {
        Self { world }
    }
    /// Create a [`ChirpReader`] from arbitrary byte slices.
    ///
    /// This directly interprets the input as a chirp file and creates a bevy
    /// scene.
    ///
    /// # Errors
    /// If the input is an invalid `chirp` file. If this returns `Err`, then
    /// [`Self::world`] will be in an invalid partially-applied state.
    ///
    /// Possible errors include:
    /// - Invalid syntax
    /// - Calls a `code(handle)` where `handle` is not present in [`Handles`].
    /// - Errors returned by [`ParseDsl::method`] (usually parsing or invalid
    ///   method errors)
    ///
    /// The [`interpret::Errors`] implement [`miette::Diagnostic`] and lists
    /// **all interpretation errors** (either it stops at the first syntax
    // error or it tries to read and interpret the whole file)
    pub fn interpret<D: ParseDsl>(
        &mut self,
        handles: &Handles,
        load_context: Option<&mut LoadContext>,
        registry: &TypeRegistry,
        input: &[u8],
    ) -> Result<(), interpret::Errors> {
        let mut state = SystemState::<Commands>::new(self.world);
        let mut cmds = state.get_mut(self.world);
        let mut cmds = cmds.spawn_empty();
        let mut interpreter = Interpreter::new::<D>(&mut cmds, load_context, registry, handles);
        let result = interpreter.interpret(input);
        if result.is_ok() {
            state.apply(self.world);
        }
        result
    }
    /// Same as [`Self::interpret`], but directly logs error message instead
    /// of returning the result.
    ///
    /// Similarly to `interpret`, the world is in an invalid state if parsing
    /// fails. If this returns `true`, parsing succeeded, if this returns `false`,
    /// it failed.
    #[allow(clippy::missing_panics_doc)] // panics only on `fmt::write` errors.
    #[must_use]
    pub fn interpret_logging<D: ParseDsl>(
        &mut self,
        handles: &Handles,
        load_context: Option<&mut LoadContext>,
        registry: &TypeRegistry,
        input: &[u8],
    ) -> bool {
        let mut state = SystemState::<Commands>::new(self.world);
        let mut cmds = state.get_mut(self.world);
        let mut cmds = cmds.spawn_empty();
        let mut interpreter = Interpreter::new::<D>(&mut cmds, load_context, registry, handles);
        let result = interpreter.interpret(input);
        if let Err(err) = &result {
            log_miette_error!(err);
        }
        let ok = result.is_ok();
        if ok {
            state.apply(self.world);
        }
        ok
    }
}
