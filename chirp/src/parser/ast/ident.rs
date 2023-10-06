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
use winnow::stream::Stream;

use super::as_u32;
use super::node::{Argument, IdentOffset};
use crate::parser::stream::{Input, Token};

macro_rules! more_unsafe_unreachable {
    ($it:expr) => {
        if cfg!(feature = "more_unsafe") {
            // SAFETY: We can only create OptNameOffset in crate::parser,
            // and we only create OptNameOffset that points to proper
            // identifier/string tokens
            unsafe { std::hint::unreachable_unchecked() }
        } else {
            let Self { start } = $it;
            let type_name = std::any::type_name::<Self>();
            unreachable!(
                "When parsing the chirp file, we generated an invalid {type_name} \
                    at {start}, This is a major cuicui bug, please open an issue:\n\n\
                    https://github.com/nicopap/cuicui_layout/issues/new"
            )
        }
    };
}

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
        let next_token = input.starting_at(self.start).next_token();
        if let Some(Token::Ident(ident) | Token::String(ident)) = next_token {
            let end = self.start + as_u32(ident.len());
            Some((ident, (self.start, end)))
        } else {
            more_unsafe_unreachable!(self);
        }
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
        if let Some(Token::Ident(ident)) = input.starting_at(self.start).next_token() {
            (ident, (self.start, self.start + as_u32(ident.len())))
        } else {
            more_unsafe_unreachable!(self);
        }
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

#[rustfmt::skip] impl From<u32> for IdentOffset {
    fn from(start: u32) -> Self { Self { start } }
}
#[rustfmt::skip] impl From<Option<IdentOffset>> for OptIdentOffset {
    fn from(value: Option<IdentOffset>) -> Self { Self { start: value.map_or(u32::MAX, |i| i.start) } }
}
#[rustfmt::skip] impl From<Option<u32>> for OptNameOffset {
    fn from(value: Option<u32>) -> Self { Self { start: value.unwrap_or(u32::MAX) } }
}
