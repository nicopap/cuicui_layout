//! The chirp grammar, defined using winnow parsers.
//!
//! The current implementation is not done with a particular look for performance,
//! just functionality and readability.
//!
//! ```
//! TokenTree
//!    = 'ident'
//!    | '(' (TokenTree)* ')'
//!    | '[' (TokenTree)* ']'
//!    | '{' (TokenTree)* '}'
//!    | StringLit
//!
//! Method = 'ident' ('(' (TokenTree)* ')')?
//!
//! Statement
//!    = 'code'    '(' 'ident' ')'
//!    | 'Entity'  StatementTail
//!    | 'ident'   StatementTail
//!    | StringLit StatementTail
//!
//! StatementTail
//!    = '(' (Method)* ')' ('{' (Statement)* '}')?
//!    | '{' (Statement)* '}'
//! ```

use std::{fmt::Debug, ops::Range};

use anyhow::Result;
use winnow::ascii::{escaped, multispace1};
use winnow::combinator::{alt, dispatch, fail};
use winnow::combinator::{delimited, opt, preceded, repeat, separated0, terminated};
use winnow::error::{ParseError, ParserError};
use winnow::token::{any, one_of, take_till0, take_till1};
use winnow::{stream::Stream, trace::trace, PResult, Parser};

use crate::lex::Stateful;

type Input<'i, T> = Stateful<'i, T>;

fn repeat0<I: Stream, O, E: ParserError<I>>(f: impl Parser<I, O, E>) -> impl Parser<I, (), E> {
    repeat(.., f)
}

fn line_comment(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    preceded(b"//", take_till0(b'\n')).void().parse_next(input)
}
/// Whitespace, thing between token, including comments.
fn whitespace(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    separated0(multispace1, line_comment).parse_next(input)
}

/// `D` followed by [`whitespace`].
fn ws<const D: u8>(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    terminated(D, whitespace).void().parse_next(input)
}

fn ws_terminated<'i, I: Itrp, O, F>(
    mut first: F,
    mut second: u8,
) -> impl Parser<Input<'i, I>, O, ()>
where
    F: Parser<Input<'i, I>, O, ()>,
{
    trace("ws_terminated", move |input: &mut _| {
        let _ = whitespace.parse_next(input);
        let o1 = first.parse_next(input)?;
        let _ = second.parse_next(input)?;
        whitespace.parse_next(input)?;
        Ok(o1)
    })
}

/// The inside of a `D`-delimited string, does not consume the terminating token.
fn string_inner<'i, const D: u8>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    escaped(take_till1(&[b'\\', D]), '\\', one_of(&[b'\\', D]))
        .recognize()
        .parse_next(input)
}

/// A string delimited by either `'` or `"`.
fn string<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    let dispatch = dispatch! { any;
        b'"' => terminated(string_inner::<b'"'>, b'"'),
        b'\'' => terminated(string_inner::<b'\''>, b'\''),
        _ => fail
    };
    dispatch.recognize().parse_next(input)
}

fn ident<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    terminated(take_till1(b"[]{}()'\" \t\n"), whitespace).parse_next(input)
}

fn token_tree<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    // We could use the following instead of inserting `ws` everywhere,
    // if not for the fact we have to manage comments.
    // let non_delimiter = not (b"()[]{}\"'");
    let delimited = dispatch! { any;
        b'(' => ws_terminated(repeat0(token_tree), b')'),
        b'[' => ws_terminated(repeat0(token_tree), b']'),
        b'{' => ws_terminated(repeat0(token_tree), b'}'),
        b'"' => terminated(string_inner::<b'"'>, ws::<b'"'>).void(),
        b'\'' => terminated(string_inner::<b'\''>, ws::<b'\''>).void(),
        _ => fail,
    };
    alt((ident.void(), delimited)).recognize().parse_next(input)
}

fn method(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    let method_argument = delimited(ws::<b'('>, repeat0(token_tree), ws::<b')'>).recognize();
    let ((name, args), span) = (ident, opt(method_argument))
        .with_span()
        .parse_next(input)?;
    input.state.method(span, name, args);
    Ok(())
}

fn statement(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    match alt((ident, string)).with_span().parse_next(input)? {
        (b"code", _) => {
            let code_ident = delimited(b'(', ident, ws::<b')'>)
                .with_span()
                .parse_next(input)?;
            input.state.code(code_ident);
            Ok(())
        }
        (b"Entity" | b"spawn", span) => {
            input.state.entity(span, None);
            statement_tail.parse_next(input)
        }
        (
            b"abstract" | b"as" | b"async" | b"await" | b"become" | b"box" | b"break" | b"const"
            | b"continue" | b"crate" | b"do" | b"dyn" | b"else" | b"enum" | b"extern" | b"false"
            | b"final" | b"fn" | b"for" | b"if" | b"impl" | b"in" | b"let" | b"loop" | b"macro"
            | b"match" | b"mod" | b"move" | b"mut" | b"override" | b"priv" | b"pub" | b"ref"
            | b"return" | b"self" | b"static" | b"struct" | b"super" | b"trait" | b"true" | b"try"
            | b"type" | b"typeof" | b"unsafe" | b"unsized" | b"use" | b"virtual" | b"where"
            | b"while" | b"yeet",
            _,
        ) => fail.parse_next(input),
        (string, span) if string.first() == Some(&b'"') => {
            input.state.entity(span, Some(&string[1..string.len() - 1]));
            statement_tail.parse_next(input)
        }
        (identifier, span) => {
            input.state.entity(span, Some(identifier));
            statement_tail.parse_next(input)
        }
    }
}

fn statements(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    input.state.spawn();
    repeat0(statement).parse_next(input)?;
    input.state.complete();
    Ok(())
}
fn statement_tail(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    let mut methods = delimited(ws::<b'('>, repeat0(method), ws::<b')'>);
    let mut statements = delimited(ws::<b'{'>, statements, ws::<b'}'>);
    let has_methods = methods.parse_next(input).is_ok();
    if has_methods {
        opt(statements).void().parse_next(input)
    } else {
        statements.parse_next(input)
    }
}

type CResult<'i, T, I> = Result<T, ParseError<Input<'i, I>, ()>>;
pub(crate) fn chirp_document<I: Itrp>(mut input: Input<I>) -> CResult<(), I> {
    // TODO(feat) non-entity statements
    let _ = line_comment(&mut input);
    whitespace.parse_next(&mut input).unwrap();
    statement.parse(input)
}

pub(crate) trait Itrp: Debug + Copy {
    fn code(&self, input: (&[u8], Range<usize>));
    fn entity(&self, span: Range<usize>, name: Option<&[u8]>);
    fn complete(self);
    fn method(&self, span: Range<usize>, name: &[u8], args: Option<&[u8]>);
    fn spawn(&self);
    fn spawn_leaf(&self) {
        self.spawn();
    }
}
