//! Add this attribute to an `impl` block to generate a `ParseDsl` impl from
//! the methods declared in the `impl` block.
//!
//! All methods with a `&mut self` argument
//! will automatically be added to the [`ParseDsl::method`] implementation.
//!
//! # Example
//! ```ignore
//! use cuicui_chirp::parse_dsl_impl;
//!
//! #[parse_dsl_impl(
//!     cuicui_chirp_path = ::cuicui_chirp,
//!     delegate = inner,
//!     type_parsers(color = css_color_parser),
//!     set_params<>,
//! )]
//! impl MyDsl {
//!     // This can be called in a chirp file in method position as `Entity(parse_dsl_bare_method)`.
//!     pub fn parse_dsl_bare_method(&mut self) {}
//!
//!     // This can be called in a chirp file in method position as
//!     // > Entity(with_args(51, "I think", "path/to.png"))
//!     pub fn with_args(&mut self, ticks: u32, though: &str, image: Handle<Image>) {}
//!
//!     // Only methods with a `&mut self` can be called from a chirp file.
//!     // This can't be called in a chirp file.
//!     pub fn method_ignored_by_parse_dsl(&self, trick: u32) -> Option<String> { None }
//!
//!     // If you want to not add a `&mut self` method to the chirp methods, use
//!     // `parse_dsl(ignore)`
//!     #[parse_dsl(ignore)]
//!     pub fn ignored_method(&mut self, style: &Style) {}
//! }
//! ```
//!
//! # Notes
//!
//! > **Warning**
//! > For hot reloading to work, **avoid panicking in methods**. Otherwise
//! > the game will terminate without the ability to reload the chirp file.
//!
//! This module is purely illustrative, it exports stub functions that
//! only exists for documentation.
//!
//! This describes "meta-attributes", attributes that
//! can be added between parenthesis to the `parse_dsl_impl` attribute to modify
//! its behavior.
//!
//! `parse_dsl_impl` also accepts a `parse_dsl` attribute on individual methods
//! within the `impl` block, see [`parse_dsl_impl::parse_dsl`] for details.
//!
//! [`ParseDsl::method`]: crate::ParseDsl::method
//! [`parse_dsl_impl::parse_dsl`]: parse_dsl
#![allow(
    unused_variables,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value
)]

use bevy::{asset::LoadContext, reflect::TypeRegistry, utils::HashMap};

#[doc(hidden)]
pub struct Generics;
#[doc(hidden)]
pub struct Path;
#[doc(hidden)]
pub struct Ident;
#[doc(hidden)]
pub struct Ignore;

/// Do not add this method to the chirp methods list.
///
/// Note that any function other than `&mut self` methods are already ignored.
///
/// This is the only accepted `parse_dsl` meta-attribute.
///
/// # Example
/// ```ignore
/// use cuicui_chirp::parse_dsl_impl;
///
/// #[parse_dsl_impl]
/// impl MyDsl {
///     #[parse_dsl(ignore)]
///     fn to_ignore(&mut self) {}
///     // ...
/// # }
/// ```
pub fn parse_dsl(ignore: Ignore) {}

/// Specify which path to use for the `cuicui_chirp` crate.
///
/// **Default**: `::cuicui_chirp`
///
/// The default should work all the time if you are using [`cuicui_chirp`](crate)
/// directly.
///
/// However, if you are renaming the crate or re-exporting it, you need to
/// explicitly rename it.
///
/// # Example
/// ```ignore
/// use crate_reexporting::chirp::parse_dsl_impl;
///
/// #[parse_dsl_impl(cuicui_chirp_path = crate_reexporting::chirp)]
/// impl MyDsl {
///     // ...
/// # }
/// ```
pub fn cuicui_chirp_path(alternate_path: Path) {}

/// Field to delegate [`ParseDsl::method`] when encountering a method name not
/// in this `impl` block.
///
/// **Default**: None, no delegation occurs.
///
/// This is the same field that you mark with `#[deref_mut]` so that methods
/// are accessible in the [`dsl!`] macros.
///
/// # Example
/// ```ignore
/// use bevy::prelude::*;
/// use cuicui_chirp::parse_dsl_impl;
///
/// #[derive(Deref, DerefMut)]
/// struct MyDsl {
///     #[deref]
///     inner_dsl: OtherDsl,
///     // ...
/// }
/// #[parse_dsl_impl(delegate = inner_dsl)]
/// impl MyDsl {
///     // ...
/// # }
/// ```
///
/// [`ParseDsl::method`]: crate::ParseDsl::method
/// [`dsl!`]: cuicui_dsl::dsl
pub fn delegate(inner_field: Ident) {}

/// Use a custom set of type bounds on the `impl` blocks generics.
///
/// **Default**: The default is whatever the bounds are in the `impl` generic
/// declaration, plus `+ ParseDsl`.
///
/// If we have a `impl<T: PartialEq, U> MyDsl<T, U>`, the default bounds will be:
///
/// > `where T: PartialEq + ParseDsl, U: ParseDsl`
///
/// If you don't want `U` to be `ParseDsl`, you should use `set_params`.
///
/// # Example
/// ```ignore
/// use cuicui_chirp::{parse_dsl_impl, ParseDsl};
///
/// // We remove the `U: ParseDsl` by explicitly declaring the type bounds.
/// #[parse_dsl_impl(set_params <T: PartialEq + ParseDsl, U>)]
/// impl<T: PartialEq, U> MyDsl<T, U> {
///     // ...
/// # }
/// ```
pub fn set_params(explicit_trait_bounds: Generics) {}

/// Use `F` as parser for arguments of type `Ident`.
///
/// **Default**: `Handle<T> = args::to_handle, &str = args::quoted, _ = args::from_reflect`
///
/// Note that only identifiers are supported (yet), so you can't define custom
/// parsers for references or generic types.
///
/// It is a series of `key = value` pairs, `key` is the type which parser must
/// be customized, and `value` is the parser to use.
///
/// For parser, you can use [`args::to_handle`], [`args::quoted`], [`args::from_reflect`],
/// [`args::from_str`] or any function that implements:
///
/// ```rust,ignore
/// fn parse(
///     registry: &TypeRegistry,
///     ctx: Option<&LoadContext>,
///     input: &'a str,
/// ) -> Result<T, anyhow::Error>;
/// ```
///
/// # Defaults
///
/// By default, `parse_dsl_impl` uses the functions in the [`parse_dsl::args`] module.
///
/// - For `Handle<T>` and `&Handle<T>`, it uses [`args::to_handle`]
/// - For `&str`, it uses [`args::quoted`]
/// - For any other type, it uses [`args::from_reflect`].
///
/// # Example
/// ```ignore
/// use std::str::FromStr;
/// use css_color::parse_css_color;
/// use cuicui_chirp::parse_dsl_impl;
///
/// impl FromStr for Rule {
///   # type Err = (); fn from_str(s: &str) -> Result<Self, ()> {Err(())}
///   // ...
/// }
/// #[parse_dsl_impl(type_parsers(Color = parse_css_color, Rule = args::from_str))]
/// impl MyDsl {
///     pub fn color_and_rule(&mut self, color: Color, rule: Rule) {}
///     // ...
/// # }
/// ```
///
/// [`parse_dsl::args`]: crate::parse_dsl::args
/// [`args::to_handle`]: crate::parse_dsl::args::to_handle
/// [`args::quoted`]: crate::parse_dsl::args::quoted
/// [`args::from_str`]: crate::parse_dsl::args::from_str
/// [`args::from_reflect`]: crate::parse_dsl::args::from_reflect
pub fn type_parsers<T, E, F>(overwrite_parsers: HashMap<Ident, F>)
where
    E: Into<anyhow::Result<T>>,
    F: Fn(&TypeRegistry, Option<&LoadContext>, &str) -> E,
{
}
