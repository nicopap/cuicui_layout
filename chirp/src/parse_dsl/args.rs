//! Parse individual method arguments.
//!
//! [`parse_dsl_impl`] uses this module's functions to parse individual arguments passed
//! to methods in a chirp file. It is however possible to define and substitute your
//! own with the [`type_parsers`] meta-attribute.
//!
//! [`parse_dsl_impl`]: mod@crate::parse_dsl_impl
//! [`type_parsers`]: crate::parse_dsl_impl::type_parsers
#![allow(clippy::inline_always)]
// allow: rust has difficulties inlining functions cross-crate. Since we only
// use inline(always) on functions that are very small, it won't add significative
// compile overhead in anycase, but may help the optimizer elide some code.

use std::{any, borrow::Cow, convert::Infallible, io, marker::PhantomData, str, str::FromStr};

use bevy::asset::{Asset, Handle, LoadContext};
use bevy::reflect::erased_serde::__private::serde::de::DeserializeSeed;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::{FromReflect, Reflect, TypeRegistry};
use thiserror::Error;

use super::escape_literal;
use crate::load_asset::LoadAsset;
use crate::parser;

fn tyname<T>() -> &'static str {
    any::type_name::<T>()
}
/// Errors from [`to_handle`].
#[allow(missing_docs)] // Already documented by error message
#[derive(Debug, Error)]
pub enum HandleDslDeserError<T> {
    #[error(
        "Didn't provide a LoadContext when deserializing a 'Handle<{}>'. \
        This is required to enable loading assets",
        tyname::<T>(),
    )]
    NoLoadContext,
    #[error("Failed to load 'Handle<{}>' from file system", tyname::<T>())]
    FileIo(#[from] io::Error),
    #[error("Loading handles is not supported with non-FileSystem IO. It will be available starting bevy 0.12")]
    UnsupportedIo,
    #[error("Couldn't load 'Handle<{}>'", tyname::<T>())]
    BadLoad(anyhow::Error),
    #[doc(hidden)]
    #[error("==OPTIMIZEDOUT== This error never occurs")]
    _Ignore(PhantomData<fn(T)>, Infallible),
}

// NOTE: we use `&'static str` instead of storing the type as a generic parameter
// so that we can downcast to this error type in crate::interpret, in order to
// pick up the underlying error offset.
/// Error occuring in [`from_reflect`].
#[allow(missing_docs)] // Already documented by error message
#[derive(Debug, Error)]
pub enum ReflectDslDeserError {
    #[error("Tried to deserialize a DSL argument using reflection, yet '{0}' is not registered.")]
    NotRegistered(&'static str),
    #[error("Ron couldn't deserialize the DSL argument of type '{1}': {0}")]
    RonDeser(#[source] Box<ron::error::SpannedError>, &'static str),
    #[error(
        "The DSL argument of type '{0}' was parsed by bevy in RON, but the \
        generated reflect proxy type couldn't be converted into '{0}'"
    )]
    BadReflect(&'static str),
}

impl ReflectDslDeserError {
    fn ron_deser<T>(source: ron::error::SpannedError) -> Self {
        Self::RonDeser(Box::new(source), tyname::<T>())
    }
    fn not_registered<T>() -> Self {
        Self::NotRegistered(tyname::<T>())
    }
    fn bad_reflect<T>() -> Self {
        Self::BadReflect(tyname::<T>())
    }
    pub(crate) fn maybe_offset(&self) -> Option<u32> {
        match self {
            Self::BadReflect(_) | Self::NotRegistered(_) => None,
            Self::RonDeser(ron, _) => {
                (ron.position.line <= 1).then(|| u32::try_from(ron.position.col).unwrap())
            }
        }
    }
}

/// Error caused by an invalid number of arguments passed to a method.
#[derive(Debug, Error)]
#[error("Expected {expected} arguments, got {got} arguments")]
pub struct ArgumentError {
    /// Number of arguments that _should_ be passed to the method.
    pub expected: usize,
    /// Number of arguments that _actually got_ passed to the method.
    pub got: usize,
}

/// Deserialize a method argument using the [`ron`] file format.
///
/// This argument parser only requires deriving and registering `T`, unlike
/// the other parsers.
///
/// # Other parsers
///
/// [self#functions]
///
/// # Errors
/// See [`ReflectDslDeserError`] for possible errors.
pub fn from_reflect<T: Reflect + FromReflect>(
    registry: &TypeRegistry,
    _: Option<&mut LoadContext>,
    input: &str,
) -> Result<T, ReflectDslDeserError> {
    use ron::de::Deserializer as Ronzer;
    use ReflectDslDeserError as Error;

    let id = any::TypeId::of::<T>();
    let registration = registry.get(id).ok_or_else(Error::not_registered::<T>)?;
    let mut ron_de = Ronzer::from_str(input).map_err(Error::ron_deser::<T>)?;
    let de = TypedReflectDeserializer::new(registration, registry);
    let deserialized = match de.deserialize(&mut ron_de) {
        Ok(ok) => ok,
        Err(err) => return Err(Error::ron_deser::<T>(ron_de.span_error(err))),
    };
    T::from_reflect(deserialized.as_ref()).ok_or_else(Error::bad_reflect::<T>)
}

/// Deserialize a method argument using the [`FromStr`] `std` trait.
///
/// For your own types, it might be more succint to define your own parser
/// rather than depend on `ron`.
///
/// # Other parsers
///
/// [self#functions]
///
/// # Errors
/// [`FromStr::from_str`] fails, there is a parsing error.
#[inline(always)]
pub fn from_str<T: FromStr>(
    _: &TypeRegistry,
    _: Option<&mut LoadContext>,
    input: &str,
) -> Result<T, T::Err>
where
    T::Err: std::error::Error + Send + Sync,
{
    input.parse()
}

/// Load an asset from the path declared in `input`.
///
/// This argument parser only works on `Handle<T>`.
///
/// # Other parsers
///
/// [self#functions]
///
/// # Errors
/// See [`HandleDslDeserError`] for possible errors.
#[inline(always)]
pub fn to_handle<T: Asset + LoadAsset>(
    _: &TypeRegistry,
    load_context: Option<&mut LoadContext>,
    input: &str,
) -> Result<Handle<T>, HandleDslDeserError<T>> {
    let Some(ctx) = load_context else {
        return Err(HandleDslDeserError::<T>::NoLoadContext);
    };
    let input = input.to_owned();
    Ok(ctx.load(input))
}

/// Returns the input as a `&str`, removing quotes applying backslash escapes.
///
/// This allocates whenever a backslash is used in the input string.
///
/// # Other parsers
///
/// [self#functions]
///
/// # Errors
///
/// This is always `Ok`. It is safe to unwrap. Rust guarentees that `Infallible`
/// can't be constructed.
#[inline(always)]
pub fn quoted<'a>(
    _: &TypeRegistry,
    _: Option<&mut LoadContext>,
    input: &'a str,
) -> Result<Cow<'a, str>, Infallible> {
    Ok(interpret_str(input))
}

fn interpret_str(mut input: &str) -> Cow<str> {
    if input.len() > 2 && input.starts_with('"') && input.ends_with('"') {
        input = &input[1..input.len() - 1];
    }
    // SAFTEY: transforms operated by escape_literal is always UTF8-safe
    unsafe {
        match escape_literal(input.as_bytes()) {
            Cow::Borrowed(bytes) => Cow::Borrowed(str::from_utf8_unchecked(bytes)),
            Cow::Owned(bytes_vec) => Cow::Owned(String::from_utf8_unchecked(bytes_vec)),
        }
    }
}

enum ArgumentsInner<'i, 'a> {
    Parser(&'a parser::Arguments<'i, 'a>),
    Named(Cow<'i, [u8]>),
}

/// Arguments passed to a method.
///
/// In the `chirp` file, this corresponds to the text within parenthesis following
/// a method name:
///
/// ```text
/// //                 vvvv  vvvvvvvvv     vvvvvv
/// Entity(method_name(arg1, arg2(Foo)   , 10 + 3) other_method))
/// ```
///
/// Arguments will be **stripped of comments and surrouding spaces**,
/// and **[parameter substitution]** will be applied within templates.
///
/// # Call format
///
/// You can use [`Arguments::len`] to check how many arguments were passed to
/// the method and [`Arguments::get`] to access an argument at a provided index.
///
/// A bare method (without following parenthesis) will have zero arguments,
/// a method with following parenthesis with no arguments will **also** have zero
/// arguments.
///
/// For example:
///
/// ```text
/// Entity(bare_method method1() method2(foobar) method3("foobar") method4(  10 + 3 , bar))
/// ```
///
/// |`name`|`bare_method`|`method1`|`method2`|`method3`|`method4`|
/// |------|-------------|---------|---------|---------|---------|
/// |`ctx.arguments.get(0)`|`None`|`None`| `foobar` | `"foobar"` | `10 + 3`|
/// |`ctx.arguments.len()`|`0`|`0` | `1`     | `1`     | `2`     |
///
///
/// # How to handle argument parsing
///
/// `cuicui_chirp` expects end-users to use the [`parse_dsl_impl`] macro or
/// [`ReflectDsl`] struct to take care of parsing for them.
///
/// A set of "blessed" parsers is predefined in the [`args`](self)
/// module. Those are the parsers used by default by `parse_dsl_impl`.
///
/// You can call them with the output of [`Arguments::get_str`].
///
/// `ReflectDsl` uses the [`from_reflect`] and [`to_handle`]
/// parsers.
///
/// [`ReflectDsl`]: crate::ReflectDsl
///
///
/// [parameter substitution]: crate#parameter-substitution
/// [`parse_dsl_impl`]: mod@crate::parse_dsl_impl
pub struct Arguments<'i, 'a>(ArgumentsInner<'i, 'a>);

impl<'i, 'a> Arguments<'i, 'a> {
    pub(crate) fn for_name(name: &'i [u8]) -> Self {
        let surrounded_by = |quote| name.starts_with(quote) && name.ends_with(quote);
        let name = if name.len() >= 2 && (surrounded_by(b"\"") || surrounded_by(b"'")) {
            escape_literal(&name[1..name.len() - 1])
        } else {
            Cow::Borrowed(name)
        };
        Self(ArgumentsInner::Named(name))
    }
    /// Whether arguments were passed to the method.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// How many arguments were passed to the method.
    #[must_use]
    pub const fn len(&self) -> usize {
        match &self.0 {
            ArgumentsInner::Parser(p) => p.len(),
            ArgumentsInner::Named(_) => 1,
        }
    }
    /// Get the `index`th argument passed to the method.
    ///
    /// `None` if `index > self.len()`.
    ///
    /// Template [parameter substitution] is applied. This allocates if there
    /// is one or more substitutions for the queryed argument, that is not the
    /// whole argument.
    ///
    /// Note that trailing and leading whitespaces are trimmed from arguments.
    ///
    /// [parameter substitution]: crate#parameter-substitution
    #[must_use]
    pub fn get(&self, index: usize) -> Option<Cow<'_, [u8]>> {
        match &self.0 {
            ArgumentsInner::Parser(p) => p.get(index),
            ArgumentsInner::Named(n) if index == 0 => Some(Cow::Borrowed(n.as_ref())),
            ArgumentsInner::Named(_) => None,
        }
    }
    /// Get the `index`th argument passed to the method as a `str`.
    ///
    /// See [`Self::get`] for more details.
    ///
    /// May allocate on invalid UTF8 (uses [`String::from_utf8_lossy`] internally).
    ///
    /// # Panics
    /// Will panics on invalid UTF8, if the argument was substitued.
    #[must_use]
    pub fn get_str(&self, index: usize) -> Option<Cow<str>> {
        match &self.0 {
            ArgumentsInner::Parser(p) => p.get(index).map(|p| match p {
                Cow::Borrowed(p) => String::from_utf8_lossy(p),
                Cow::Owned(p) => Cow::Owned(String::from_utf8(p).unwrap()),
            }),
            ArgumentsInner::Named(n) if index == 0 => Some(String::from_utf8_lossy(n)),
            ArgumentsInner::Named(_) => None,
        }
    }
}
impl<'i, 'a> From<&'a parser::Arguments<'i, 'a>> for Arguments<'i, 'a> {
    fn from(value: &'a parser::Arguments<'i, 'a>) -> Self {
        Self(ArgumentsInner::Parser(value))
    }
}
