//! Parse and split arguments provided from a [`ParseDsl`].
//!
//! The [`args`] module is used by the `parse_dsl_impl` macro when generating
//! [`ParseDsl`] implementation based on methods in an `impl` block.
//!
//! The [`split()`] function is used by the macro to separate arguments to a method.
//!
//! # Internal architecture
//!
//! The actual parser implementation for the `chirp` file format is private
//! and available in the `crate::grammar` module.
//!
//! The way parsed text gets interpreted is implemented in the [`crate::interpret`]
//! module.

use std::marker::PhantomData;

use anyhow::Result;
use bevy::asset::LoadContext;
use bevy::reflect::TypeRegistryInternal as TypeRegistry;
use cuicui_dsl::{BaseDsl, DslBundle};
use thiserror::Error;

pub use escape::escape_literal;
pub use split::split;

#[cfg(test)]
mod tests;

pub mod args;
mod escape;
pub mod split;

/// The input specification called a method not implemented in `D`.
///
/// Useful as a catchall when parsing a DSL calling an innexisting method.
#[derive(Debug, Error)]
#[error(
    "Didn't find method '{method}' for parse DSL of type '{}'",
    std::any::type_name::<D>(),
)]
pub struct DslParseError<D> {
    method: String,
    _ty: PhantomData<D>,
}
impl<D> DslParseError<D> {
    /// Create a [`DslParseError`] for `method` in `parse_type`.
    pub fn new(method: impl Into<String>) -> Self {
        Self { method: method.into(), _ty: PhantomData }
    }
}

/// Argument to [`ParseDsl::method`].
///
/// # Call format
///
/// When the method is called in the "bare" format, without arguments, such as:
/// ```text
/// Entity (bare_method)
/// ```
/// `args` will be the empty string.
///
/// Otherwise, `args` will include the surrouding parenthesis of the method call.
/// For example:
/// ```text
/// Entity (bare_method method1() method2(foobar) method3("foobar") method4(foo, bar))
/// ```
/// Will call [`ParseDsl::method`] with a `MethodCtx` with the following fields set:
///
/// |`name`|`bare_method`|`method1`| `method2`  | `method3`    | `method4`    |
/// |------|-------------|---------|------------|--------------|--------------|
/// |`args`|`''`         |`'()'`   |`'(foobar)'`|`'("foobar")'`|`'(foo, bar)'`|
///
///
/// # How to handle argument parsing
///
/// `cuicui_chirp` expects end-users to use the `parse_dsl_impl` macro or
/// [`ReflectDsl`] struct to take care of parsing for them.
///
/// A set of "blessed" parsers is predefined in the [`args`]
/// module. Those are the parsers used by default by `parse_dsl_impl`.
///
/// `ReflectDsl` uses the [`args::from_reflect`] and [`args::to_handle`]
/// parsers.
///
/// If you want to implement your own parsers, it is recommended that you follow
/// a similar syntax as the "native" syntax, and re-using the publicly-provided
/// parsers already used in `cuicui_chirp` is the best way to accomplish this.
///
/// To split arguments to methods between individual parameters, `parse_dsl_impl`
/// uses the [`split::split`] function. consider re-using it.
///
/// [`ReflectDsl`]: crate::ReflectDsl
pub struct MethodCtx<'a, 'l, 'll, 'r> {
    /// The method name.
    pub name: &'a str,
    /// The method arguments (notice **plural**).
    pub args: &'a str,
    /// The [`LoadContext`] used to load assets referenced in `chirp` files.
    pub ctx: Option<&'l mut LoadContext<'ll>>,
    /// The [`TypeRegistry`] the interpreter was initialized with.
    pub registry: &'r TypeRegistry,
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
        let MethodCtx { name, args, .. } = data;
        if name == "named" {
            self.named(args.to_string());
            Ok(())
        } else {
            // TODO(bug): Since it is the ultimate fallback, this won't help
            // probably need to add a few methods to ParseDsl.
            Err(DslParseError::<Self>::new(name).into())
        }
    }
}
