use winnow::combinator::{dispatch, opt, repeat, success};
use winnow::error::ErrMode::{Backtrack, Cut};
use winnow::stream::Stream;
use winnow::token::any;
use winnow::trace::trace;
use winnow::{PResult, Parser};

use super::Error;
use crate::parser::stream::{tokens, Token, TokenType};
use crate::parser::{ast, Input};

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
    ($inner:expr, ')') => { winnow::combinator::terminated($inner, tokens::Rparen) };
    ($inner:expr, ']') => { winnow::combinator::terminated($inner, tokens::Rbracket) };
    ($inner:expr, '}') => { winnow::combinator::terminated($inner, tokens::Rcurly) };
}

#[inline(always)]
pub(super) fn ident(input: &mut Input) -> PResult<ast::IdentOffset, Error> {
    let start = input.next_start();
    match input.next_token() {
        Some(token!("ident")) => Ok(ast::IdentOffset { start }),
        got => Err(Backtrack(Error::Expected(TokenType::Ident, got.into()))),
    }
}
fn span_from_len(start: u32, len: usize) -> (u32, u32) {
    (start, start + u32::try_from(len).unwrap())
}
#[inline(always)]
pub(super) fn many_tts<const SPLIT_COMMA: bool>(input: &mut Input) -> PResult<(u32, u32), Error> {
    let start = input.next_start();
    repeat::<_, _, (), _, _>(1.., token_tree::<SPLIT_COMMA>)
        .recognize()
        .map(|v| span_from_len(start, v.len()))
        .parse_next(input)
}
#[inline(always)]
pub(super) fn token_tree<const SPLIT_COMMA: bool>(input: &mut Input) -> PResult<(), Error> {
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
