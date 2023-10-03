//! Parse and split arguments provided from a [`ParseDsl`].
//!
//! The [`args`] module is used by the `parse_dsl_impl` macro when generating
//! [`ParseDsl`] implementation based on methods in an `impl` block.
//!
//! The arguments as declared in the chirp file are passed to the methods as
//! the [`MethodCtx::arguments`] field. Check the [`MethodCtx`] docs for details.
//!
//! # Internal architecture
//!
//! The actual parser implementation for the `chirp` file format is private
//! and available in the `crate::grammar` module.
//!
//! This is the classic interpreter architecture of:
//!
//! > lexer → parser → AST → interpreter
//!
//! The way parsed text gets interpreted is implemented in the [`crate::interpret`]
//! module.

use anyhow::Result;
use bevy::asset::LoadContext;
use bevy::reflect::TypeRegistryInternal as TypeRegistry;
use cuicui_dsl::{BaseDsl, DslBundle};
use thiserror::Error;

pub use args::Arguments;
pub use escape::escape_literal;

mod escape;

pub mod args;

/// The input specification called a method that does not exist.
///
/// Useful as a catchall when parsing a DSL calling an innexisting method.
///
/// When encoutering this error, the interpreter uses the name span for error
/// reporting rather than the arguments span.
#[derive(Debug, Error)]
#[error("No '{method}' method")]
pub struct DslParseError {
    method: Box<str>,
}
impl DslParseError {
    /// Create a [`DslParseError`] for `method` in `parse_type`.
    pub fn new(method: impl Into<Box<str>>) -> Self {
        Self { method: method.into() }
    }
}

/// Context to run a method on [`ParseDsl::method`].
///
/// # Call format
///
/// `arguments` contain the arguments as parsed by `cuicui_chirp`. Parsing
/// removes comments and surounding spaces.
///
/// See the [`Arguments`] documentation for details.
pub struct MethodCtx<'i, 'c, 'cc> {
    // TODO(perf): Most likey could be a `[u8]` instead.
    /// The method name.
    pub name: &'i str,
    /// The method arguments.
    pub arguments: Arguments<'i, 'c>,
    /// The [`LoadContext`] used to load assets referenced in `chirp` files.
    pub ctx: Option<&'c mut LoadContext<'cc>>,
    /// The [`TypeRegistry`] the interpreter was initialized with.
    pub registry: &'c TypeRegistry,
    // TODO(perf): Consider re-using cuicui_fab::Binding
    // TODO(feat): bindings/references
}

/// A [`DslBundle`] that can be parsed.
pub trait ParseDsl: DslBundle {
    /// Apply method named `name` to `self`.
    ///
    /// # Calling format
    ///
    /// This is called with a [`MethodCtx`] argument. See the [`MethodCtx`]
    /// documentation for details as to what format to expect as argument.
    ///
    /// # Errors
    ///
    /// You may chose to fail for any reason, the expected failure case
    /// is failure to parse an argument in`ctx.args` or trying to call an
    /// innexisting method with `ctx.name`.
    ///
    /// [parent node]: cuicui_dsl::dsl#parent-node
    fn method(&mut self, ctx: MethodCtx) -> Result<()>;
}
impl ParseDsl for BaseDsl {
    fn method(&mut self, data: MethodCtx) -> Result<()> {
        let MethodCtx { name, arguments: args, .. } = data;
        if name == "named" {
            let name = args.get(0).unwrap();
            let str = String::from(String::from_utf8_lossy(name.as_ref()));
            self.named(str);
            Ok(())
        } else {
            Err(DslParseError::new(name).into())
        }
    }
}
