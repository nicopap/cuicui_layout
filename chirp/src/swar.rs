//! Unused moudle/dead code
//!
//! Theoretically, this could improve parsing performances eightfold.
type Swar = u32;
const LANES: usize = 8;
const SWAR_BYTES: usize = (Swar::BITS / 8) as usize;

// TODO(perf): if we can pack this in a u64, we could make the cmp a 3 op
const END_COUNT: usize = 10;
const END_IDENT: [u8; END_COUNT] = *b"=(){}\"' \t\n";

enum EndToken {
    Equal = 0,
    Lparen = 1,
    Rparen = 2,
    Lbracket = 3,
    Rbracket = 4,
    Quote = 5,
    Space = 7,
    None = 10,
}

fn check_end(x: u8) -> EndToken {
    use EndToken::{Equal, Lbracket, Lparen, Quote, Rbracket, Rparen, Space};
    let cmp_table = [x; END_COUNT];
    let xor_table = cmp_table ^ END_IDENT;
    match xor_table.find(|x| x == 0) {
        Some(0) => Some(Equal),
        Some(1) => Some(Lparen),
        Some(2) => Some(Rparen),
        Some(3) => Some(Lbracket),
        Some(4) => Some(Rbracket),
        Some(5 | 6) => Some(Quote),
        Some(7 | 8 | 9) => Some(Space),
        None => EndToken::None,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Token<'i> {
    Rparen,
    Lparen,
    Rbracket,
    Lbracket,
    Ident(&'i BStr),
    String(&'i BStr),
}
