//! [`ReflectDsl`] and helper types.
//!
//! Instead of using [`ParseDsl`]
use std::{any::type_name, convert::Infallible, fmt, marker::PhantomData};

use anyhow::Result;
use bevy::ecs::prelude::Bundle;
use bevy::prelude::{Deref, DerefMut};
use bevy::reflect::erased_serde::__private::serde::de::DeserializeSeed;
use bevy::reflect::{serde::TypedReflectDeserializer, Reflect, Struct};
use cuicui_dsl::DslBundle;
use thiserror::Error;

use crate::parse_dsl::{MethodCtx, ParseDsl};

/// Occurs in [`ReflectDsl::typed_method`].
#[derive(Error)]
enum ReflectDslError<T> {
    #[error(
        "Method on `ReflectDsl` was called with not exactly one argument. \
        Try having double parenthesis around the method argument"
    )]
    NotExactlyOneArgument,
    #[error(
        "Tried to set the field '{0}' of ReflectDsl<{ty}>, but {ty} \
        doesn't have such a field",
        ty=type_name::<T>()
    )]
    BadField(String),
    #[error(
        "The field {path} of '{ty}' is not registered. \
        Please register the type '{missing}' to be able to use ReflectDsl<{ty}>.",
        ty=type_name::<T>(),
    )]
    NotRegistered { path: String, missing: String },
    #[error("Failed to deserialize ReflectDsl<{}>: {0}", type_name::<T>())]
    BadDeser(anyhow::Error),
    #[error("This error never happens")]
    _Ignonre(PhantomData<fn(T)>, Infallible),
}
impl<T> fmt::Debug for ReflectDslError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ReflectDslError::{BadDeser, BadField, NotRegistered, _Ignonre};
        match self {
            BadField(field) => f.debug_tuple("BadField").field(field).finish(),
            NotRegistered { path, missing } => f
                .debug_struct("NotRegistered")
                .field("path", path)
                .field("missing", missing)
                .finish(),
            BadDeser(error) => f.debug_tuple("BadDeser").field(error).finish(),
            Self::NotExactlyOneArgument => write!(f, "NotExactlyOneArgument"),
            _Ignonre(..) => unreachable!(),
        }
    }
}

/// A `serde` deserializer used to parse some `input` into a `Box<dyn Reflect>`.
///
/// This is used in [`ReflectDsl`] to deserialize method arguments (`T`'s field values).
/// A default implementation is provided with [`RonFormat`].
pub trait Format {
    /// Deserialize into a `Box<dyn Reflect>`, any error is propagated by [`ReflectDsl::method`].
    #[allow(clippy::missing_errors_doc)] // false+: We can't say what our users will fail with.
    fn deserialize(input: &[u8], de: TypedReflectDeserializer) -> Result<Box<dyn Reflect>>;
}
/// Deserialize method arguments as `ron` strings.
///
/// This is the default deserialization method for [`ReflectDsl`].
pub struct RonFormat;
impl Format for RonFormat {
    fn deserialize(input: &[u8], de: TypedReflectDeserializer) -> Result<Box<dyn Reflect>> {
        Ok(de.deserialize(&mut ron::de::Deserializer::from_bytes(input)?)?)
    }
}

/// Automatic [`ParseDsl`] implementation for any [`Bundle`] + [`Reflect`] `struct`.
///
/// If you find using the `parse_dsl_impl` macro burdensome, and just want to
/// use any bevy `Bundle` as a DSL, you can use `ReflectDsl` to use the `struct`
/// fields as DSL "methods", and [`Reflect` deserialization][refl-deser] to parse
/// the arguments automatically.
///
/// # How to use
///
/// You have a type as follow:
/// ```no_run
/// use cuicui_chirp::ReflectDsl;
/// # use bevy::prelude::*;
///
/// #[derive(Bundle, Reflect, Default)]
/// struct MyBundle {
///     transform: Transform,
///     visibility: Visibility,
/// }
/// # let mut app = App::new();
/// // and you did register it with:
/// app.register_type::<MyBundle>();
/// ```
/// You want to use it in a DSL when parsing files. Consider the following
/// `chirp` file:
/// ```text
/// entity(row) {
///     entity (
///         transform (
///            translation: (x: 0.0, y: 0.0, z: 0.0),
///            rotation: (0.0, 0.0, 0.0, 1.0),
///            scale: (x: 1.0, y: 1.0, z: 1.0),
///          ),
///          visibility Inherited,
///     );
/// }
/// ```
/// You want both `LayoutDsl` methods and fields of `MyBundle` to work in your
/// chirp files.
///
/// In order to do that, you should add the loader for the correct DSL as follow:
/// ```no_run
/// use cuicui_chirp::ReflectDsl;
/// # use bevy::prelude::*;
///
/// # type LayoutDsl = ();
/// # #[derive(Bundle, Reflect, Default)] struct MyBundle { transform: Transform, visibility: Visibility }
/// // Add `ReflectDsl<MyBundle, _>` as an extension on the `LayoutDsl` DSL.
/// type Dsl = ReflectDsl<MyBundle, LayoutDsl>;
/// # let mut app = App::new();
/// app.add_plugins((
/// # bevy::asset::AssetPlugin::default(),
///     // ...
///     // The loader now recognizes the methods `transform` and `visibility`.
///     // They were fields of `MyBundle`
///     cuicui_chirp::loader::Plugin::new::<Dsl>(),
/// ));
/// ```
///
/// # Caveats
///
/// This doesn't work with the `dsl!` macro. You can only use `ReflectDsl` with
/// scenes defined in `chirp` files.
///
/// [refl-deser]: https://docs.rs/bevy_reflect/latest/bevy_reflect/#serialization
#[derive(Debug, Clone, Deref, DerefMut)]
pub struct ReflectDsl<T: Struct, D: DslBundle = (), F: Format = RonFormat> {
    inner: Option<T>,
    #[deref]
    delegate_dsl: D,
    _format: PhantomData<F>,
}

impl<T: Default + Struct, D: DslBundle, F: Format> Default for ReflectDsl<T, D, F> {
    fn default() -> Self {
        Self {
            inner: Some(T::default()),
            delegate_dsl: D::default(),
            _format: PhantomData,
        }
    }
}
impl<T, D, F> DslBundle for ReflectDsl<T, D, F>
where
    T: Bundle + Default + Struct,
    D: DslBundle,
    F: Format,
{
    fn insert(&mut self, cmds: &mut cuicui_dsl::EntityCommands) {
        // unwrap: This `Self::default` in `Some` state, and only becomes `None` when `insert`
        // is called. Since it is only called once, it is fine to unwrap.
        cmds.insert(self.inner.take().unwrap());
        self.delegate_dsl.insert(cmds);
    }
}
impl<T, D, F> ReflectDsl<T, D, F>
where
    T: Bundle + Default + Struct,
    D: DslBundle,
    F: Format,
{
    /// This is just so the error type is easier to convert in the `ParseDsl::method` impl.
    fn typed_method(&mut self, ctx: &MethodCtx) -> Result<(), ReflectDslError<T>> {
        use ReflectDslError::{BadDeser, BadField};
        // unwrap: Same logic as in `DslBundle::insert`
        let inner = self.inner.as_mut().unwrap();
        if ctx.arguments.len() != 1 {
            return Err(ReflectDslError::NotExactlyOneArgument);
        }
        let argument = ctx.arguments.get(0).unwrap();
        let Some(field_to_update) = inner.field_mut(ctx.name) else {
            return Err(BadField(ctx.name.to_string()));
        };
        let id = field_to_update.type_id();
        let not_registered = || ReflectDslError::NotRegistered {
            path: ctx.name.to_string(),
            missing: field_to_update.type_name().to_string(),
        };
        let registration = ctx.registry.get(id).ok_or_else(not_registered)?;
        let de = TypedReflectDeserializer::new(registration, ctx.registry);
        let field_value = F::deserialize(&argument, de).map_err(BadDeser)?;
        // unwrap: Error should never happen, since we get the registration for field.
        field_to_update.set(field_value).unwrap();
        Ok(())
    }
}
impl<T, D, F> ParseDsl for ReflectDsl<T, D, F>
where
    T: Bundle + Default + Struct,
    D: DslBundle,
    F: Format,
{
    fn method(&mut self, ctx: MethodCtx) -> Result<()> {
        Ok(self.typed_method(&ctx)?)
    }
}
