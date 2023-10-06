#![allow(clippy::match_bool, clippy::inline_always)]
use std::{fmt, marker::PhantomData, slice};

use winnow::error::Needed;
use winnow::stream::{Location, Offset, Stream, StreamIsPartial};
use winnow::{BStr, Parser};

use super::lex;

#[inline(always)]
fn as_usize(u32: u32) -> usize {
    u32.try_into()
        .expect("==OPTIMIZEDOUT== This never happens as_usize")
}
#[inline(always)]
fn as_u32(usize: usize) -> u32 {
    debug_assert!(u32::try_from(usize).is_ok());
    // SAFETY: not really safe. We are using `as_u32` on slice lengths. This
    // effectively becomes a problem only if a .chirp file is larger than 4GB.
    unsafe { u32::try_from(usize).unwrap_unchecked() }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Token<'i> {
    Equal,
    Lparen,
    Rparen,
    Lcurly,
    Rcurly,
    Lbracket,
    Rbracket,
    Comma,
    Reserved(&'i [u8]),
    Ident(&'i [u8]),
    String(&'i [u8]),
}
impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Equal => TokenType::from(Some(Token::Equal)).fmt(f),
            Token::Lparen => TokenType::from(Some(Token::Lparen)).fmt(f),
            Token::Rparen => TokenType::from(Some(Token::Rparen)).fmt(f),
            Token::Lcurly => TokenType::from(Some(Token::Lcurly)).fmt(f),
            Token::Rcurly => TokenType::from(Some(Token::Rcurly)).fmt(f),
            Token::Lbracket => TokenType::from(Some(Token::Lbracket)).fmt(f),
            Token::Rbracket => TokenType::from(Some(Token::Rbracket)).fmt(f),
            Token::Comma => TokenType::from(Some(Token::Comma)).fmt(f),
            Token::Reserved(bytes) => f.debug_tuple("Reserved").field(&BStr::new(bytes)).finish(),
            Token::Ident(bytes) => f.debug_tuple("Ident").field(&BStr::new(bytes)).finish(),
            Token::String(bytes) => f.debug_tuple("String").field(&BStr::new(bytes)).finish(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Equal,
    Lparen,
    Rparen,
    Lcurly,
    Rcurly,
    Lbracket,
    Rbracket,
    Comma,
    Reserved,
    Ident,
    String,
    Code,
    As,
    Use,
    Fn,
    None,
}
impl From<Option<Token<'_>>> for TokenType {
    #[inline(always)]
    fn from(value: Option<Token<'_>>) -> Self {
        match value {
            Some(Token::Equal) => TokenType::Equal,
            Some(Token::Lparen) => TokenType::Lparen,
            Some(Token::Rparen) => TokenType::Rparen,
            Some(Token::Lcurly) => TokenType::Lcurly,
            Some(Token::Rcurly) => TokenType::Rcurly,
            Some(Token::Lbracket) => TokenType::Lbracket,
            Some(Token::Rbracket) => TokenType::Rbracket,
            Some(Token::Comma) => TokenType::Comma,
            Some(Token::Reserved(_)) => TokenType::Reserved,
            Some(Token::Ident(_)) => TokenType::Ident,
            Some(Token::String(_)) => TokenType::String,
            None => TokenType::None,
        }
    }
}
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nice_token = match self {
            TokenType::Equal => "'='",
            TokenType::Lparen => "'('",
            TokenType::Rparen => "')'",
            TokenType::Lcurly => "'{'",
            TokenType::Rcurly => "'}'",
            TokenType::Lbracket => "'['",
            TokenType::Rbracket => "']'",
            TokenType::Comma => "','",
            TokenType::Reserved => "a reserved keyword",
            TokenType::Ident => "an identifier",
            TokenType::Code => "'code'",
            TokenType::Fn => "'fn'",
            TokenType::Use => "'use'",
            TokenType::As => "'as'",
            TokenType::String => "\"a string literal\"",
            TokenType::None => "nothing, the end of file",
        };
        f.write_str(nice_token)
    }
}
impl Token<'_> {
    pub(crate) fn len(&self) -> u32 {
        use Token::{Comma, Equal, Lbracket, Lcurly, Lparen, Rbracket, Rcurly, Rparen};
        match self {
            Comma | Equal | Lparen | Rparen | Lcurly | Rcurly | Lbracket | Rbracket => 1,
            Token::Reserved(s) | Token::Ident(s) | Token::String(s) => as_u32(s.len()),
        }
    }
}

/// Generate parsers for individual tokens.
///
/// A previous version used `&'static str`, matched on the `str` and picked the
/// parsed based on that. But the optimizer was not good enough to understand
/// that a parser for a static value `&str` could only match a single branch of
/// the `match`.
///
/// The approach where each token has its own ZST seems to compile to much
/// happier code.
macro_rules! grammar {
    ($( $name:ident ),* $(,)?) => {
        $(
        #[derive(Default, Clone, Copy)]
        pub struct $name;
        impl<'i, S: Clone + fmt::Debug> Parser<Input<'i, S>, Token<'i>, Error> for $name {
            #[inline(always)]
            #[cfg(not(feature = "trace_lexer"))]
            fn parse_next(&mut self, input: &mut Input<'i, S>) -> PResult<Token<'i>, Error> {
                match input.next_token() {
                    Some(token @ Token::$name) => Ok(token),
                    got => Err(Backtrack(Error::Expected(TokenType::$name, got.into()))),
                }
            }
            #[cfg(feature = "trace_lexer")]
            fn parse_next(&mut self, input: &mut Input<'i, S>) -> PResult<Token<'i>, Error> {
                let parser = |input: &mut Input<'i, S>| match input.next_token() {
                    Some(token @ Token::$name) => Ok(token),
                    got => Err(Backtrack(Error::Expected(TokenType::$name, got.into()))),
                };
                winnow::trace::trace(TokenType::$name.to_string(), parser).parse_next(input)
            }
        }
        )*
    }
}
macro_rules! grammar_identifiers {
    ($( $identifier:literal as $name:ident ),* $(,)?) => {
        $(
        #[derive(Default, Clone, Copy)]
        pub struct $name;
        impl<'i, S: Clone + fmt::Debug> Parser<Input<'i, S>, Token<'i>, Error> for $name {
            #[inline(always)]
            #[cfg(not(feature = "trace_lexer"))]
            fn parse_next(&mut self, input: &mut Input<'i, S>) -> PResult<Token<'i>, Error> {
                match input.next_token() {
                    Some(token @ Token::Ident($identifier)) => Ok(token),
                    got => Err(Backtrack(Error::Expected(TokenType::$name, got.into()))),
                }
            }
            #[cfg(feature = "trace_lexer")]
            fn parse_next(&mut self, input: &mut Input<'i, S>) -> PResult<Token<'i>, Error> {
                let parser = |input: &mut Input<'i, S>| match input.next_token() {
                    Some(token @ Token::Ident($identifier)) => Ok(token),
                    got => Err(Backtrack(Error::Expected(TokenType::$name, got.into()))),
                };
                winnow::trace::trace(TokenType::$name.to_string(), parser).parse_next(input)
            }
        }
        )*
    }
}

#[allow(clippy::wildcard_imports)]
pub mod tokens {
    use super::super::Error;
    use super::*;
    use winnow::{error::ErrMode::Backtrack, PResult};

    grammar![Equal, Lparen, Rparen, Lcurly, Rcurly, Lbracket, Rbracket, Comma];
    grammar_identifiers![b"as" as As, b"use" as Use, b"fn" as Fn, b"code" as Code];
}

pub struct TokenIter<'i, S> {
    stream: Input<'i, S>,
}
impl<'i, S: Clone + fmt::Debug> Iterator for TokenIter<'i, S> {
    type Item = (usize, Token<'i>);
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.stream.next_token()?;
        let offset = self.stream.start;
        let token_start_offset = offset - token.len();
        Some((as_usize(token_start_offset), token))
    }
}

/// [`winnow::Parser`] checkpoint for [`Input`].
#[derive(Debug, Clone, Copy)]
pub struct StateCheckpoint {
    end: u32,
    start: u32,
}
impl StateCheckpoint {
    pub const fn start(self) -> u32 {
        self.start
    }
}

/// Custom stream impl.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Input<'i, S = ()> {
    // TODO(clean): replace with &'i BStr when new lexer is validated.
    /// pointer address of the initial stream position.
    initial: *const u8,
    /// The start of the current subslice
    start: u32,
    /// The length of `initial` slice, and the end of the current subslice.
    end: u32,
    /// User-provided state
    pub state: S,
    lft: PhantomData<&'i ()>,
}
impl<S> fmt::Debug for Input<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bstr = self.input();
        bstr.fmt(f)
    }
}
impl<'i, S> Input<'i, S> {
    /// Wrap another Stream with span tracking
    #[inline(always)]
    pub fn new(input: &'i [u8], state: S) -> Self {
        let end = as_u32(input.len());
        let initial = input.as_ptr();
        Self { initial, start: 0, end, state, lft: PhantomData }
    }

    #[inline(always)]
    fn get_location(&self) -> usize {
        as_usize(self.start)
    }

    #[inline(always)]
    fn subslice(&self, subslice_len: usize) -> &'i [u8] {
        let len = as_usize(self.end - self.start);
        assert!(subslice_len <= len);

        let offset = as_usize(self.start);
        // SAFETY:
        // - `self.end` is always the size in bytes of `self.initial`
        // - `self.start` is always below `self.end`
        unsafe { slice::from_raw_parts::<'i, u8>(self.initial.add(offset), subslice_len) }
    }
    #[inline(always)]
    pub fn input_u8(&self) -> &'i [u8] {
        let len = as_usize(self.end - self.start);
        let offset = as_usize(self.start);
        // SAFETY:
        // - `self.end` is always the size in bytes of `self.initial`
        // - `self.start` is always below `self.end`
        unsafe { slice::from_raw_parts::<'i, u8>(self.initial.add(offset), len) }
    }
    fn input(&self) -> &'i BStr {
        BStr::new(self.input_u8())
    }
    pub fn is_empty(&self) -> bool {
        let mut input = self.input_u8();
        lex::next_token(&mut input).is_none()
    }

    pub const fn current_offset(&self) -> u32 {
        self.start
    }

    pub(crate) fn next_start(&self) -> u32 {
        let mut slice = self.input_u8();
        lex::consume_spaces(&mut slice);
        self.end - as_u32(slice.len())
    }
    #[inline(always)]
    pub fn starting_at(&self, start: u32) -> Self
    where
        S: Clone + fmt::Debug,
    {
        let mut ret = self.clone();
        ret.start = start;
        ret
    }
}

impl Offset<StateCheckpoint> for StateCheckpoint {
    fn offset_from(&self, initial: &StateCheckpoint) -> usize {
        let advanced = self;
        let frst = initial.start;
        let snd = advanced.start;
        debug_assert!(frst <= snd, "offset_to arg should be subslice of self");
        as_usize(snd - frst)
    }
}
impl<S> Offset<StateCheckpoint> for Input<'_, S> {
    #[inline(always)]
    fn offset_from(&self, initial: &StateCheckpoint) -> usize {
        let advanced = self;
        let frst = initial.start;
        let snd = advanced.start;
        debug_assert!(frst <= snd, "offset_to arg should be subslice of self");
        as_usize(snd - frst)
    }
}

impl<S> fmt::Display for Input<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.input().fmt(f)
    }
}

impl<'i, S: Clone + fmt::Debug> Stream for Input<'i, S> {
    type Token = Token<'i>;
    type Slice = &'i [u8];

    type IterOffsets = TokenIter<'i, S>;

    type Checkpoint = StateCheckpoint;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        let mut stream = self.clone();
        // SAFETY: invariants upholds valid initial[start..end]
        unsafe { stream.initial = stream.initial.add(as_usize(stream.start)) };
        stream.end -= self.start;
        stream.start = 0;
        TokenIter { stream }
    }
    #[inline(always)]
    fn eof_offset(&self) -> usize {
        as_usize(self.end - self.start)
    }

    #[inline(always)]
    fn next_token(&mut self) -> Option<Self::Token> {
        let mut slice = self.input_u8();
        let token = lex::next_token(&mut slice);
        self.start = self.end - as_u32(slice.len());
        token
    }

    #[inline(always)]
    fn offset_for<P>(&self, f: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.iter_offsets()
            .find_map(|(offset, t)| f(t).then_some(offset))
    }
    #[inline(always)]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        match self.iter_offsets().nth(tokens) {
            Some((offset, _)) => Ok(offset),
            None => Err(Needed::Unknown),
        }
    }
    // NOTE: The winnow API requires us to use known-good offsets, so we never
    // split within a token.
    // TODO(PERF): consider something more sensible to avoid whitespaces in
    // `.recognize()` See design_docs/whitespace_after_lex.md
    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        let mut slice = self.subslice(offset);
        // We skip whitespaces for the presented slice.
        lex::consume_spaces(&mut slice);
        self.start += as_u32(offset);
        slice
    }

    #[inline(always)]
    fn checkpoint(&self) -> StateCheckpoint {
        StateCheckpoint { end: self.end, start: self.start }
    }
    #[inline(always)]
    fn reset(&mut self, checkpoint: Self::Checkpoint) {
        self.start = checkpoint.start;
        self.end = checkpoint.end;
    }

    #[inline(always)]
    fn raw(&self) -> &dyn fmt::Debug {
        self
    }
}

impl<'i, S> StreamIsPartial for Input<'i, S> {
    type PartialState = ();

    fn complete(&mut self) {}
    fn restore_partial(&mut self, (): ()) {}
    fn is_partial_supported() -> bool {
        false
    }
}
impl<'i, S> Location for Input<'i, S> {
    #[inline(always)]
    fn location(&self) -> usize {
        self.get_location()
    }
}

/// Provide the `.checkpoint` method on `Parser`, to get the current input checkpoint.
pub(super) struct WithCheckpoint<'i, F, S, O, E>
where
    F: Parser<Input<'i, S>, O, E>,
{
    parser: F,
    p: PhantomData<(S, O, E, &'i ())>,
}
impl<'i, F, S, O, E> Parser<Input<'i, S>, (O, StateCheckpoint), E>
    for WithCheckpoint<'i, F, S, O, E>
where
    F: Parser<Input<'i, S>, O, E>,
    S: fmt::Debug + Clone,
{
    #[inline]
    fn parse_next(&mut self, input: &mut Input<'i, S>) -> winnow::PResult<(O, StateCheckpoint), E> {
        let checkpoint = input.checkpoint();
        self.parser
            .parse_next(input)
            .map(move |output| (output, checkpoint))
    }
}

/// Provide the `.spanned` method on `Parser`, this has more useful span info.
pub(super) struct WithSpan<'i, F, S, O, E>
where
    F: Parser<Input<'i, S>, O, E>,
{
    parser: F,
    p: PhantomData<(S, O, E, &'i ())>,
}

impl<'i, F, S, O, E> Parser<Input<'i, S>, (O, (u32, u32)), E> for WithSpan<'i, F, S, O, E>
where
    F: Parser<Input<'i, S>, O, E>,
    S: fmt::Debug + Clone,
{
    #[inline]
    fn parse_next(&mut self, input: &mut Input<'i, S>) -> winnow::PResult<(O, (u32, u32)), E> {
        let start = input.next_start();
        self.parser.parse_next(input).map(move |output| {
            let end = input.start;
            (output, (start, end))
        })
    }
}
pub(super) trait ParserExt<'i, S, O, E>: Parser<Input<'i, S>, O, E> {
    fn spanned(self) -> WithSpan<'i, Self, S, O, E>
    where
        Self: Sized;
    fn checkpoint(self) -> WithCheckpoint<'i, Self, S, O, E>
    where
        Self: Sized;
}
impl<'i, S, O, E, T: Parser<Input<'i, S>, O, E>> ParserExt<'i, S, O, E> for T {
    fn spanned(self) -> WithSpan<'i, Self, S, O, E>
    where
        Self: Sized,
    {
        WithSpan { parser: self, p: PhantomData }
    }
    fn checkpoint(self) -> WithCheckpoint<'i, Self, S, O, E>
    where
        Self: Sized,
    {
        WithCheckpoint { parser: self, p: PhantomData }
    }
}
