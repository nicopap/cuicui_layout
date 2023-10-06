//! Define [`HeaderFieldAccess`], a way to formalize the definition of a node "header".
//!
//! The node [`Header`] has a known [`Block`] count and layout.
//!
//! Sometimes the fields are "packed" so that two bits of data are encoded within
//! the same `Block`. In this case [`Usplit`] encode access to the packed fields.
//! The field types must implement [`FromMask32`].
//!
//! `FromMask32` is `From<u32>` for most types. `OptNameOffset` has however special
//! handling since it encodes the `None` variant as `u32::MAX` and with masking,
//! some of the bits in the `u32` won't be set.
#![allow(clippy::wrong_self_convention)]
use std::marker::PhantomData;

use from_mask32::FromMask32;

#[rustfmt::skip]
mod from_mask32 {
    pub trait FromMask32: 'static {
        fn from_mask32<const SPLIT: usize>(bits: u32) -> Self;
        fn as_u32(self) -> u32;
    }
    impl FromMask32 for u32 {
        fn from_mask32<const SPLIT: usize>(bits: u32) -> Self { bits }
        fn as_u32(self) -> u32 { self }
    }
    impl FromMask32 for () {
        fn from_mask32<const SPLIT: usize>(_: u32) {}
        fn as_u32(self) -> u32 {
            panic!(
                "==OPTIMIZEDOUT== Tried to call ()::as_u32, this should never happen. \
                This is a cuicui_chirp bug, please open an issue!\n\
                https://github.com/nicopap/cuicui_layout/issues/new",
            )
        }
    }
    impl FromMask32 for super::super::OptNameOffset {
        fn from_mask32<const SPLIT: usize>(bits: u32) -> Self {
            let start = if bits == (1 << SPLIT) - 1 { u32::MAX } else { bits };
            Self { start }
        }
        fn as_u32(self) -> u32 { self.start }
    }
    impl FromMask32 for super::super::OptIdentOffset {
        fn from_mask32<const SPLIT: usize>(bits: u32) -> Self {
            let start = if bits == (1 << SPLIT) - 1 { u32::MAX } else { bits };
            Self { start }
        }
        fn as_u32(self) -> u32 { self.start }
    }
    impl FromMask32 for super::super::node::IdentOffset {
        fn from_mask32<const SPLIT: usize>(start: u32) -> Self { Self { start } }
        fn as_u32(self) -> u32 { self.start }
    }
    impl FromMask32 for super::super::node::StKind {
        fn from_mask32<const SPLIT: usize>(bits: u32) -> Self {
            match bits {
                b if b == Self::Spawn as u32 => Self::Spawn,
                b if b == Self::Template as u32 => Self::Template,
                b if b == Self::Code as u32 => Self::Code,
                #[cfg(feature = "more_unsafe")]
                _ => Self::Spawn,
                #[cfg(not(feature = "more_unsafe"))]
                _ => unreachable!(
                    "The Discriminant field of a AST Statement node was invalid. \
                    This is a cuicui_chirp bug, please open an issue!\n\
                    https://github.com/nicopap/cuicui_layout/issues/new",
                ),
            }
        }
        fn as_u32(self) -> u32 { self as u32 }
    }
}

pub(super) trait HeaderFieldAccess {
    type FieldType: 'static;

    fn get<const N: usize>(header: &[Block; N]) -> Self::FieldType;
    fn write_to(field: Self::FieldType, blocks: &mut [Block]);
}

pub(super) struct Upper;
pub(super) struct Lower;
pub(super) struct Idx<const I: usize>;

impl<const I: usize> HeaderFieldAccess for Idx<I> {
    type FieldType = u32;

    #[inline]
    fn get<const N: usize>(header: &[Block; N]) -> Self::FieldType {
        header[I].0
    }
    #[inline]
    fn write_to(field: u32, blocks: &mut [Block]) {
        blocks[I] = Block(field);
    }
}

impl<U: FromMask32, L: FromMask32, const B: usize, const I: usize> HeaderFieldAccess
    for ((Idx<I>, Usplit<U, L, B>), Lower)
{
    type FieldType = L;

    #[inline]
    fn get<const N: usize>(header: &[Block; N]) -> Self::FieldType {
        let bits = header[I].0;
        Usplit::<U, L, B>::new(bits).lower()
    }
    #[inline]
    fn write_to(field: L, blocks: &mut [Block]) {
        let mask = (1u32 << B).wrapping_sub(1);
        blocks[I].0 &= !mask;
        blocks[I].0 |= field.as_u32() & mask;
    }
}

impl<U: FromMask32, L: FromMask32, const B: usize, const I: usize> HeaderFieldAccess
    for ((Idx<I>, Usplit<U, L, B>), Upper)
{
    type FieldType = U;

    #[inline]
    fn get<const N: usize>(header: &[Block; N]) -> Self::FieldType {
        let bits = header[I].0;
        Usplit::<U, L, B>::new(bits).upper()
    }
    #[inline]
    fn write_to(field: U, blocks: &mut [Block]) {
        let mask = (1u32 << B).wrapping_sub(1);
        blocks[I].0 &= mask;
        blocks[I].0 |= (field.as_u32() & mask) << B;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub(in crate::parser) struct Block(pub(super) u32);

#[derive(Clone, Copy)]
pub(super) struct Usplit<Uppr, Lwr, const SPLIT: usize>(u32, PhantomData<(Uppr, Lwr)>);

impl<Uppr: FromMask32, Lwr: FromMask32, const SPLIT: usize> Usplit<Uppr, Lwr, SPLIT> {
    fn new(bits: u32) -> Self {
        Self(bits, PhantomData)
    }
    fn upper(self) -> Uppr {
        let bits = self.0 >> SPLIT;
        Uppr::from_mask32::<SPLIT>(bits)
    }
    fn lower(self) -> Lwr {
        let bits = self.0 & ((1 << SPLIT) - 1);
        Lwr::from_mask32::<SPLIT>(bits)
    }
}
