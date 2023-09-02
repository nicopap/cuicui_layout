//! The chirp grammar, defined using winnow parsers.
//!
//! The current implementation is not done with a particular look for performance,
//! just functionality and readability.
//!
//! ```text
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

pub(crate) type Input<'i, T> = Stateful<'i, T>;

fn repeat0<I: Stream, O, E: ParserError<I>>(f: impl Parser<I, O, E>) -> impl Parser<I, (), E> {
    repeat(.., f)
}

fn line_comment(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    let parser = preceded(b"//", take_till0(b'\n')).void();
    trace("line_comment", parser).parse_next(input)
}
/// Whitespace, thing between token, including comments.
pub(crate) fn whitespace(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    let parser = separated0(multispace1, line_comment);
    trace("whitespace", parser).parse_next(input)
}

/// `D` followed by [`whitespace`].
fn ws<const D: u8>(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    let parser = terminated(D, whitespace);
    trace("ws", parser).void().parse_next(input)
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

const ESCAPABLES: [u8; 7] = *b"\\\"'ntru";

/// The inside of a `D`-delimited string, does not consume the terminating token.
fn string_inner<'i, const D: u8>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    let parser = escaped(take_till1(&[b'\\', D]), '\\', one_of(ESCAPABLES));
    trace("string_inner", parser).recognize().parse_next(input)
}

/// A string delimited by either `'` or `"`.
fn string<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    let parser = dispatch! { any;
        b'"' => terminated(string_inner::<b'"'>, b'"'),
        b'\'' => terminated(string_inner::<b'\''>, b'\''),
        _ => fail
    };
    trace("string", parser).recognize().parse_next(input)
}

fn ident<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    let parser = terminated(take_till1(b"=[]{}()'\" \t\n"), whitespace);
    trace("ident", parser).parse_next(input)
}
fn arg_tt_ident<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    trace("arg_tt_ident", take_till1(b",[]{}()'\" \t\n")).parse_next(input)
}

pub(crate) fn arg_token_tree<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    let delimited = dispatch! { any;
        b'(' => delimited(whitespace, repeat0(token_tree), b')'),
        b'[' => delimited(whitespace, repeat0(token_tree), b']'),
        b'{' => delimited(whitespace, repeat0(token_tree), b'}'),
        b'"' => terminated(string_inner::<b'"'>, b'"').void(),
        b'\'' => terminated(string_inner::<b'\''>, b'\'').void(),
        _ => fail,
    };
    let parser = alt((arg_tt_ident.void(), delimited));
    trace("arg_token_tree", parser)
        .recognize()
        .parse_next(input)
}

fn token_tree<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], ()> {
    let delimited = dispatch! { any;
        b'(' => ws_terminated(repeat0(token_tree), b')'),
        b'[' => ws_terminated(repeat0(token_tree), b']'),
        b'{' => ws_terminated(repeat0(token_tree), b'}'),
        b'"' => terminated(string_inner::<b'"'>, ws::<b'"'>).void(),
        b'\'' => terminated(string_inner::<b'\''>, ws::<b'\''>).void(),
        _ => fail,
    };
    let parser = alt((b'='.void(), ident.void(), delimited));
    trace("token_tree", parser).recognize().parse_next(input)
}

fn method(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    let parser = delimited(ws::<b'('>, repeat0(token_tree), b')').recognize();
    let method_argument = trace("method_argument", terminated(parser, whitespace));
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
            input.state.set_name(span, None);
            statement_tail(input)
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
        (str, span) if str.first() == Some(&b'"') => {
            input.state.set_name(span, Some(&str[1..str.len() - 1]));
            statement_tail(input)
        }
        (identifier, span) => {
            input.state.set_name(span, Some(identifier));
            statement_tail(input)
        }
    }
}

fn statements(input: &mut Input<impl Itrp>) -> PResult<(), ()> {
    input.state.spawn_with_children();
    repeat0(statement).parse_next(input)?;
    input.state.complete_children();
    Ok(())
}
fn statement_tail<I: Itrp>(input: &mut Input<I>) -> PResult<(), ()> {
    let mut methods = delimited(ws::<b'('>, repeat0(method), ws::<b')'>);
    let mut statements = delimited(ws::<b'{'>, statements, ws::<b'}'>);
    let has_methods = methods.parse_next(input).is_ok();
    if has_methods {
        let no_children = |i: &mut Input<I>| {
            i.state.insert_entity();
            Ok(())
        };
        alt((statements, no_children)).void().parse_next(input)
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
    fn set_name(&self, span: Range<usize>, name: Option<&[u8]>);
    fn complete_children(self);
    fn method(&self, span: Range<usize>, name: &[u8], args: Option<&[u8]>);
    fn spawn_with_children(&self);
    fn insert_entity(&self) {
        self.spawn_with_children();
        self.complete_children();
    }
}
impl Itrp for () {
    fn code(&self, _: (&[u8], Range<usize>)) {}
    fn set_name(&self, _: Range<usize>, _: Option<&[u8]>) {}
    fn complete_children(self) {}
    fn method(&self, _: Range<usize>, _: &[u8], _: Option<&[u8]>) {}
    fn spawn_with_children(&self) {}
}
