#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery, missing_docs)]
#![allow(
    clippy::use_self,
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate
)]

mod interpret;
pub mod parse;

use std::str;

use bevy::prelude::BuildChildren;
use cuicui_dsl::EntityCommands;
use kdl::KdlDocument;

use interpret::DslInterpret;

pub use anyhow;
#[cfg(feature = "derive")]
pub use cuicui_format_derive::parse_dsl_impl;
pub use interpret::{Handles, InterpError};
pub use parse::{DslError, ParseDsl};

#[doc(hidden)]
pub mod bevy_types {
    pub use bevy::prelude::Entity;
}

/// Deserialized `dsl!` object.
///
/// Use [`DslFormat::parse`] to create a `DslFormat` from arbitrary byte slices.
///
/// Use [`DslFormat::interpret`] to interpret the `DslFormat` and add it to the
/// world with provided `cmds` as root.
pub struct DslFormat {
    document: KdlDocument,
}
impl DslFormat {
    /// Create a [`DslFormat`] from arbitrary byte slices.
    ///
    /// Currently, UTF-8 encoded KDL is expected.
    ///
    /// # Errors
    /// When parsing failed. See [`DslError`] for details.
    pub fn parse(input: &[u8]) -> Result<Self, DslError> {
        let input_utf8 = str::from_utf8(input)?;
        let document = input_utf8.parse()?;
        Ok(DslFormat { document })
    }
    /// Spawns UI according to KDL spec of this, using the `D` [`ParseDsl`].
    ///
    /// # Errors
    /// When interpretation failed. See [`InterpError`] for details.
    pub fn interpret<D: ParseDsl>(
        &self,
        mut cmds: EntityCommands,
        handles: &Handles,
    ) -> Result<(), InterpError> {
        let mut err = Ok(());
        cmds.with_children(|cmds| {
            err = DslInterpret::<D>::new(handles).statements(&self.document, cmds);
        });
        err
    }
}
