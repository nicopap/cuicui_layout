//! Wrapper for method arguments for configurable parsing.
//!
//! The functions in this module are available in the `parse_dsl_impl` macro's
//! `type_parsers` argument. It is however possible to define and substitute your
//! own.
#![allow(clippy::inline_always)]
// allow: rust has difficulties inlining functions cross-crate. Since we only
// use inline(always) on functions that are very small, it won't add significative
// compile overhead in anycase, but may help the optimizer elide some code.

use std::{any, convert::Infallible, marker::PhantomData, str, str::FromStr};

use bevy::asset::{Asset, AssetPath, Handle, LoadContext};
use bevy::reflect::{
    erased_serde::__private::serde::de::DeserializeSeed, serde::TypedReflectDeserializer,
    FromReflect, Reflect, TypeRegistryInternal as TypeRegistry,
};
use thiserror::Error;

/// Error occuring in [`to_handle`].
#[allow(missing_docs)] // Already documented by error message
#[derive(Debug, Error)]
pub enum HandleDslDeserError<T> {
    #[error(
        "Didn't provide a LoadContext when deserializing a 'Handle<{}>'. \
        This is required to enable loading assets.",
        any::type_name::<T>(),
    )]
    NoLoadContext,
    #[doc(hidden)]
    #[error("This error never occurs")]
    _Ignore(PhantomData<fn(T)>),
}

/// Error occuring in [`from_reflect`].
#[allow(missing_docs)] // Already documented by error message
#[derive(Debug, Error)]
pub enum ReflectDslDeserError<T> {
    #[error(
        "Tried to deserialize a DSL argument using reflection, yet '{}' \
        is not registered.",
        any::type_name::<T>(),
    )]
    NotRegistered,
    #[error(
        "Ron couldn't deserialize the DSL argument of type '{}': {0}",
        any::type_name::<T>(),
    )]
    RonDeser(#[from] ron::error::SpannedError),
    #[error(
        "Bevy couldn't deserialize the DSL argument of type '{}': {0}",
        any::type_name::<T>(),
    )]
    BevyRonDeser(#[from] ron::error::Error),
    #[error(
        "The DSL argument of type '{}' was parsed by bevy in RON, but the \
        generated reflect proxy type couldn't be converted into '{}'",
        any::type_name::<T>(),
        any::type_name::<T>(),
    )]
    BadReflect,
    #[doc(hidden)]
    #[error("This error never occurs")]
    _Ignore(PhantomData<fn(T)>),
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
    _: Option<&LoadContext>,
    input: &str,
) -> Result<T, ReflectDslDeserError<T>> {
    use ron::de::Deserializer as Ronzer;
    use ReflectDslDeserError as Error;

    let registration = registry
        .get(any::TypeId::of::<T>())
        .ok_or(Error::<T>::NotRegistered)?;
    let mut ron_de = Ronzer::from_str(input).map_err(Error::<T>::RonDeser)?;
    let de = TypedReflectDeserializer::new(registration, registry)
        .deserialize(&mut ron_de)
        .map_err(Error::BevyRonDeser::<T>)?;
    T::from_reflect(de.as_ref()).ok_or(Error::<T>::BadReflect)
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
    _: Option<&LoadContext>,
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
pub fn to_handle<T: Asset>(
    _: &TypeRegistry,
    load_context: Option<&LoadContext>,
    input: &str,
) -> Result<Handle<T>, HandleDslDeserError<T>> {
    let Some(ctx) = load_context else {
        return Err(HandleDslDeserError::<T>::NoLoadContext);
    };
    Ok(ctx.get_handle(AssetPath::from(input)))
}

/// Returns the input as a `&str` without further changes.
///
/// # Errors
/// This is always `Ok`. It is safe to unwrap. Rust guarentees that `Infallible`
/// can't be constructed.
#[inline(always)]
pub const fn identity<'a>(
    _: &TypeRegistry,
    _: Option<&LoadContext>,
    input: &'a str,
) -> Result<&'a str, Infallible> {
    Ok(input)
}
