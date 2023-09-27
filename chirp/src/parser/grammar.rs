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
#![allow(clippy::inline_always)]
// allow: The generated code is fine, it's in line with how winnow does things
// internally.

use std::fmt::Debug;

use anyhow::Result;
use winnow::combinator::{alt, cut_err, dispatch, opt, repeat, success, terminated};
use winnow::combinator::{delimited as delim, peek, separated0};
use winnow::error::ErrMode::{Backtrack, Cut};
use winnow::error::ParserError;
use winnow::{stream::Stream, token::any, trace::trace, PResult, Parser};

use super::stream::{tokens as t, Input, SpannedExt, Token, TokenType};
use super::{Error, Itrp, Span};
use Token::{
    Comma, Equal, Ident, Lbracket, Lcurly, Lparen, Rbracket, Rcurly, Rparen, String as TStr,
};

type VoidResult = PResult<(), Error>;

fn many<I: Stream, O, E: ParserError<I>>(f: impl Parser<I, O, E>) -> impl Parser<I, (), E> {
    repeat(0.., f)
}

fn many1<I: Stream, O, E: ParserError<I>>(f: impl Parser<I, O, E>) -> impl Parser<I, (), E> {
    repeat(1.., f)
}

#[inline(always)]
fn ident<'i>(input: &mut Input<'i, impl Itrp>) -> PResult<&'i [u8], Error> {
    match input.next_token() {
        Some(Token::Ident(ident)) => Ok(ident),
        got => Err(Backtrack(Error::Expected(TokenType::Ident, got.into()))),
    }
}
pub(crate) fn arg_token_tree<'i>(
    input: &mut Input<'i, impl Itrp>,
    mut f: impl FnMut(&'i [u8]),
) -> PResult<(), Error> {
    let elem_parser = many1(token_tree_impl::<true>).recognize().try_map(|arg| {
        f(arg);
        Ok(())
    });
    let list_parser = separated0(elem_parser, t::Comma);
    match terminated(list_parser, opt(t::Comma)).parse_next(input) {
        Ok(()) if input.is_empty() => Ok(()),
        Ok(()) => Err(token_tree_impl::<true>(input).unwrap_err()),
        Err(err) => Err(err),
    }
}
fn token_tree(input: &mut Input<impl Itrp>) -> VoidResult {
    token_tree_impl::<false>(input)
}
#[inline(always)]
fn token_tree_impl<const SPLIT_COMMA: bool>(input: &mut Input<impl Itrp>) -> VoidResult {
    let parser = dispatch! { opt(any);
        Some(Lparen) => terminated(many(token_tree), t::Rparen),
        Some(Lbracket) => terminated(many(token_tree), t::Rbracket),
        Some(Lcurly) => terminated(many(token_tree), t::Rcurly),
        Some(Comma) if SPLIT_COMMA => |_: &mut _| Err(Backtrack(Error::Unexpected)),
        Some(Ident(_) | TStr(_) | Comma | Equal) => success(()),
        None | Some(Rparen | Rbracket | Rcurly) => |_: &mut _| Err(Backtrack(Error::Unexpected)),
        _ => |_: &mut _| Err(Cut(Error::Unbalanced)),
    };
    trace("token_tree", parser).parse_next(input)
}
#[inline(always)]
fn method(input: &mut Input<impl Itrp>) -> VoidResult {
    let state = input.state.clone();
    let argument = delim(t::Lparen, many(token_tree), cut_err(t::Rparen));
    let ident = |i: &mut _| match opt(any).parse_next(i) {
        Ok(Some(Ident(ident))) => Ok(ident),
        Ok(Some(Rparen)) => Err(Backtrack(Error::Unexpected)),
        Ok(any_else) => Err(Cut(Error::BadMethod(any_else.into()))),
        Err(err) => Err(err),
    };
    (ident.spanned(), opt(argument).recognize().spanned())
        .map(|i| state.t_method(i))
        .parse_next(input)
}
#[inline(always)]
fn statement(input: &mut Input<impl Itrp>) -> VoidResult {
    let state = input.state.clone();

    dispatch! { opt(any).spanned();
        (Some(Ident(b"code")), _) => delim(t::Lparen, ident, t::Rparen).spanned().map(|i| state.code(i)),
        (Some(TStr(name) | Ident(name)), span) => {
            if ![&b"Entity"[..], b"spawn"].contains(&name) {
                state.set_name(span, name);
            }
            trace("statement_tail", statement_tail)
        },
        (Some(Rcurly), _) => |_: &mut _| Err(Backtrack(Error::Unexpected)),
        (bad_token, _) => |_: &mut _| Err(Cut(Error::StartStatement(bad_token.into()))),
    }
    .parse_next(input)
}
#[inline(always)]
fn statements(input: &mut Input<impl Itrp>) -> VoidResult {
    input.state.spawn_with_children();
    many(statement).parse_next(input)?;
    input.state.complete_children();
    Ok(())
}
#[inline(always)]
fn statement_tail(input: &mut Input<impl Itrp>) -> VoidResult {
    let state = input.state.clone();

    let methods = delim(t::Lparen, many(method), t::Rparen);
    let mut statements = delim(t::Lcurly, statements, t::Rcurly);
    let spawn_leaf = success(()).map(|_| state.insert_entity());
    match peek(any::<_, ()>).parse_next(input) {
        Ok(Lparen) => terminated(methods, alt((statements, spawn_leaf))).parse_next(input),
        Ok(Lcurly) => statements.parse_next(input),
        bad_token => Err(Cut(Error::StatementDelimiter(bad_token.ok().into()))),
    }
}

#[derive(Debug)]
pub(crate) struct SpanError {
    pub(crate) span: Span,
    pub(crate) error: Error,
}
pub(crate) fn chirp_document<I: Itrp>(mut input: Input<I>) -> Result<(), SpanError> {
    // TODO(feat) non-entity statements
    match statement(&mut input) {
        Ok(()) if input.is_empty() => Ok(()),
        Ok(()) => {
            let error_start = input.next_start();
            let error = statement(&mut input).unwrap_err().into_inner().unwrap();
            let error_end = input.next_start();
            Err(SpanError { span: (error_start, error_end), error })
        }
        Err(err) => {
            let error_start = input.next_start();
            let error_end = error_start;
            let error = err.into_inner().unwrap();
            Err(SpanError { span: (error_start, error_end), error })
        }
    }
}
