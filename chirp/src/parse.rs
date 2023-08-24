//! Parse a DSL.

use std::{borrow::Cow, marker::PhantomData};

use bevy::{asset::LoadContext, reflect::TypeRegistryInternal as TypeRegistry};
use cuicui_dsl::{BaseDsl, DslBundle};
use thiserror::Error;

use winnow::{ascii, BStr, Located, PResult, Parser};

/// Error returned by one of the `argN` functions.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArgError {
    #[error("Expected {0} arguments, got {1}")]
    CountMismatch(u32, usize),
    // TODO(perf): theoretically can be removed, as we already parsed everything
    // before passing it to `argN`
    #[error("Parser error")]
    ArgParse,
}

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
pub struct MethodCtx<'a, 'l, 'll, 'r> {
    /// The method name.
    pub name: Cow<'a, str>,
    /// The method arguments (notice **plural**).
    ///
    /// Use the [`quick`] module to split the argument in as many relevant
    /// sections as necessary.
    pub args: Cow<'a, str>,
    /// The [`LoadContext`] used to load assets referenced in `chirp` files.
    pub ctx: Option<&'l LoadContext<'ll>>,
    /// The [`TypeRegistry`] the interpreter was initialized with.
    pub registry: &'r TypeRegistry,
    // TODO(perf): Consider re-using cuicui_fab::Binding
    // TODO(feat): bindings/references
}

/// A [`DslBundle`] that can be parsed.
pub trait ParseDsl: DslBundle {
    /// Apply method named `name` to `self`.
    ///
    /// Note that in a [parent node] statement, the initial identifier — if not
    /// `code` or `spawn` — is applied as the last method.
    ///
    /// # Errors
    /// This function may fail. With `anyhow::Error`, any error type may be used.
    ///
    /// You may chose to fail for any reason, the expected failure case
    /// is failure to parse an argument in`ctx.args` or trying to call an
    /// innexisting method with `ctx.name`.
    ///
    /// [parent node]: cuicui_dsl::dsl#parent-node
    fn method(&mut self, ctx: MethodCtx) -> Result<(), anyhow::Error>;
}
impl ParseDsl for BaseDsl {
    fn method(&mut self, data: MethodCtx) -> Result<(), anyhow::Error> {
        let MethodCtx { name, args, .. } = data;
        match name.as_ref() {
            "named" => {
                self.named(match args {
                    Cow::Borrowed(b) => b.to_owned(),
                    Cow::Owned(o) => o,
                });
                Ok(())
            }
            method => Err(DslParseError::<Self>::new(method).into()),
        }
    }
}
const SCOPE_TERMINATE: [u8; 7] = *b"()[]{}\\";
const SCOPE_ESCAPE: [u8; 8] = *b"()[]{},\\";
const EXPOSED_TERMINATE: [u8; 6] = *b"([{},\\";
#[inline]
pub(crate) fn scoped_text<'i>(input: &mut Located<&'i BStr>) -> PResult<&'i [u8], ()> {
    use winnow::{
        combinator::{dispatch, fail, repeat, terminated},
        token::{any, one_of, take_till1},
    };
    let semi_exposed = || ascii::escaped(take_till1(SCOPE_TERMINATE), '\\', one_of(SCOPE_ESCAPE));
    let repeat = |f| repeat::<_, _, (), _, _>(0.., f);
    let inner = move || (semi_exposed(), repeat((scoped_text, semi_exposed())));
    let dispatch = dispatch! {any;
        b'{' => terminated(inner(), b'}'),
        b'[' => terminated(inner(), b']'),
        b'(' => terminated(inner(), b')'),
        _ => fail,
    };
    dispatch.recognize().parse_next(input)
}
#[inline]
pub(crate) fn balanced_text<'i>(input: &mut Located<&'i BStr>) -> PResult<&'i [u8], ()> {
    use winnow::{combinator::repeat, token::one_of, token::take_till1};

    let exposed = || ascii::escaped(take_till1(EXPOSED_TERMINATE), '\\', one_of(SCOPE_ESCAPE));

    let repeat = |f| repeat::<_, _, (), _, _>(0.., f);
    (exposed(), repeat((scoped_text, exposed())))
        .recognize()
        .parse_next(input)
}
/// Functions to parse cleanly method arguments.
///
/// See [`arg0`], [`arg1`], [`arg2`], [`arg3`].
///
/// The functions return errors when either the [`str::parse`] method fails
/// or the argument is not properly formatted.
///
/// [`arg0`]: quick::arg0
/// [`arg1`]: quick::arg1
/// [`arg2`]: quick::arg2
/// [`arg3`]: quick::arg3
#[allow(missing_docs, clippy::missing_errors_doc)]
pub mod quick {

    use std::str::from_utf8_unchecked;

    use winnow::{ascii::multispace0, combinator::preceded, BStr, Located, Parser};

    use super::{balanced_text, ArgError};

    macro_rules! dummy {
        ($_ignore:tt, $($actual:tt)*) => { $($actual)* };
    }
    macro_rules! parse_iter {
        (@single $bad_count:ident, $iter:ident) => {
            $iter.next().ok_or_else(|| $bad_count($iter.count))??
        };
        (@ret $bad_count:ident, $iter:ident, $args:tt) => {
            if $iter.next().is_some() {
                Err($bad_count($iter.count + $iter.count()))
            } else {
                Ok($args)
            }
        };
        ($name:ident, $count:literal, $($tys:ident),*) => {
            pub fn $name(input: &str) -> Result<($( dummy![$tys, &str] ),*), ArgError> {
                let bad_count = |count| ArgError::CountMismatch($count, count);
                match Args::new(input) {
                    Args::Iter(mut iter) => {
                        let args = ($( dummy!($tys, parse_iter!(@single bad_count, iter)) ),*);
                        parse_iter!(@ret bad_count, iter, args)
                    }
                    mut args => Err(bad_count(args.count())),
                }
            }

        }
    }

    struct ArgIter<'a> {
        input: Located<&'a BStr>,
        count: usize,
    }

    impl<'a> ArgIter<'a> {
        fn new(input: &'a str) -> Self {
            Self { input: Located::new(BStr::new(input)), count: 0 }
        }
    }
    impl<'a> Iterator for ArgIter<'a> {
        type Item = Result<&'a str, ArgError>;
        fn next(&mut self) -> Option<Self::Item> {
            #[cold]
            #[allow(clippy::missing_const_for_fn)] // false positive
            fn err<T>(_: T) -> ArgError {
                ArgError::ArgParse
            }
            if self.input.is_empty() {
                return None;
            }
            self.count += 1;
            if self.count - 1 == 0 {
                let text = balanced_text.parse_next(&mut self.input).map_err(err);
                // SAFETY: `ArgIter.input` is always valid utf8 because of the
                // constructor and the parser working exclusively on ASCII
                Some(text.map(|t| unsafe { from_utf8_unchecked(t) }))
            } else {
                let mut parser = preceded((b',', multispace0), balanced_text);
                let text = parser.parse_next(&mut self.input).map_err(err);
                Some(text.map(|t| unsafe { from_utf8_unchecked(t) }))
            }
        }
    }
    enum Args<'a> {
        Empty,
        Iter(ArgIter<'a>),
    }

    impl<'a> Args<'a> {
        fn new(input: &str) -> Args {
            match () {
                () if input.is_empty() => Args::Empty,
                () => Args::Iter(ArgIter::new(input)),
            }
        }
        fn count(&mut self) -> usize {
            match self {
                Args::Empty => 0,
                Args::Iter(iter) => iter.count(),
            }
        }
    }
    pub fn arg0(input: &str) -> Result<(), ArgError> {
        let bad_count = |count| ArgError::CountMismatch(0, count);
        match Args::new(input) {
            Args::Empty => Ok(()),
            Args::Iter(args) => Err(bad_count(args.count())),
        }
    }
    pub fn arg1(input: &str) -> Result<&str, ArgError> {
        let bad_count = |count| ArgError::CountMismatch(1, count);
        match Args::new(input) {
            Args::Iter(mut iter) => {
                let arg1 = parse_iter!(@single bad_count, iter);
                parse_iter!(@ret bad_count, iter, arg1)
            }
            Args::Empty => Err(bad_count(0)),
        }
    }
    parse_iter!(arg2, 2, T1, T2);
    parse_iter!(arg3, 3, T1, T2, T3);
    parse_iter!(arg4, 4, T1, T2, T3, T4);
    parse_iter!(arg5, 5, T1, T2, T3, T4, T5);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_arguments() {
        let count = |expected, got| ArgError::CountMismatch(expected, got);

        // 0 arguments expected
        assert_eq!(quick::arg0(""), Ok(()));
        assert_eq!(quick::arg0("some text"), Err(count(0, 1)));
        assert_eq!(quick::arg0("some text, text"), Err(count(0, 2)));

        // 1 argument expected
        assert_eq!(quick::arg1(""), Err(count(1, 0)));
        assert_eq!(quick::arg1("some text"), Ok("some text"));
        assert_eq!(quick::arg1("1337, text"), Err(count(1, 2)));

        // 2 argument expected
        assert_eq!(quick::arg2(""), Err(count(2, 0)));
        assert_eq!(quick::arg2("some text"), Err(count(2, 1)));
        assert_eq!(quick::arg2("some text, 1337"), Ok(("some text", "1337")));
        assert_eq!(
            quick::arg2("1337, 4263, Too many arguments"),
            Err(count(2, 3))
        );

        // 3 argument expected
        assert_eq!(quick::arg3(""), Err(count(3, 0)));
        assert_eq!(quick::arg3("some text"), Err(count(3, 1)));
        assert_eq!(quick::arg3("some text, 1337"), Err(count(3, 2)));
        assert_eq!(
            quick::arg3("1337,     4363, more arguments"),
            Ok(("1337", "4363", "more arguments"))
        );
    }
}
