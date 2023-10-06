//! Definition of "offset" tokens.
//!
//! Those tokens represent a string of text. They do not store the actual
//! string value however, they store the offset in [`Input`] of the relevant text.
//!
//! More specifically:
//! - [`Argument`] stores the start and end of a token tree passed to a method or template
//!   as argument. Unlike the other token types, it doesn't require any parsing, at the
//!   cost of occupying twice the memory in the AST.
//! - [`IdentOffset`] stores the start of a single identifier. To reconstruct the
//!   whole string it parses a single token. The lexer is fairly efficient, so
//!   we can do this without the fear of being very slow.
//! - `Opt***`: Optional variants of `***` where value of `u32::MAX` represents
//!   the absence of value.
//! - [`OptNameOffset`] is similar to `IdentOffset`, but the parsed token may either
//!   be a string or identifier (it is also optional).
use super::as_u32;
use super::node::{Argument, IdentOffset};
use crate::parser::stream::Input;

/// Offset in an [`Input`] of an entity name, may be an identifier or string literal,
/// and **is optional**.
#[derive(Clone, Copy, Debug)]
pub struct OptNameOffset {
    pub(super) start: u32,
}
#[derive(Clone, Debug, Copy)]
pub struct OptIdentOffset {
    pub(super) start: u32,
}
impl OptNameOffset {
    pub fn get_with_span<'i>(self, input: &Input<'i>) -> Option<(&'i [u8], (u32, u32))> {
        if self.start == u32::MAX {
            return None;
        }
        // SAFETY:
        // - We can only create OptNameOffset in crate::parser
        // - We only create OptNameOffset in crate::parser::grammar
        // - And they are the starting offset of either an identifier or string, always
        let ident = unsafe { input.starting_at(self.start).next_statement_name() };
        let end = self.start + as_u32(ident.len());
        Some((ident, (self.start, end)))
    }
}
impl OptIdentOffset {
    pub fn read_spanned<'i>(self, input: &Input<'i>) -> Option<(&'i [u8], (u32, u32))> {
        if self.start == u32::MAX {
            return None;
        }
        Some(IdentOffset { start: self.start }.read_spanned(input))
    }
}
impl IdentOffset {
    pub fn read_spanned<'i>(self, input: &Input<'i>) -> (&'i [u8], (u32, u32)) {
        // SAFETY:
        // - We can only create IdentOffset in crate::parser
        // - We only create IdentOffset in crate::parser::grammar
        // - And they are the starting offset of identifiers, always
        let ident = unsafe { input.starting_at(self.start).next_ident() };
        (ident, (self.start, self.start + as_u32(ident.len())))
    }
    pub fn read<'i>(self, input: &Input<'i>) -> &'i [u8] {
        self.read_spanned(input).0
    }
}
impl Argument<'_> {
    pub fn read<'i>(self, input: &Input<'i>) -> &'i [u8] {
        let (start, end) = (self.start() as usize, self.end() as usize);
        &input.input_u8()[start..end]
    }
}

#[rustfmt::skip] impl From<u32> for IdentOffset { fn from(start: u32) -> Self { Self { start } } }
#[rustfmt::skip] impl From<Option<IdentOffset>> for OptIdentOffset {
    fn from(value: Option<IdentOffset>) -> Self { Self { start: value.map_or(u32::MAX, |i| i.start) } }
}
#[rustfmt::skip] impl From<Option<u32>> for OptNameOffset {
    fn from(value: Option<u32>) -> Self { Self { start: value.unwrap_or(u32::MAX) } }
}
