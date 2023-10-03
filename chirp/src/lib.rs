/*!
[`cuicui_dsl`]: cuicui_dsl
[dsl-inheritance]: cuicui_dsl#inheritance
[`loader::Plugin`]: loader::Plugin
[`parse_dsl::args::Arguments`]: parse_dsl::args::Arguments
[`parse_dsl::args::from_reflect`]: parse_dsl::args::from_reflect
[`parse_dsl::args`]: parse_dsl::args
[`parse_dsl::args::quoted`]: parse_dsl::args::quoted
[`parse_dsl::args::to_handle`]: parse_dsl::args::to_handle
[`parse_dsl_impl`]: mod@parse_dsl_impl
[`parse_dsl_impl::delegate`]: parse_dsl_impl::delegate
[`ParseDsl`]: ParseDsl
[`ReflectDsl`]: reflect::ReflectDsl
[`Reflect`]: bevy::prelude::Reflect
[`WorldHandles`]: WorldHandles
*/
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
/// `impl` block macro to implement [`ParseDsl`].
///
/// See [the detailed documentation](mod@parse_dsl_impl).
#[cfg(feature = "macros")]
pub use cuicui_chirp_macros::parse_dsl_impl;
pub use interpret::{Handles, InterpError};
pub use load_asset::LoadAsset;
pub use loader::{Chirp, ChirpBundle, ChirpState, WorldHandles};
pub use parse_dsl::{MethodCtx, ParseDsl};
pub use reflect::ReflectDsl;

mod load_asset;
mod parser;

pub mod interpret;
pub mod loader;
pub mod parse_dsl;
pub mod reflect;

#[doc(hidden)]
#[cfg(feature = "test_and_doc")]
pub mod __doc_helpers {
    pub use cuicui_dsl::macros::__doc_helpers::*;

    impl<D: DslBundle> crate::ParseDsl for DocDsl<D> {
        fn method(&mut self, _: crate::MethodCtx) -> anyhow::Result<()> {
            Ok(())
        }
    }
    #[derive(Default, Component)]
    pub struct Camera2dBundle;
    #[derive(Default, Component)]
    pub struct LayoutRootCamera;
    pub mod cuicui_layout_bevy_ui {
        pub type UiDsl = super::DocDsl;
    }
}

/// Add this attribute to an `impl` block to generate a `ParseDsl` impl from
/// the methods declared in the `impl` block.
///
/// All methods with a `&mut self` argument
/// will automatically be added to the [`ParseDsl::method`] implementation.
///
/// > **Warning**
/// > For hot reloading to work, **avoid panicking in methods**. Otherwise
/// > the game will terminate without the ability to reload the chirp file.
///
/// This module is purely illustrative, it exports stub functions that
/// only exists for documentation.
///
/// This describes "meta-attributes", attributes that
/// can be added between parenthesis to the `parse_dsl_impl` attribute to modify
/// its behavior.
///
/// `parse_dsl_impl` also accepts a `parse_dsl` attribute on individual methods
/// within the `impl` block, see [`parse_dsl_impl::parse_dsl`] for details.
///
/// # Example
/// ```ignore
/// use cuicui_chirp::parse_dsl_impl;
///
/// #[parse_dsl_impl(
///     cuicui_chirp_path = ::cuicui_chirp,
///     delegate = inner,
///     type_parsers(color = css_color_parser),
///     set_params<>,
/// )]
/// impl MyDsl {
///     // This can be called in a chirp file in method position as `Entity(parse_dsl_bare_method)`.
///     pub fn parse_dsl_bare_method(&mut self) {}
///
///     // This can be called in a chirp file in method position as
///     // > Entity(with_args(51, "I think", "path/to.png"))
///     pub fn with_args(&mut self, ticks: u32, though: &str, image: Handle<Image>) {}
///
///     // Only methods with a `&mut self` can be called from a chirp file.
///     // This can't be called in a chirp file.
///     pub fn method_ignored_by_parse_dsl(&self, trick: u32) -> Option<String> { None }
///
///     // If you want to not add a `&mut self` method to the chirp methods, use
///     // `parse_dsl(ignore)`
///     #[parse_dsl(ignore)]
///     pub fn ignored_method(&mut self, style: &Style) {}
/// }
/// ```
#[cfg(feature = "macros")]
#[allow(
    unused_variables,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value
)]
pub mod parse_dsl_impl {
    use bevy::{asset::LoadContext, reflect::TypeRegistry, utils::HashMap};

    #[cfg(doc)]
    use crate::{parse_dsl::args, *};
    #[cfg(doc)]
    use cuicui_dsl::dsl;

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
    pub fn type_parsers<T, E, F>(overwrite_parsers: HashMap<Ident, F>)
    where
        E: Into<anyhow::Result<T>>,
        F: Fn(&TypeRegistry, Option<&LoadContext>, &str) -> E,
    {
    }
}

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
    ) -> Result<Entity, interpret::Errors> {
        let mut state = SystemState::<Commands>::new(self.world);
        let mut cmds = state.get_mut(self.world);
        let mut cmds = cmds.spawn_empty();
        let id = cmds.id();
        let result = Interpreter::interpret::<D>(input, &mut cmds, load_context, registry, handles);

        if result.is_ok() {
            state.apply(self.world);
        }
        result.map(|_| id)
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
        let result = Interpreter::interpret::<D>(input, &mut cmds, load_context, registry, handles);

        if let Err(err) = &result {
            log_miette_error!(err);
            false
        } else {
            state.apply(self.world);
            true
        }
    }
}
