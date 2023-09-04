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
use winnow::combinator::{alt, dispatch, opt, repeat, success, terminated};
use winnow::combinator::{delimited as delim, peek, separated0};
use winnow::error::ErrMode::Backtrack;
use winnow::error::{ErrMode, FromExternalError};
use winnow::error::{ErrorKind, ParseError, ParserError};
use winnow::token::any;
use winnow::{stream::Stream, trace::trace, PResult, Parser};

use crate::lex::{tokens as t, Stateful, Token, TokenType};
use Token::{Comma, Equal, Ident, Lbracket, Lcurly, Lparen, String as TStr};

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("Expected '{0}' got '{1}'")]
    Expected(TokenType, TokenType),
    #[error("Unbalanced delimiter")]
    Unbalanced,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Following the statement identifier, expected a '{{' or '(', but got '{0}'")]
    StatementDelimiter(TokenType),
    #[error("Expected start of a statement, with an identifer or string literal, but got '{0}'")]
    StartStatement(TokenType),
    #[error("Found comma delimited in method arg")]
    EndComma,
}
impl<I: Itrp> ParserError<Input<'_, I>> for Error {
    fn from_error_kind(_: &Input<I>, _: ErrorKind) -> Self {
        // eprintln!("from_error_kind: '{_}'");
        // eprintln!("from_error_kind: {_}");
        Self::Unexpected
    }

    fn append(self, _: &Input<I>, _: ErrorKind) -> Self {
        // eprintln!("from_error_kind: '{_}'");
        // eprintln!("append: {_}");
        self
    }
}
impl<I: Itrp> FromExternalError<Input<'_, I>, Error> for Error {
    fn from_external_error(_: &Input<'_, I>, _: ErrorKind, e: Error) -> Self {
        // eprintln!("from_external_error: '{_}'");
        // eprintln!("from_external_error: {_}");
        e
    }
}

pub(crate) type Input<'i, T> = Stateful<'i, T>;
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
    mut input: Input<'i, impl Itrp>,
    mut f: impl FnMut(&'i [u8]),
) -> Result<(), ErrMode<Error>> {
    let elem_parser = many1(token_tree_impl::<true>).recognize().try_map(|arg| {
        f(arg);
        Ok(())
    });
    let list_parser = separated0(elem_parser, t::Comma);
    match terminated(list_parser, opt(t::Comma)).parse_next(&mut input) {
        Ok(()) if input.is_empty() => Ok(()),
        Ok(()) => Err(token_tree_impl::<true>(&mut input).unwrap_err()),
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
        Some(Comma) if SPLIT_COMMA => |_: &mut _| Err(Backtrack(Error::EndComma)),
        Some(Ident(_) | TStr(_) | Comma | Equal) => success(()),
        _ => |_: &mut _| Err(Backtrack(Error::Unbalanced)),
    };
    trace("token_tree", parser).parse_next(input)
}
#[inline(always)]
fn method(input: &mut Input<impl Itrp>) -> VoidResult {
    let state = input.state.clone();
    let argument = delim(t::Lparen, many(token_tree), t::Rparen);
    (ident, opt(argument).recognize())
        .with_span()
        .map(|i| state.t_method(i))
        .parse_next(input)
}
#[inline(always)]
fn statement(input: &mut Input<impl Itrp>) -> VoidResult {
    let state = input.state.clone();

    dispatch! { opt(any).with_span();
        (Some(Ident(b"code")), _) => delim(t::Lparen, ident, t::Rparen).with_span().map(|i| state.code(i)),
        (Some(TStr(name) | Ident(name)), span) => {
            if ![&b"Entity"[..], b"spawn"].contains(&name) {
                state.set_name(span, name);
            }
            trace("statement_tail", statement_tail)
        },
        (bad_token, _) => |_: &mut _| Err(Backtrack(Error::StartStatement(bad_token.into()))),
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
        bad_token => Err(Backtrack(Error::StatementDelimiter(bad_token.ok().into()))),
    }
}

type CResult<'i, T, I> = Result<T, ParseError<Input<'i, I>, Error>>;
#[inline(never)]
pub(crate) fn chirp_document<I: Itrp>(input: Input<I>) -> CResult<(), I> {
    // TODO(feat) non-entity statements
    trace("statement", statement).parse(input)
}

pub(crate) trait Itrp: Debug + Clone {
    fn code(&self, input: (&[u8], Range<usize>));
    fn set_name(&self, span: Range<usize>, name: &[u8]);
    fn complete_children(&self);
    fn method(&self, span: Range<usize>, name: &[u8], args: &[u8]);
    fn t_method(&self, ((method, args), span): ((&[u8], &[u8]), Range<usize>)) {
        self.method(span, method, args);
    }
    fn spawn_with_children(&self);
    fn insert_entity(&self) {
        self.spawn_with_children();
        self.complete_children();
    }
}
impl Itrp for () {
    fn code(&self, _: (&[u8], Range<usize>)) {}
    fn set_name(&self, _: Range<usize>, _: &[u8]) {}
    fn complete_children(&self) {}
    fn method(&self, _: Range<usize>, _: &[u8], _: &[u8]) {}
    fn spawn_with_children(&self) {}
}
