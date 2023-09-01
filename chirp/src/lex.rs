#![allow(clippy::match_bool, clippy::inline_always)]
use std::{fmt, ops};

use winnow::{
    error::Needed,
    stream::{Compare, Location, Offset, Stream, StreamIsPartial},
    BStr,
};

impl<'i, S> Stateful<'i, S> {
    /// Wrap another Stream with span tracking
    pub fn new(input: &'i BStr, state: S) -> Self {
        let initial = input.as_ptr() as usize;
        Self { initial, input, state }
    }

    fn get_location(&self) -> usize {
        let input_offset = self.input.as_ptr() as usize;
        input_offset - self.initial
    }
}

impl<'i, S> Offset<<&'i BStr as Stream>::Checkpoint> for Stateful<'_, S> {
    #[inline(always)]
    fn offset_from(&self, start: &<&'i BStr as Stream>::Checkpoint) -> usize {
        self.input.offset_from(start)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Stateful<'i, S> {
    initial: usize,
    /// Inner input being wrapped in state
    pub input: &'i BStr,
    /// User-provided state
    pub state: S,
}

impl<'i, S> AsRef<&'i BStr> for Stateful<'i, S> {
    #[inline(always)]
    fn as_ref(&self) -> &&'i BStr {
        &self.input
    }
}

impl<'i, S> ops::Deref for Stateful<'i, S> {
    type Target = &'i BStr;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<S> fmt::Display for Stateful<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.input.fmt(f)
    }
}

impl<'i, S: Clone + std::fmt::Debug> Stream for Stateful<'i, S> {
    // TODO(perf): replace this with ident(){}[]"' and trim whitespaces
    // (including comments) from stream.
    type Token = <&'i BStr as Stream>::Token;
    type Slice = <&'i BStr as Stream>::Slice;

    type IterOffsets = <&'i BStr as Stream>::IterOffsets;

    type Checkpoint = <&'i BStr as Stream>::Checkpoint;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        self.input.iter_offsets()
    }
    #[inline(always)]
    fn eof_offset(&self) -> usize {
        self.input.eof_offset()
    }

    #[inline(always)]
    fn next_token(&mut self) -> Option<Self::Token> {
        self.input.next_token()
    }

    // TODO(perf): Compare using SWAR
    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.input.offset_for(predicate)
    }
    #[inline(always)]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        self.input.offset_at(tokens)
    }
    #[inline(always)]
    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        self.input.next_slice(offset)
    }

    #[inline(always)]
    fn checkpoint(&self) -> Self::Checkpoint {
        self.input.checkpoint()
    }
    #[inline(always)]
    fn reset(&mut self, checkpoint: Self::Checkpoint) {
        self.input.reset(checkpoint);
    }

    #[inline(always)]
    fn raw(&self) -> &dyn std::fmt::Debug {
        &self.input
    }
}

impl<'i, S> StreamIsPartial for Stateful<'i, S> {
    type PartialState = ();

    fn complete(&mut self) {}
    fn restore_partial(&mut self, (): ()) {}
    fn is_partial_supported() -> bool {
        false
    }
}
impl<'i, S> Location for Stateful<'i, S> {
    #[inline(always)]
    fn location(&self) -> usize {
        self.get_location()
    }
}
impl<'i, S, const N: usize> Compare<&'_ [u8; N]> for Stateful<'i, S> {
    fn compare(&self, t: &'_ [u8; N]) -> winnow::stream::CompareResult {
        match self.input.get(..N) == Some(t) {
            true => winnow::stream::CompareResult::Ok,
            false => winnow::stream::CompareResult::Error,
        }
    }
    fn compare_no_case(&self, t: &'_ [u8; N]) -> winnow::stream::CompareResult {
        let identical = t
            .iter()
            .map(u8::to_ascii_lowercase)
            .zip(self.input.iter().map(u8::to_ascii_lowercase))
            .fold(true, |acc, (x, y)| acc & (x == y));
        match identical {
            true => winnow::stream::CompareResult::Ok,
            false => winnow::stream::CompareResult::Error,
        }
    }
}
