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
//!    = 'code'      '(' 'ident' ')'
//!    | 'Entity'    StatementTail
//!    | 'ident' '!' '(' (TokenTree (',' TokenTree)*)? ')' (StatementTail)?
//!    | 'ident'     StatementTail
//!    | StringLit   StatementTail
//!
//! StatementTail
//!    = '(' (Method)* ')' ('{' (Statement)* '}')?
//!    | '{' (Statement)* '}'
//!
//! Path = 'ident' ('/' 'ident')*
//! Use = 'use' Path ('as' 'ident')?
//! Fn = ('pub')? 'fn' 'ident' '(' ('ident' (',' 'ident')*)? ')' '{' Statement '}'
//! ChirpFile = (Use)* (Fn)* Statement
//! ```
#![allow(clippy::inline_always)]
// allow: The generated code is fine, it's in line with how winnow does things
// internally.

// TODO(perf): Use a single buffer we extend imperatively.
// Currently we use a heap-heavy AST.
// What we should do instead is have a single `&mut Vec` and pass it around
// (most likely by defining all parsing methods functions as methods on a struct
// that holds the `&mut Vec`)
// Then parse imperatively the input stream.
// The neat thing about our grammar is that it never requires meaningfull backtracking.
// Any backtracing will lead to an error.

use anyhow::Result;
use winnow::combinator::{dispatch, opt, repeat, separated0, success};
use winnow::error::ErrMode::{Backtrack, Cut};
use winnow::{stream::Stream, token::any, trace::trace, PResult, Parser};

use super::ast::{Argument, ChirpFile, Function, Import, Method, Node, Statement, Template};
use super::ast::{IdentOffset, OptNameOffset};
use super::stream::{tokens, Input, Token, TokenType};
use super::{Error, Span};

#[cfg(test)]
mod tests;

pub fn many<'i, O, P>(parser: P) -> impl Parser<Input<'i>, Vec<O>, Error>
where
    P: Parser<Input<'i>, O, Error>,
{
    repeat(.., parser)
}
pub fn sep<'i, O, P>(parser: P) -> impl Parser<Input<'i>, Vec<O>, Error>
where
    P: Parser<Input<'i>, O, Error>,
{
    separated0(parser, tokens::Comma)
}

#[rustfmt::skip]
macro_rules! token {
    ($first:tt $(| $many:tt)*) => { token!(@ $first) $(| token!(@ $many))* };
    (@ "ident")  => { Token::Ident(_) };
    (@ "string") => { Token::String(_) };
    (@ '(') => { Token::Lparen };
    (@ ')') => { Token::Rparen };
    (@ '{') => { Token::Lcurly };
    (@ '}') => { Token::Rcurly };
    (@ '[') => { Token::Lbracket };
    (@ ']') => { Token::Rbracket };
    (@ ',') => { Token::Comma };
    (@ '=') => { Token::Equal };
}
#[rustfmt::skip]
macro_rules! tokens {
    ('(' $inner:expr, ')') => { winnow::combinator::delimited(tokens::Lparen, $inner, tokens::Rparen) };
    ('{' $inner:expr, '}') => { winnow::combinator::delimited(tokens::Lcurly, $inner, tokens::Rcurly) };
    ("fn" $inner:expr )    => { winnow::combinator::preceded(tokens::Fn, $inner) };
    ("use" $inner:expr)    => { winnow::combinator::preceded(tokens::Use, $inner) };
    ("as" $inner:expr)     => { winnow::combinator::preceded(tokens::As, $inner) };
    ($inner:expr, ')')     => { winnow::combinator::terminated($inner, tokens::Rparen) };
    ($inner:expr, ']')     => { winnow::combinator::terminated($inner, tokens::Rbracket) };
    ($inner:expr, '}')     => { winnow::combinator::terminated($inner, tokens::Rcurly) };
}

#[inline(always)]
fn ident(input: &mut Input) -> PResult<IdentOffset, Error> {
    let start = input.next_start();
    match input.next_token() {
        Some(token!("ident")) => Ok(IdentOffset::new(start)),
        got => Err(Backtrack(Error::Expected(TokenType::Ident, got.into()))),
    }
}
#[inline(always)]
pub fn many_tts<const SPLIT_COMMA: bool>(input: &mut Input) -> PResult<Argument, Error> {
    let start = input.next_start();
    repeat::<_, _, (), _, _>(1.., token_tree::<SPLIT_COMMA>)
        .recognize()
        .map(|v| Argument::new(start, v.len()))
        .parse_next(input)
}
#[inline(always)]
fn token_tree<const SPLIT_COMMA: bool>(input: &mut Input) -> PResult<(), Error> {
    let parser = dispatch! { opt(any);
        Some(token!('(')) => tokens!(many_tts::<false>.void(), ')'),
        Some(token!('[')) => tokens!(many_tts::<false>.void(), ']'),
        Some(token!('{')) => tokens!(many_tts::<false>.void(), '}'),
        Some(token!(',')) if SPLIT_COMMA => |_: &mut _| Err(Backtrack(Error::Unexpected)),
        Some(token!("ident" | "string" | ',' | '=')) => success(()),
        None | Some(token!(')' | ']' | '}')) => |_: &mut _| Err(Backtrack(Error::Unexpected)),
        _ => |_: &mut _| Err(Cut(Error::Unbalanced)),
    };
    trace("token_tree", parser).parse_next(input)
}

#[inline(always)]
fn method(input: &mut Input) -> PResult<Method, Error> {
    let sep = |parser| separated0(parser, tokens::Comma);
    let args = tokens!('(' sep(many_tts::<true>), ')');
    let args = opt(args).map(Option::unwrap_or_default);
    (ident, args).map(Method::new).parse_next(input)
}

#[inline(always)]
fn template(name: IdentOffset, input: &mut Input) -> PResult<Template, Error> {
    let args = tokens!('(' sep(many_tts::<true>), ')');
    let methods = opt(tokens!('(' many(method), ')')).map(Option::unwrap_or_default);
    let statements = opt(tokens!('{' many(node), '}')).map(Option::unwrap_or_default);
    (success(name), args, methods, statements)
        .map(Template::new)
        .parse_next(input)
}
#[inline(always)]
fn statement(name: OptNameOffset, input: &mut Input) -> PResult<Statement, Error> {
    let statements = opt(tokens!('{' many(node), '}')).map(Option::unwrap_or_default);

    match any::<_, Error>.parse_next(input) {
        Ok(token!('(')) => (success(name), tokens!(many(method), ')'), statements)
            .map(Statement::both)
            .parse_next(input),
        Ok(token!('{')) => (success(name), tokens!(many(node), '}'))
            .map(Statement::children)
            .parse_next(input),
        bad_token => Err(Cut(Error::StatementDelimiter(bad_token.ok().into()))),
    }
}
#[inline(always)]
fn node(input: &mut Input) -> PResult<Node, Error> {
    use Token::{Ident, String as TStr};

    let start = input.next_start();
    match any::<_, Error>.parse_next(input) {
        Ok(Ident(b"code")) => tokens!('(' ident, ')').map(Node::Code).parse_next(input),
        Ok(Ident(name)) if name.ends_with(b"!") => {
            template(start.into(), input).map(Node::Template)
        }
        Ok(TStr(name) | Ident(name)) => {
            let name = (![b"Entity", &b"spawn"[..]].contains(&name)).then_some(start);
            statement(name.into(), input).map(Node::Statement)
        }
        Ok(token!('}')) => Err(Backtrack(Error::Unexpected)),
        bad_token => Err(Cut(Error::StartStatement(bad_token.ok().into()))),
    }
}

#[inline(always)]
fn import(input: &mut Input) -> PResult<Import, Error> {
    let alias = opt(tokens!("as" ident));
    tokens!("use"(ident, alias))
        .map(Import::new)
        .parse_next(input)
}

#[inline(always)]
fn function(input: &mut Input) -> PResult<Function, Error> {
    let arguments = tokens!('(' sep(ident), ')');
    let body = tokens!('{' node, '}');
    tokens!("fn"(ident, arguments, body))
        .map(Function::new)
        .parse_next(input)
}

pub(crate) fn chirp_file(mut input: Input) -> Result<ChirpFile, (Error, Span)> {
    let result = (many(import), many(function), node)
        .map(ChirpFile::new)
        .parse_next(&mut input);

    let offset = input.current_offset();

    match result {
        Ok(chirp_file) if input.is_empty() => Ok(chirp_file),
        Ok(_) => Err((Error::TrailingText, (offset, offset))),
        Err(Cut(err) | Backtrack(err)) => Err((err, (offset, offset))),
        _ => unreachable!(),
    }
}
