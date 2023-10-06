#![allow(clippy::verbose_bit_mask)]

use std::slice;

use super::stream::Token;

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
// allow: I manually reviewed the generated code and it is better with the inline.
#[allow(clippy::inline_always)]
#[inline(always)]
fn recognize(x: u8) -> Recognized {
    let mut cmp_table = [x; RECOGNIZED_SYMBOL_COUNT];

    for i in 0..RECOGNIZED_SYMBOL_COUNT {
        cmp_table[i] ^= RECOGNIZED_SYMBOLS[i];
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

pub(super) unsafe fn next_statement_name<'i>(input: &mut &'i [u8]) -> &'i [u8] {
    let (first, remaining) = input.split_at(1);
    *input = remaining;
    match recognize(first[0]) {
        Recognized::Quote => {
            let mut q = Quoted::<b'"'>(first);
            q.next(input);
            q.0
        }
        Recognized::Apostrophe => {
            let mut q = Quoted::<b'\''>(first);
            q.next(input);
            q.0
        }
        Recognized::Ident => Ident(first).next(input),
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
}
pub(super) unsafe fn next_ident<'i>(input: &mut &'i [u8]) -> &'i [u8] {
    let (first, remaining) = input.split_at(1);
    *input = remaining;
    Ident(first).next(input)
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
        let mut esc = false;
        loop {
            let Some(last) = self.advance(input) else {
                // TODO(bug): when the string is not complete,
                // The file is invalid, we should somehow handle that.
                return (!esc).then_some(Token::String(self.0));
            };
            match last {
                b'\\' => esc = !esc,
                q if q == Q && esc => esc = !esc,
                q if q == Q => return Some(Token::String(self.0)),
                _ => esc = false,
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Swar8(u64);
impl Swar8 {
    fn load_from(slice: &[u8]) -> Self {
        let len = slice.len().min(8);
        let mut acc = [0; 8];
        acc[..len].copy_from_slice(&slice[..len]);
        Swar8(u64::from_le_bytes(acc))
    }
    fn position<const B: u8>(self) -> Option<usize> {
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
            Some(pos)
        } else {
            None
        }
    }
}

fn next_comment<'i>(input: &mut &'i [u8]) -> Option<Token<'i>> {
    loop {
        match Swar8::load_from(input).position::<b'\n'>() {
            Some(offset) => {
                // SAFETY: `Swar8::position` is guarenteed to return either None
                // or a value between 0 max len of input.
                *input = unsafe { input.get_unchecked(offset + 1..) };
                return after_space(input);
            }
            None => *input = input.get(8..)?,
        }
    }
}
fn next_maybe_comment<'i>(input: &mut &'i [u8]) -> Option<Token<'i>> {
    let (first, remaining) = input.split_first()?;
    *input = remaining;
    if *first == b'/' {
        next_comment(input)
    } else {
        // TODO(BUG): single slashes are completely ignored.
        after_space(input)
    }
}

const END_SPACE_COUNT: usize = 4;
const END_SPACE: [u8; END_SPACE_COUNT] = *b"/ \n\t";

enum EndSpace {
    Slash,
    Token,
    None,
}
fn recognize_space(x: u8) -> EndSpace {
    let mut cmp_table = [x; END_SPACE_COUNT];

    for i in 0..END_SPACE_COUNT {
        cmp_table[i] ^= END_SPACE[i];
    }
    match cmp_table.iter().position(|x| *x == 0) {
        Some(0) => EndSpace::Slash,
        Some(1..=3) => EndSpace::None,
        None => EndSpace::Token,
        Some(_) => unreachable!("==OPTIMIZEDOUT== match position in const size array"),
    }
}
pub(super) fn consume_spaces(input: &mut &[u8]) {
    loop {
        let Some((&x, remaining)) = input.split_first() else {
            return;
        };
        match recognize_space(x) {
            EndSpace::Slash => {
                *input = remaining;
                consume_maybe_comment(input);
                return;
            }
            EndSpace::Token => {
                return;
            }
            EndSpace::None => {
                *input = remaining;
            }
        }
    }
}
fn consume_maybe_comment(input: &mut &[u8]) {
    let Some((&first, remaining)) = input.split_first() else {
        return;
    };
    if first == b'/' {
        *input = remaining;
        consume_comment(input);
    }
}
fn consume_comment(input: &mut &[u8]) {
    loop {
        match Swar8::load_from(input).position::<b'\n'>() {
            Some(offset) => {
                // SAFETY: `Swar8::position` is guarenteed to return either None
                // or a value between 0 max len of input.
                *input = unsafe { input.get_unchecked(offset + 1..) };
                consume_spaces(input);
                return;
            }
            None => match input.get(8..) {
                Some(remaining) => *input = remaining,
                None => return,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_position() {
        let pos = |x| Swar8::load_from(x).position::<b'x'>();
        assert_eq!(pos(b"x   "), Some(0));
        assert_eq!(pos(b" x  "), Some(1));
        assert_eq!(pos(b"___"), None);
        assert_eq!(pos(b""), None);
        assert_eq!(pos(b"x"), Some(0));
        assert_eq!(pos(b"________x"), None);
        assert_eq!(pos(b"0123456x"), Some(7));
        assert_eq!(pos(b"0123456x8"), Some(7));
        assert_eq!(pos(b"01234567x"), None);
        assert_eq!(pos(b"0123456x8910"), Some(7));
        assert_eq!(pos(b"   x                  "), Some(3));
        assert_eq!(pos(b"x x x x               "), Some(0));
        assert_eq!(pos(b" xx_"), Some(1));
        assert_eq!(pos(b"__x____x__________"), Some(2));
    }

    #[test]
    fn valid_string() {
        #[track_caller]
        fn full_valid(input: &[u8]) {
            valid(input, input, b"");
        }
        #[track_caller]
        fn valid(input: &[u8], expected: &[u8], remaining: &[u8]) {
            let (first, mut input) = input.split_at(1);
            let token = Quoted::<b'"'>(first).next(&mut input);
            assert_eq!(Some(Token::String(expected)), token);
            assert_eq!(remaining, input);
        }
        full_valid(br#""hello""#);
        full_valid(br#""hello world""#);
        full_valid(br#""hello\"world""#);
        full_valid(br#""hello\\world""#);

        full_valid(br#""hello\\\\\"world""#); // hello\\"world
        full_valid(br#""\\hello world\\""#); // \hello world\
        full_valid(br#""  hello world\"""#); // |  hello world"|
        full_valid(br#""\'hello world\"""#); // 'hello world"
    }
}
