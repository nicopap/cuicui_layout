use std::slice;

use crate::lex::Token;

// TODO(perf): split the within-method lexer (,) and outside lexer
const RECOGNIZED_SYMBOL_COUNT: usize = 15;
const RECOGNIZED_SYMBOLS: [u8; RECOGNIZED_SYMBOL_COUNT] = *b"=(){}[],\"'/ \n\t\r";

enum Recognized {
    Equal = 0,
    Lparen = 1,
    Rparen = 2,
    Lcurly = 3,
    Rcurly = 4,
    Lbracket = 5,
    Rbracket = 6,
    Comma = 7,
    Quote = 8,
    Apostrophe = 9,
    Slash = 10,
    Space = 11,
    Ident = 15,
}

#[inline(always)]
fn recognize(x: u8) -> Recognized {
    let mut cmp_table = [x; RECOGNIZED_SYMBOL_COUNT];

    for i in 0..RECOGNIZED_SYMBOL_COUNT {
        cmp_table[i] ^= RECOGNIZED_SYMBOLS[i]
    }
    match cmp_table.iter().position(|x| *x == 0) {
        Some(0) => Recognized::Equal,
        Some(1) => Recognized::Lparen,
        Some(2) => Recognized::Rparen,
        Some(3) => Recognized::Lcurly,
        Some(4) => Recognized::Rcurly,
        Some(5) => Recognized::Lbracket,
        Some(6) => Recognized::Rbracket,
        Some(7) => Recognized::Comma,
        Some(8) => Recognized::Quote,
        Some(9) => Recognized::Apostrophe,
        Some(10) => Recognized::Slash,
        Some(11..=14) => Recognized::Space,
        None => Recognized::Ident,
        Some(_) => unreachable!("==OPTIMIZEDOUT== match position in const size array"),
    }
}

struct Ident<'i>(&'i [u8]);

impl<'i> Ident<'i> {
    fn next(&mut self, input: &mut &[u8]) -> &'i [u8] {
        use Recognized::{
            Apostrophe, Comma, Equal, Lbracket, Lcurly, Lparen, Quote, Rbracket, Rcurly, Rparen,
            Space,
        };

        //     let _t = trace_span!("Ident::next", name = "Ident::next").entered();
        // This should have been a recursive call, but noooo, no tail call elimination in rust.
        loop {
            let Some(&next) = input.first() else {
                return self.0;
            };
            match recognize(next) {
                Equal | Lparen | Rparen | Lcurly | Rcurly | Quote | Apostrophe | Space
                | Lbracket | Rbracket | Comma => return self.0,

                // TODO(feat): comment on double slash
                Recognized::Slash | Recognized::Ident => {
                    let len = self.0.len();
                    let ptr = <[u8]>::as_ptr(self.0);
                    // SAFETY: the `.0` slice is a subslice of `input`,
                    // we just checked that input has an additional slot.
                    self.0 = unsafe { slice::from_raw_parts(ptr, len + 1) };
                    *input = &input[1..];
                }
            }
        }
    }
}

pub fn next_token<'i>(input: &mut &'i [u8]) -> Option<Token<'i>> {
    after_space(input)
}
// TODO(perf): have a "rough" pass checking for whitespace only on a slice
// first, then finding the next non-space index.
// TODO(perf): We can get rid of `&[u8]` (two words) in Quoted and Ident
// by storing a "offset until begin of token" only, then re-constructing
// the slice in `next_token` from that offset.
// A potentially simpler alternative is to store the `input` as a `*const u8`
// before running Quote::next or Ident::next
fn after_space<'i>(input: &mut &'i [u8]) -> Option<Token<'i>> {
    // let _t = trace_span!("after_space", name = "after_space").entered();

    loop {
        if input.is_empty() {
            return None;
        }
        let (first, remaining) = input.split_at(1);
        *input = remaining;
        match recognize(first[0]) {
            Recognized::Equal => return Some(Token::Equal),
            Recognized::Lparen => return Some(Token::Lparen),
            Recognized::Rparen => return Some(Token::Rparen),
            Recognized::Lcurly => return Some(Token::Lcurly),
            Recognized::Rcurly => return Some(Token::Rcurly),
            Recognized::Lbracket => return Some(Token::Lbracket),
            Recognized::Rbracket => return Some(Token::Rbracket),
            Recognized::Comma => return Some(Token::Comma),
            Recognized::Quote => return Quoted::<b'"'>(first).next(input),
            Recognized::Apostrophe => return Quoted::<b'\''>(first).next(input),
            Recognized::Slash => return next_maybe_comment(input),
            Recognized::Ident => return Some(Token::Ident(Ident(first).next(input))),
            Recognized::Space => {}
        }
    }
}

struct Quoted<'i, const Q: u8>(&'i [u8]);
impl<'i, const Q: u8> Quoted<'i, Q> {
    fn advance(&mut self, input: &mut &'i [u8]) -> Option<u8> {
        let first = *input.first()?;
        let len = self.0.len();
        let ptr = <[u8]>::as_ptr(self.0);
        // SAFETY: the `.0` slice is a subslice of `input`,
        // we just checked that input has an additional slot.
        self.0 = unsafe { slice::from_raw_parts(ptr, len + 1) };
        *input = &input[1..];
        Some(first)
    }
    // TODO(perf): have a "rough" pass checking for Q or backslash. If not, we
    // don't care about the N next u8.
    fn next(&mut self, input: &mut &'i [u8]) -> Option<Token<'i>> {
        //     let _t = trace_span!("Quoted::next", name = "Quoted::next").entered();
        let mut esc = false;
        loop {
            let Some(last) = self.advance(input) else {
                return (!esc).then_some(Token::String(self.0));
            };
            match last {
                b'\\' => esc = !esc,
                q if q == Q && esc => esc = !esc,
                q if q == Q => return Some(Token::String(self.0)),
                _ => {}
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndComment {
    LineEnding(usize),
    None,
}
// fn end_comment<const LANES: usize>(xs: [u8; LANES]) -> EndComment {
//     let mid = usize::MAX / 2;
//     let pos = xs.iter().fold(usize::MAX, |acc, x| match x {
//         b'\n' if acc > mid => usize::MAX - acc,
//         _ if acc > mid => acc - 1,
//         _ => acc,
//     });
//     if pos < mid { EndComment::LineEnding(pos) } else { EndComment::None }
// }

#[derive(Clone, Copy)]
struct Swar8(u64);
impl Swar8 {
    fn load_from(slice: &[u8]) -> Self {
        let len = slice.len().min(8);
        let mut acc = [0; 8];
        acc[..len].copy_from_slice(&slice[..len]);
        Swar8(u64::from_le_bytes(acc))
    }
    fn position<const B: u8>(self) -> EndComment {
        let mask = u64::from_le_bytes([B; 8]);
        let mut masked = self.0 ^ mask;
        let mut pos = 0;
        let mut encountered = false;
        for _ in 0..8 {
            encountered |= masked & 0xff == 0;
            if !encountered {
                pos += 1;
            }
            masked >>= 8;
        }
        if encountered {
            EndComment::LineEnding(pos)
        } else {
            EndComment::None
        }
    }
}

fn next_comment<'i>(input: &mut &'i [u8]) -> Option<Token<'i>> {
    // let _t = trace_span!("next_comment", name = "next_comment").entered();
    loop {
        match Swar8::load_from(input).position::<b'\n'>() {
            EndComment::LineEnding(offset) => {
                // SAFETY: `Swar8::position` is guarenteed to return either None
                // or a value between 0 max len of input.
                *input = unsafe { input.get_unchecked(offset..) };
                return after_space(input);
            }
            EndComment::None => *input = input.get(8..)?,
        }
    }
}
fn next_maybe_comment<'i>(input: &mut &'i [u8]) -> Option<Token<'i>> {
    // let _t = trace_span!("next_maybe_comment", name = "next_maybe_comment").entered();
    let (first, remaining) = input.split_first()?;
    *input = remaining;
    match *first {
        b'/' => next_comment(input),
        _ => after_space(input),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correct_position() {
        let some = |x| EndComment::LineEnding(x);
        let none = EndComment::None;
        let pos = |x| Swar8::load_from(x).position::<b'x'>();
        assert_eq!(pos(b"x   "), some(0));
        assert_eq!(pos(b" x  "), some(1));
        assert_eq!(pos(b"___"), none);
        assert_eq!(pos(b""), none);
        assert_eq!(pos(b"x"), some(0));
        assert_eq!(pos(b"________x"), none);
        assert_eq!(pos(b"0123456x"), some(7));
        assert_eq!(pos(b"0123456x8"), some(7));
        assert_eq!(pos(b"01234567x"), none);
        assert_eq!(pos(b"0123456x8910"), some(7));
        assert_eq!(pos(b"   x                  "), some(3));
        assert_eq!(pos(b"x x x x               "), some(0));
        assert_eq!(pos(b" xx_"), some(1));
        assert_eq!(pos(b"__x____x__________"), some(2));
    }
}
