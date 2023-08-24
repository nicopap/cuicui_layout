#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(
    clippy::use_self,
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate
)]

macro_rules! log_miette_error {
    ($err: expr) => {
        #[cfg(feature = "fancy_errors")]
        let message = {
            let mut s = String::new();
            miette::GraphicalReportHandler::new()
                .render_report(&mut s, $err)
                .unwrap();
            s
        };
        #[cfg(not(feature = "fancy_errors"))]
        let message = $err;
        bevy::log::error!("{message}");
    };
}

pub mod interpret;
pub mod loader;
pub mod parse;
pub mod wrapparg;

use bevy::{
    asset::LoadContext,
    ecs::system::SystemState,
    prelude::{Commands, World},
    reflect::TypeRegistryInternal as TypeRegistry,
};

use interpret::Interpreter;

pub use anyhow;
#[cfg(feature = "macros")]
pub use cuicui_chirp_macros::parse_dsl_impl;
pub use interpret::{Handles, InterpError};
pub use parse::ParseDsl;

#[doc(hidden)]
pub mod bevy_types {
    pub use bevy::prelude::Entity;
}

/// Deserialized `dsl!` object.
///
/// Use [`Chirp::new`] to create a `Chirp` that will spawn stuff into the
/// provided [`World`]. Note that you may create a bevy `Scene` and pass the
/// `Scene`'s world instead of re-using the app world.
///
/// Use [`Chirp::interpret`] to interpret the `Chirp` file/text and add it to the
/// world.
pub struct Chirp<'a> {
    /// The scene read from the provided input.
    pub world: &'a mut World,
}
impl<'a> Chirp<'a> {
    /// Create a new `Chirp` that will write to the provided world.
    ///
    /// Note that you may create a temporary world instead of using the main
    /// app world.
    pub fn new(world: &'a mut World) -> Self {
        Self { world }
    }
    /// Create a [`Chirp`] from arbitrary byte slices.
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
        load_context: Option<&LoadContext>,
        registry: &TypeRegistry,
        input: &[u8],
    ) -> Result<(), interpret::Errors> {
        let mut state = SystemState::<Commands>::new(self.world);
        let mut cmds = state.get_mut(self.world);
        let mut interpreter = Interpreter::new::<D>(&mut cmds, load_context, registry, handles);
        let result = interpreter.interpret(input);
        if result.is_ok() {
            state.apply(self.world);
        }
        result
    }
    /// Same as [`Self::interpret`], but directly logs error message instead
    /// of returning the result.
    #[allow(clippy::missing_panics_doc)] // panics only on `fmt::write` errors.
    pub fn interpret_logging<D: ParseDsl>(
        &mut self,
        handles: &Handles,
        load_context: Option<&LoadContext>,
        registry: &TypeRegistry,
        input: &[u8],
    ) {
        let mut state = SystemState::<Commands>::new(self.world);
        let mut cmds = state.get_mut(self.world);
        let mut interpreter = Interpreter::new::<D>(&mut cmds, load_context, registry, handles);
        let result = interpreter.interpret(input);
        if let Err(err) = &result {
            log_miette_error!(err);
        }
        if result.is_ok() {
            state.apply(self.world);
        }
    }
}
