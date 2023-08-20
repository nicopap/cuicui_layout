#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(
    clippy::use_self,
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate
)]

mod interpret;
pub mod parse;

use bevy::{
    ecs::system::SystemState,
    prelude::{error, Commands, World},
};

use interpret::Interpreter;

pub use anyhow;
#[cfg(feature = "macros")]
pub use cuicui_chirp_macros::parse_dsl_impl;
pub use interpret::{Handles, InterpError};
pub use parse::ParseDsl;
use winnow::BStr;

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
    pub fn interpret<D: ParseDsl>(&mut self, _handles: &Handles, input: &[u8]) {
        let mut state = SystemState::<Commands>::new(self.world);
        let mut cmds = state.get_mut(self.world);
        let mut input = BStr::new(input);
        if let Err(err) = Interpreter::new::<D>(&mut cmds).statements(&mut input) {
            error!("{err}");
        };
        state.apply(self.world);
    }
}
