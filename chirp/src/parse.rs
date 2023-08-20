//! Parse a DSL.

use std::{borrow::Cow, marker::PhantomData, str::FromStr};

use cuicui_dsl::{BaseDsl, DslBundle};
use thiserror::Error;

use winnow::{
    ascii,
    error::{ContextError, ErrMode, StrContext},
    BStr, PResult, Parser,
};

/// Error returned by one of the `argN` functions.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error)]
pub enum ArgError {
    #[error("Expected {0} arguments, got {1}")]
    CountMismatch(u32, usize),
    #[error(transparent)]
    Parse(#[from] anyhow::Error),
    #[error("Parser error: {0}")]
    ArgParse(ErrMode<ContextError<StrContext>>),
}
impl PartialEq for ArgError {
    fn eq(&self, other: &Self) -> bool {
        use ArgError::{ArgParse, CountMismatch, Parse};
        match (self, other) {
            (CountMismatch(l1, l2), CountMismatch(r1, r2)) => (l1 == r1) & (l2 == r2),
            (Parse(l), Parse(r)) => l.to_string() == r.to_string(),
            (ArgParse(l), ArgParse(r)) => l == r,
            (CountMismatch(..) | Parse(_) | ArgParse(_), _) => false,
        }
    }
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
pub struct MethodCtx<'a> {
    /// The method name.
    pub name: Cow<'a, str>,
    /// The method arguments (notice **plural**).
    ///
    /// Use the [`quick`] module to split the argument in as many relevant
    /// sections as necessary.
    pub args: Cow<'a, str>,
    // let path = AssetPath::new_ref(load_context.path(), Some(&label));
    // load_context.get_handle(path)?
    // pub load_context: LoadContext<'b>,
    // TODO(perf): Consider re-using cuicui_fab::Binding
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
pub(crate) fn scoped_text<'i>(input: &mut &'i BStr) -> PResult<&'i [u8]> {
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
pub(crate) fn balanced_text<'i>(input: &mut &'i BStr) -> PResult<&'i [u8]> {
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

    use winnow::{ascii::multispace0, combinator::preceded, BStr, Parser};

    use super::{balanced_text, ArgError, FromStr};

    macro_rules! dummy {
        ($_ignore:tt, $($actual:tt)*) => { $($actual)* };
    }
    macro_rules! parse_iter {
        (@single $bad_count:ident, $iter:ident) => {
            $iter
                .next()
                .ok_or_else(|| $bad_count($iter.count))?
                .map_err(anyhow::Error::from)?
                .parse()
                .map_err(anyhow::Error::from)?
        };
        (@ret $bad_count:ident, $iter:ident, $args:tt) => {
            if $iter.next().is_some() {
                Err($bad_count($iter.count + $iter.count()))
            } else {
                Ok($args)
            }
        };
        ($name:ident, $count:literal, $($tys:ident),*) => {
            pub fn $name <$( $tys: FromStr, )*>(input: &str) -> Result<($( $tys ),*), ArgError>
            where $(
                <$tys as FromStr>::Err: std::error::Error + Send + Sync + 'static,
            )* {
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
        input: &'a BStr,
        count: usize,
    }

    impl<'a> ArgIter<'a> {
        fn new(input: &'a str) -> Self {
            Self { input: BStr::new(input), count: 0 }
        }
    }
    impl<'a> Iterator for ArgIter<'a> {
        type Item = Result<&'a str, ArgError>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.input.is_empty() {
                return None;
            }
            self.count += 1;
            let err = ArgError::ArgParse;
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
    pub fn arg1<T1: FromStr>(input: &str) -> Result<T1, ArgError>
    where
        T1::Err: std::error::Error + Send + Sync + 'static,
    {
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
        let s = |str: &str| str.to_owned();
        let count = |expected, got| ArgError::CountMismatch(expected, got);
        let badnumber_error =
            || ArgError::from(anyhow::Error::from("badnumber".parse::<u32>().unwrap_err()));

        // 0 arguments expected
        assert_eq!(quick::arg0(""), Ok(()));
        assert_eq!(quick::arg0("some text"), Err(count(0, 1)));
        assert_eq!(quick::arg0("some text, text"), Err(count(0, 2)));

        // 1 argument expected
        assert_eq!(quick::arg1::<String>(""), Err(count(1, 0)));
        assert_eq!(quick::arg1::<String>("some text"), Ok(s("some text")));
        assert_eq!(quick::arg1::<u32>("4263"), Ok(4263));
        assert_eq!(quick::arg1::<u32>("badnumber"), Err(badnumber_error()));
        assert_eq!(quick::arg1::<u32>("1337, text"), Err(count(1, 2)));

        // 2 argument expected
        assert_eq!(quick::arg2::<String, String>(""), Err(count(2, 0)));
        assert_eq!(quick::arg2::<String, String>("some text"), Err(count(2, 1)));
        assert_eq!(
            quick::arg2::<u32, String>("4263, some text"),
            Ok((4263, s("some text")))
        );
        assert_eq!(
            quick::arg2::<String, u32>("some text, 1337"),
            Ok((s("some text"), 1337))
        );
        assert_eq!(
            quick::arg2::<String, u32>("some text, badnumber"),
            Err(badnumber_error())
        );
        assert_eq!(
            quick::arg2::<u32, u32>("1337, 4263, Too many arguments"),
            Err(count(2, 3))
        );

        // 3 argument expected
        // let result = quick::arg3::<String, u32>(""); // Err
        // let result = quick::arg3::<String, u32>("some text"); // Ok
        // let result = quick::arg3::<String, u32>("(some text)"); // Ok
        // let result = quick::arg3::<u32, u32>("4263"); // Ok
        // let result = quick::arg3::<u32, u32>("(1337)"); // Ok
        // let result = quick::arg3::<u32, u32>("(badnumber)"); // Err
        // let result = quick::arg3::<u32, u32>("(1337, text)"); // Err
    }
}
