//! Split method arguments.
//!
//! See [`split`].

use std::str::from_utf8_unchecked;

use thiserror::Error;
use winnow::error::ErrMode;

use crate::grammar::{self, arg_token_tree};
use crate::lex::Stateful;

/// Error returned by one of the `argN` functions.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArgError {
    #[error("Expected {0} arguments, got {}{1}", if .0 == &255 { "more than " } else {""})]
    CountMismatch(u8, u8),
    #[error("Parser error at offset: {0}")]
    ArgParse(grammar::Error),
}
impl<'i> From<ErrMode<grammar::Error>> for ArgError {
    fn from(err: ErrMode<grammar::Error>) -> Self {
        // TODO(err): Better error reporting
        ArgError::ArgParse(err.into_inner().unwrap())
    }
}

/// Split `input` into `N` elements on commas (`,`).
///
/// # Output format
///
/// The expected input format is described in [`super::MethodCtx`]. The output
/// is an `N`-sized array.
///
/// Each element of the array is a `&str` such that:
/// - It is split on "top level" commas. Meaning: commas that are found outside
///   of delimiters such as `(){}[]`, and outside of "quoted" (or 'quoted') strings.
/// - It trims whitespaces of the resulting split elements.
/// - It preserves as-is quotes and backslashes, however it accounts for
///   them when splitting.
///
/// # Examples
///
/// ```
/// use cuicui_chirp::parse::split::{split, ArgError::CountMismatch};
///
/// assert_eq!(split::<2>("(20%, 21px)"), Ok(["20%", "21px"]));
/// assert_eq!(split::<0>(""), Ok([]));
/// assert_eq!(split::<0>("(hello, world,)"), Err(CountMismatch(0, 2)));
/// assert_eq!(split::<99>("(hello, world,)"), Err(CountMismatch(99, 2)));
/// assert_eq!(split::<0>("()"), Ok([]));
/// assert_eq!(
///     split::<1>(r#"("hello\",\"world")"#),
///     Ok([r#""hello\",\"world""#]),
/// );
/// assert_eq!(
///     split::<2>(r#"("hello","world")"#),
///     Ok([r#""hello""#,r#""world""#]),
/// );
/// assert_eq!(
///     split::<3>("(float(3,1  ,4,5), matrix  [10, 1], {woody: woop, bady: boop})"),
///     Ok(["float(3,1  ,4,5)", "matrix  [10, 1]", "{woody: woop, bady: boop}"]),
/// );
/// ```
///
/// # Errors
/// - When the number of split elements is not exactly `N`.
/// - When including a string with a bad escape sequence (currently only
///   `\"'ntru` are accepted after a backslash)
/// - When `input` contains unbalanced `(){}[]` outside of string literals.
///
/// # Panics
/// If `N > 255`
#[inline(never)]
pub fn split<const N: usize>(input: &str) -> Result<[&str; N], ArgError> {
    let mut ret = [""; N];
    let n = u8::try_from(N).unwrap();

    let input = match () {
        () if N == 0 && (input.is_empty() || input == "()") => {
            return Ok(ret);
        }
        () if input.starts_with('(') && input.ends_with(')') => &input[1..input.len() - 1],
        () => input,
    };
    let mut arg_count = 0;
    let input = Stateful::new(input.trim().as_bytes(), ());
    arg_token_tree(input, |arg| {
        let str_arg = unsafe { from_utf8_unchecked(arg) };
        if arg_count < N {
            ret[arg_count] = str_arg.trim();
        }
        arg_count += 1;
    })?;

    if arg_count == N {
        Ok(ret)
    } else {
        let arg_count = u8::try_from(arg_count).unwrap_or(255);
        Err(ArgError::CountMismatch(n, arg_count))
    }
}
