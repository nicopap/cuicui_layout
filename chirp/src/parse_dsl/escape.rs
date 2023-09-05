use std::borrow::Cow;

type Swar = u32;
const LANES: usize = 8;
const SWAR_BYTES: usize = (Swar::BITS / 8) as usize;

#[allow(clippy::verbose_bit_mask)] // what a weird lint
fn contains_swar(mut xored: Swar) -> bool {
    // For a position, nothing easier: pos = 0; pos += ret; ret &= xored & 0xff != 0;
    let mut ret = false;
    for _ in 0..SWAR_BYTES {
        ret |= xored & 0xff == 0;
        xored >>= 8;
    }
    ret
}

fn fast_contains<const WHAT: u8>(check: &[u8]) -> bool {
    let mask = Swar::from_le_bytes([WHAT; SWAR_BYTES]);

    // SAFETY: [u8; SWAR_BYTES] is a valid Swar
    let (head, body, tail) = unsafe { check.align_to::<[Swar; LANES]>() };

    head.iter().chain(tail).any(|c| *c == WHAT)
        || body
            .iter()
            .map(|vs| {
                vs.iter()
                    .fold(false, |acc, &v| acc | contains_swar(v ^ mask))
            })
            .any(Into::into)
}
/// Escape backslashes in `to_escape`, returning the escaped string.
///
/// If `to_escape` doesn't contain any backslash, this returns `to_escape`
/// as-is as a [`Cow::Borrowed`].
#[must_use]
pub fn escape_literal(to_escape: &[u8]) -> Cow<[u8]> {
    #[cold]
    fn owned(bytes: &[u8]) -> Cow<[u8]> {
        let mut ret = bytes.to_vec();
        let mut prev_bs = false;
        ret.retain_mut(|c| {
            match c {
                b'n' if prev_bs => *c = b'\n',
                b't' if prev_bs => *c = b'\t',
                b'r' if prev_bs => *c = b'\r',
                _ => {}
            };
            let is_bs = c == &b'\\';
            let keep = !is_bs | prev_bs;
            prev_bs = !keep & is_bs;
            keep
        });
        Cow::Owned(ret)
    }
    if fast_contains::<b'\\'>(to_escape) {
        owned(to_escape)
    } else {
        Cow::Borrowed(to_escape)
    }
}
