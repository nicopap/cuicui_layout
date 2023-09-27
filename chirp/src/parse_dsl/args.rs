//! Parse individual method arguments.
//!
//! The functions in this module are available in the `parse_dsl_impl` macro's
//! `type_parsers` argument. It is however possible to define and substitute your
//! own.
//!
//! If a method accepts several arguments, the string is first split using the
//! [`super::split()`] function.
#![allow(clippy::inline_always)]
// allow: rust has difficulties inlining functions cross-crate. Since we only
// use inline(always) on functions that are very small, it won't add significative
// compile overhead in anycase, but may help the optimizer elide some code.

use std::{any, borrow::Cow, convert::Infallible, fs, io, marker::PhantomData, str, str::FromStr};

use bevy::asset::{Asset, FileAssetIo, Handle, LoadContext, LoadedAsset};
use bevy::reflect::erased_serde::__private::serde::de::DeserializeSeed;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::{FromReflect, Reflect, TypeRegistryInternal as TypeRegistry};
use thiserror::Error;

use super::escape_literal;
use crate::load_asset::LoadAsset;

fn tyname<T>() -> &'static str {
    any::type_name::<T>()
}
/// Error occuring in [`to_handle`].
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

/// Deserialize a method argument using the [`ron`] file format.
///
/// This argument parser only requires deriving and registering `T`, unlike
/// the other parsers.
///
/// # Other parsers
/// - [`from_str`]
/// - [`to_handle`]
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
/// - [`from_reflect`]
/// - [`to_handle`]
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
/// - [`from_str`]
/// - [`from_reflect`]
///
/// # Errors
/// See [`HandleDslDeserError`] for possible errors.
#[inline(always)]
pub fn to_handle<T: Asset + LoadAsset>(
    _: &TypeRegistry,
    load_context: Option<&mut LoadContext>,
    input: &str,
) -> Result<Handle<T>, HandleDslDeserError<T>> {
    use HandleDslDeserError::{BadLoad, UnsupportedIo};

    let Some(ctx) = load_context else {
        return Err(HandleDslDeserError::<T>::NoLoadContext);
    };
    let file_io: &FileAssetIo = ctx.asset_io().downcast_ref().ok_or(UnsupportedIo)?;
    let input = interpret_str(input);
    let mut file_path = file_io.root_path().clone();
    file_path.push(input.as_ref());
    let bytes = fs::read(&file_path)?;
    let asset = T::load(&file_path, &bytes, ctx).map_err(BadLoad)?;
    Ok(ctx.set_labeled_asset(input.as_ref(), LoadedAsset::new(asset)))
}

/// Returns the input as a `&str`, removing quotes applying backslash escapes.
///
/// This allocates whenever a backslash is used in the input string.
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
