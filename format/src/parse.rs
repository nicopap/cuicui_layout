//! Parse a DSL.

use core::fmt;
use std::{
    marker::PhantomData,
    str::{FromStr, Utf8Error},
};

use bevy::prelude::Entity;
use cuicui_dsl::{BaseDsl, DslBundle, EntityCommands};
use kdl::KdlError;
use thiserror::Error;

use winnow::{
    error::{ContextError, ErrMode, StrContext},
    PResult,
};

/// Error occuring at the initial KDL parsing of the DSL.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error)]
pub enum DslError {
    #[error("Input data is not valid UTF8: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("Input file is not a valid KDL file: {0}")]
    Kdl(#[from] KdlError),
}

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

#[derive(Debug)]
enum ParseType {
    Method,
    LeafNode,
}
impl fmt::Display for ParseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseType::Method => write!(f, "method"),
            ParseType::LeafNode => write!(f, "leaf node"),
        }
    }
}
/// The input specification called a method not implemented in `D`.
#[derive(Debug, Error)]
#[error(
    "Didn't find {parse_type} '{method}' for parse DSL of type '{}'",
    std::any::type_name::<D>(),
)]
pub struct DslParseError<D> {
    method: String,
    parse_type: ParseType,
    _ty: PhantomData<D>,
}
impl<D> DslParseError<D> {
    fn new(method: impl Into<String>, parse_type: ParseType) -> Self {
        Self {
            method: method.into(),
            parse_type,
            _ty: PhantomData,
        }
    }
}

/// Argument to [`ParseDsl::method`].
pub struct InterpretMethodCtx<'a> {
    /// The method name.
    pub name: &'a str,
    /// The method arguments (notice **plural**).
    ///
    /// Use the [`quick`] module to split the argument in as many relevant
    /// sections as necessary.
    pub args: &'a str,
    // let path = AssetPath::new_ref(load_context.path(), Some(&label));
    // load_context.get_handle(path)?
    // pub load_context: LoadContext<'b>,
    // TODO(perf): Consider re-using cuicui_fab::Binding
}

/// Argument to [`ParseDsl::leaf_node`].
pub struct InterpretLeafCtx<'n, 'a, 'c, 'w, 's> {
    /// The leaf node name.
    pub name: &'n str,
    /// The first method to the leaf node.
    pub leaf_arg: &'n str,
    /// The [`EntityCommands`] for the current statement's [`Entity`].
    pub cmds: &'c mut EntityCommands<'w, 's, 'a>,
    // pub load_context: LoadContext<'b>,
}

/// A [`DslBundle`] that can be parsed.
pub trait ParseDsl: DslBundle {
    /// Apply method named `name` to `self`.
    ///
    /// Note that in a [parent node] statement, the initial identifier — if not
    /// `code` or `spawn` — is applied as the last method.
    ///
    /// [parent node]: cuicui_dsl::dsl#parent-node
    fn method(&mut self, ctx: InterpretMethodCtx) -> Result<(), anyhow::Error>;
    /// Apply leaf node method named `name` to `self`.
    ///
    /// Called when encountering a [leaf node].
    ///
    /// Note that it respects the semantic of the DSL, `leaf_node` is only called
    /// after all other [methods] of the current [statement] are applied.
    ///
    /// [leaf node]: cuicui_dsl::dsl#leaf-node
    /// [methods]: cuicui_dsl::dsl#dsl-methods
    /// [statement]: cuicui_dsl::dsl#dsl-statements
    fn leaf_node(&mut self, ctx: InterpretLeafCtx) -> Result<Entity, anyhow::Error>;
}
impl ParseDsl for BaseDsl {
    fn method(&mut self, data: InterpretMethodCtx) -> Result<(), anyhow::Error> {
        let InterpretMethodCtx { name, args, .. } = data;
        match name {
            "named" => {
                self.named(args.to_owned());
                Ok(())
            }
            method => Err(DslParseError::<Self>::new(method, ParseType::Method).into()),
        }
    }
    fn leaf_node(&mut self, data: InterpretLeafCtx) -> Result<Entity, anyhow::Error> {
        Err(DslParseError::<Self>::new(data.name, ParseType::LeafNode).into())
    }
}
fn balanced_text<'i>(input: &mut &'i str) -> PResult<&'i str> {
    use winnow::{
        ascii::escaped,
        combinator::{dispatch, fail, repeat, terminated},
        token::{any, one_of, take_till1},
        Parser,
    };
    const SCOPE_TERMINATE: [char; 7] = ['(', ')', '[', ']', '{', '}', '\\'];
    const SCOPE_ESCAPE: [char; 9] = ['(', ')', '[', ']', '{', '}', '|', ',', '\\'];
    const EXPOSED_TERMINATE: [char; 7] = ['(', '[', '{', '}', '|', ',', '\\'];
    fn scope<'i>(input: &mut &'i str) -> PResult<&'i str> {
        let semi_exposed = || escaped(take_till1(SCOPE_TERMINATE), '\\', one_of(SCOPE_ESCAPE));
        let repeat = |f| repeat::<_, _, (), _, _>(0.., f);
        let inner = move || (semi_exposed(), repeat((scope, semi_exposed())));
        let dispatch = dispatch! {any;
            '{' => terminated(inner(), '}'),
            '[' => terminated(inner(), ']'),
            '(' => terminated(inner(), ')'),
            _ => fail,
        };
        dispatch.recognize().parse_next(input)
    }
    let exposed = || escaped(take_till1(EXPOSED_TERMINATE), '\\', one_of(SCOPE_ESCAPE));

    let repeat = |f| repeat::<_, _, (), _, _>(0.., f);
    (exposed(), repeat((scope, exposed())))
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

    use winnow::{ascii::multispace0, combinator::preceded, Parser};

    use super::{balanced_text, ArgError, FromStr};

    struct ArgIter<'a> {
        input: &'a str,
        count: usize,
    }

    impl<'a> ArgIter<'a> {
        fn new(input: &'a str) -> Self {
            Self { input: &input[1..input.len() - 1], count: 0 }
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
                Some(balanced_text.parse_next(&mut self.input).map_err(err))
            } else {
                let mut parser = preceded((',', multispace0), balanced_text);
                Some(parser.parse_next(&mut self.input).map_err(err))
            }
        }
    }
    enum Args<'a> {
        Iter(ArgIter<'a>),
        Empty,
        One(&'a str),
    }

    impl<'a> Args<'a> {
        fn new(input: &str) -> Args {
            match () {
                () if input.is_empty() => Args::Empty,
                () if !input.starts_with('(') => Args::One(input),
                () => Args::Iter(ArgIter::new(input)),
            }
        }
        fn count(&mut self) -> usize {
            match self {
                Args::Iter(iter) => iter.count(),
                Args::Empty => 0,
                Args::One(_) => 1,
            }
        }
    }
    pub fn arg0(input: &str) -> Result<(), ArgError> {
        match Args::new(input) {
            Args::Empty => Ok(()),
            mut args => Err(ArgError::CountMismatch(0, args.count())),
        }
    }
    pub fn arg1<T1: FromStr>(input: &str) -> Result<T1, ArgError>
    where
        T1::Err: std::error::Error + Send + Sync + 'static,
    {
        let bad_count = |count: usize| ArgError::CountMismatch(1, count);
        match Args::new(input) {
            Args::One(input) => Ok(input.parse().map_err(anyhow::Error::from)?),
            Args::Iter(mut iter) => {
                let arg1 = iter
                    .next()
                    .ok_or_else(|| bad_count(iter.count))?
                    .map_err(anyhow::Error::from)?
                    .parse()
                    .map_err(anyhow::Error::from)?;
                if iter.next().is_some() {
                    Err(bad_count(iter.count + iter.count()))
                } else {
                    Ok(arg1)
                }
            }
            mut args => Err(bad_count(args.count())),
        }
    }
    pub fn arg2<T1: FromStr, T2: FromStr>(input: &str) -> Result<(T1, T2), ArgError>
    where
        T1::Err: std::error::Error + Send + Sync + 'static,
        T2::Err: std::error::Error + Send + Sync + 'static,
    {
        let bad_count = |count: usize| ArgError::CountMismatch(2, count);
        match Args::new(input) {
            Args::Iter(mut iter) => {
                let arg1 = iter
                    .next()
                    .ok_or_else(|| bad_count(iter.count))?
                    .map_err(anyhow::Error::from)?
                    .parse()
                    .map_err(anyhow::Error::from)?;
                let arg2 = iter
                    .next()
                    .ok_or_else(|| bad_count(iter.count))?
                    .map_err(anyhow::Error::from)?
                    .parse()
                    .map_err(anyhow::Error::from)?;
                if iter.next().is_some() {
                    Err(bad_count(iter.count + iter.count()))
                } else {
                    Ok((arg1, arg2))
                }
            }
            mut args => Err(bad_count(args.count())),
        }
    }
    pub fn arg3<T1: FromStr, T2: FromStr, T3: FromStr>(
        input: &str,
    ) -> Result<(T1, T2, T3), ArgError>
    where
        T1::Err: std::error::Error + Send + Sync + 'static,
        T2::Err: std::error::Error + Send + Sync + 'static,
        T3::Err: std::error::Error + Send + Sync + 'static,
    {
        let bad_count = |count: usize| ArgError::CountMismatch(3, count);
        match Args::new(input) {
            Args::Iter(mut iter) => {
                let arg1 = iter
                    .next()
                    .ok_or_else(|| bad_count(iter.count))?
                    .map_err(anyhow::Error::from)?
                    .parse()
                    .map_err(anyhow::Error::from)?;
                let arg2 = iter
                    .next()
                    .ok_or_else(|| bad_count(iter.count))?
                    .map_err(anyhow::Error::from)?
                    .parse()
                    .map_err(anyhow::Error::from)?;
                let arg3 = iter
                    .next()
                    .ok_or_else(|| bad_count(iter.count))?
                    .map_err(anyhow::Error::from)?
                    .parse()
                    .map_err(anyhow::Error::from)?;
                if iter.next().is_some() {
                    Err(bad_count(iter.count + iter.count()))
                } else {
                    Ok((arg1, arg2, arg3))
                }
            }
            mut args => Err(bad_count(args.count())),
        }
    }
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
        assert_eq!(quick::arg0("(some text, text)"), Err(count(0, 2)));

        // 1 argument expected
        assert_eq!(quick::arg1::<String>(""), Err(count(1, 0)));
        assert_eq!(quick::arg1::<String>("some text"), Ok(s("some text")));
        assert_eq!(quick::arg1::<String>("(some text)"), Ok(s("some text")));
        assert_eq!(quick::arg1::<u32>("4263"), Ok(4263));
        assert_eq!(quick::arg1::<u32>("(1337)"), Ok(1337));
        assert_eq!(quick::arg1::<u32>("(badnumber)"), Err(badnumber_error()));
        assert_eq!(quick::arg1::<u32>("(1337, text)"), Err(count(1, 2)));

        // 2 argument expected
        assert_eq!(quick::arg2::<String, String>(""), Err(count(2, 0)));
        assert_eq!(quick::arg2::<String, String>("some text"), Err(count(2, 1)));
        assert_eq!(
            quick::arg2::<String, String>("(some text)"),
            Err(count(2, 1))
        );
        assert_eq!(
            quick::arg2::<u32, String>("(4263,     some text)"),
            Ok((4263, s("some text")))
        );
        assert_eq!(
            quick::arg2::<String, u32>("(some text, 1337)"),
            Ok((s("some text"), 1337))
        );
        assert_eq!(
            quick::arg2::<String, u32>("(some text, badnumber)"),
            Err(badnumber_error())
        );
        assert_eq!(
            quick::arg2::<u32, u32>("(1337, 4263, Too many arguments)"),
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
