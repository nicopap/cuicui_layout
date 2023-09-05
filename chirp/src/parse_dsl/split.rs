//! Split method arguments.
//!
//! See [`split`].

use std::str::from_utf8_unchecked;

use thiserror::Error;
use winnow::error::ErrMode::{Backtrack, Cut, Incomplete};

use crate::parser::{self, arg_token_tree, Input};

/// Error returned by one of the `argN` functions.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArgError {
    #[error("Expected {0} arguments, got {}{1}", if .0 == &255 { "more than " } else {""})]
    CountMismatch(u8, u8),
    #[error("Parser error [{1}]: {0}")]
    ArgParse(parser::Error, u32),
}
impl ArgError {
    #[inline]
    fn count_mismatch(expected: u8, got: usize) -> Self {
        let got = u8::try_from(got).unwrap_or(255);
        ArgError::CountMismatch(expected, got)
    }
    pub(crate) const fn maybe_offset(&self) -> Option<u32> {
        match self {
            Self::CountMismatch(..) => None,
            Self::ArgParse(_, offset) => Some(*offset),
        }
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
/// use cuicui_chirp::parse_dsl::{split, ArgError::CountMismatch};
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
pub fn split<const N: usize>(input: &str) -> Result<[&str; N], ArgError> {
    let mut ret = [""; N];
    let n = u8::try_from(N).unwrap();

    // TODO(clean): This "init" is just weird and surprising.
    let (init, input) = match () {
        () if N == 0 && (input.is_empty() || input == "()") => {
            return Ok(ret);
        }
        () if input.starts_with('(') && input.ends_with(')') => (1, &input[1..input.len() - 1]),
        () => (0, input),
    };
    let mut arg_count = 0;
    let mut input = Input::new(input.trim().as_bytes(), ());
    let maybe_error = arg_token_tree(&mut input, |arg| {
        let str_arg = unsafe { from_utf8_unchecked(arg) };
        if arg_count < N {
            ret[arg_count] = str_arg.trim();
        }
        arg_count += 1;
    });
    match maybe_error {
        Ok(()) if arg_count == N => Ok(ret),
        Ok(()) => Err(ArgError::count_mismatch(n, arg_count)),
        Err(Backtrack(err) | Cut(err)) => Err(ArgError::ArgParse(err, init + input.next_start())),
        Err(Incomplete(_)) => unreachable!("We created the input, and we know it is not partial"),
    }
}
