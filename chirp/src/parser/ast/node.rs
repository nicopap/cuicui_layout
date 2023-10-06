//! Define all AST nodes with the layout of their header.
use std::fmt;

use super::build::{self, WriteHeader};
use super::header::{Block, HeaderFieldAccess, Idx, Lower, Upper, Usplit};
#[cfg(not(feature = "more_unsafe"))]
use super::list::Node;
use super::list::{List, SimpleNode};
use super::{as_u32, as_usize, OptIdentOffset, OptNameOffset, RefAst};

#[cfg(feature = "more_unsafe")]
#[derive(Clone, Copy)]
struct Header<'a, const N: usize>(&'a [Block; N]);

#[cfg(not(feature = "more_unsafe"))]
#[derive(Clone, Copy)]
struct Header<'a, const N: usize>(&'a [Block]);

#[derive(Clone, Copy)]
pub struct FnIndex<'a>(pub(super) Fn<'a>);

impl<'a, const N: usize> Header<'a, N> {
    fn raw_block(self) -> &'a [Block] {
        self.0
    }
    /// Return slice of `len` offset **from the end of the header** by `offset`.
    ///
    /// # Safety
    /// The offset+len should be within the same slice.
    #[track_caller]
    unsafe fn offset(self, offset: u32, len: u32) -> &'a [Block] {
        #[cfg(feature = "more_unsafe")]
        {
            // SAFETY: This is extremely unsafe. It is however sound because:
            // - Header is only constructed in `new` methods on nodes in `impl_header!`
            // - Header::new is only called within this module
            // - Which itself is only used in crate::parser::interpret
            // - crate::parser::interpret can only walk ASTs built by crate::parser::grammar
            // - crate::parser::grammar can only build VALID ASTs because it can
            //   only build ASTs through methods in crate::parser::ast::build
            let ptr = unsafe { self.0.as_ptr().add(N + as_usize(offset)) };
            unsafe { std::slice::from_raw_parts(ptr, as_usize(len)) }
        }
        #[cfg(not(feature = "more_unsafe"))]
        {
            assert!(
                as_usize(len + offset) + N <= self.0.len(),
                "Trying to get subslice {}..{} of Node of size {}",
                as_usize(offset) + N,
                as_usize(offset + len) + N,
                self.0.len(),
            );
            let start = N + as_usize(offset);
            &self.0[start..start + as_usize(len)]
        }
    }
}

/// Define an AST node.
///
/// The trailing list of fields between `{}` represents "fields" as in defined
/// in `design_docs/ast.md`, it is not exactly a rust field, but a value derived
/// from the array `[Block; $size]` located in `Header`.
macro_rules! impl_header {
    ( $node_name:ident, $header_name:ident, $size:literal,
      {$($vis:vis $method_name:ident : $method_accessor:ty => $method_accessor_field_type:ty),* $(,)?}
    ) => {

        #[derive(Clone, Copy)]
        pub(in crate::parser) struct $node_name<'a>(Header<'a, $size>);
        impl<'a> $node_name<'a> {
            #[allow(unused)]
            pub(super) const HEADER_SIZE: u32 = $size;

            /// Create a `Self` assuming that `header` is a `Self` header
            /// with the correct number of blocks.
            ///
            /// # Safety
            /// - `header` is a reference to a slice element, and there are
            ///   enough blocks following `header` to make a proper `Self` node.
            #[inline(always)]
            pub(super) unsafe fn new_unchecked(header: &'a [Block]) -> Self {
                #[cfg(feature = "more_unsafe")]
                let header = {
                    // SAFETY:
                    // - the `header` reference is the start of a slice of size at least $size
                    //   (upheld by function's invariants)
                    // - it is non-null as it is a reference
                    let header = header.as_ptr().cast::<[Block; $size]>();
                    unsafe { header.as_ref().unwrap_unchecked() }
                };
                Self(Header(header))
            }
            /// Create a `Self` from the first element in the slice.
            #[track_caller]
            #[inline(always)]
            pub(super) fn new(header: &'a [Block]) -> Self {
                #[cfg(not(feature = "more_unsafe"))]
                assert!(
                    $size <= header.len(),
                    "{} HEADER size doesn't fit in provided slice: header([Block; {}]) > slice(&[Block](len {}))",
                    stringify!($node_name), $size, header.len(),
                );
                // SAFETY: the `header` reference is the start of a slice of size at least $size
                let ret = unsafe { Self::new_unchecked(header) };
                #[cfg(not(feature = "more_unsafe"))]
                assert!(
                    as_usize(ret.len_from_header().unwrap_or(0)) <= header.len(),
                    "{} size doesn't fit in provided slice: node({}) > slice(&[Block](len {}))",
                    stringify!($node_name), ret.len(), header.len(),
                );
                ret
            }

            #[allow(unused)]
            pub(in crate::parser) fn block_index(self, ast: RefAst<'a>) -> isize {
                unsafe { self.0 .0.as_ptr().offset_from(ast.0.as_ptr()) }
            }
            $(
            #[track_caller]
            #[inline(always)]
            $vis fn $method_name(self) -> $method_accessor_field_type {
                let slice = self.0 .0;
                #[cfg(not(feature = "more_unsafe"))]
                let slice = {
                    assert!($size <= slice.len());
                    unsafe { slice.as_ptr().cast::<[Block; $size]>().as_ref().unwrap_unchecked() }
                };
                <$method_accessor>::get(slice)
            }
            )*
        }
        impl fmt::Debug for $node_name<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($node_name))
                    $(.field(stringify!($method_name), &self.$method_name()))*
                    .finish()
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct $header_name { $(
            pub $method_name : $method_accessor_field_type,
        )* }
        impl WriteHeader for $header_name {
            const SIZE: u32 = $size;
            type Buffer<'a> = build::Buffer<'a, $size>;

            #[inline(always)]
            fn write_header(self, buffer: build::Buffer<$size>) {
                $( <$method_accessor>::write_to(self.$method_name, buffer.0); )*
            }
        }
    };
}

#[derive(Clone, Copy)]
pub(in crate::parser) enum StType<'a> {
    Spawn(Spawn<'a>),
    Template(Template<'a>),
    Code(Code<'a>),
}
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum StKind {
    Spawn,
    Template,
    Code,
}

impl_header![ChirpFile, ChirpFileHeader, 2, {
    import_count: Idx<0> => u32,
    pub(super) root_statement_offset: Idx<1> => u32,
}];
impl<'a> ChirpFile<'a> {
    pub(super) fn import_len(self) -> u32 {
        self.import_count() * Import::SIZE
    }
    pub fn imports(self) -> List<'a, Import<'a>> {
        List::new(unsafe { self.0.offset(0, self.import_len()) })
    }
    pub fn fn_declrs(self) -> List<'a, Fn<'a>> {
        let len = self.root_statement_offset() - self.import_len() - Self::HEADER_SIZE;
        List::new(unsafe { self.0.offset(self.import_len(), len) })
    }
    pub fn root_statement(self) -> Statement<'a> {
        let offset = self.root_statement_offset() - Self::HEADER_SIZE;
        let statement_len = as_u32(self.0 .0.len()) - self.root_statement_offset();
        let len = if cfg!(feature = "more_unsafe") { 0 } else { statement_len };
        let statement_slice = unsafe { self.0.offset(offset, len) };
        unsafe { Statement::new_unchecked(statement_slice) }
    }
}
type FnHeader0 = (Idx<0>, Usplit<u32, IdentOffset, 26>);
impl_header![Fn, FnHeader, 1, {
    parameter_count: (FnHeader0, Upper) => u32,
    pub name: (FnHeader0, Lower) => IdentOffset,
}];
impl<'a> Fn<'a> {
    pub(super) fn parameter_len(self) -> u32 {
        self.parameter_count() * <IdentOffset as SimpleNode>::SIZE
    }
    pub fn parameters(self) -> List<'a, IdentOffset> {
        List::new(unsafe { self.0.offset(0, self.parameter_len()) })
    }
    #[inline]
    pub fn body(self) -> Statement<'a> {
        let fn_len = as_u32(self.0 .0.len()) - self.parameter_len() - FnHeader::SIZE;
        let len = if cfg!(feature = "more_unsafe") { 0 } else { fn_len };
        unsafe { Statement::new_unchecked(self.0.offset(self.parameter_len(), len)) }
    }
    pub fn index(self) -> FnIndex<'a> {
        FnIndex(self)
    }
}
impl_header![Method, MethodHeader, 1, {
    argument_count: (FnHeader0, Upper) => u32,
    pub name: (FnHeader0, Lower) =>IdentOffset,
}];
impl<'a> Method<'a> {
    pub(super) fn argument_len(self) -> u32 {
        self.argument_count() * Argument::SIZE
    }
    #[inline]
    pub fn arguments(self) -> List<'a, Argument<'a>> {
        List::new(unsafe { self.0.offset(0, self.argument_len()) })
    }
}
type StHeader0 = (Idx<0>, Usplit<StKind, (), 28>);
impl_header![Statement, StatementHeader, 1, {
    discriminant: (StHeader0, Upper) => StKind,
}];
impl<'a> Statement<'a> {
    #[inline]
    pub fn typed(self) -> StType<'a> {
        use StKind::Template as KTemplate;

        match self.discriminant() {
            // SAFETY: currently I'm unsure this is sound
            StKind::Spawn => unsafe { StType::Spawn(Spawn::new_unchecked(self.0.raw_block())) },
            KTemplate => unsafe { StType::Template(Template::new_unchecked(self.0.raw_block())) },
            StKind::Code => unsafe { StType::Code(Code::new_unchecked(self.0.raw_block())) },
        }
    }
}
type SHeader0 = (Idx<0>, Usplit<(), OptNameOffset, 28>);
impl_header![Spawn, SpawnHeader, 3, {
    pub name: (SHeader0, Lower) => OptNameOffset,
    pub(super) methods_len: Idx<1> => u32,
    pub(super) children_len: Idx<2> => u32,
}];
impl<'a> Spawn<'a> {
    #[inline]
    pub fn methods(self) -> List<'a, Method<'a>> {
        List::new(unsafe { self.0.offset(0, self.methods_len()) })
    }
    #[inline]
    pub fn children(self) -> List<'a, Statement<'a>> {
        List::new(unsafe { self.0.offset(self.methods_len(), self.children_len()) })
    }
}
type THeader0 = (Idx<0>, Usplit<(), IdentOffset, 28>);
type THeader1 = (Idx<1>, Usplit<u32, u32, 26>);
impl_header![Template, TemplateHeader, 3, {
    pub name: (THeader0, Lower) => IdentOffset,
    argument_count: (THeader1, Upper) => u32,
    pub(super) methods_len: (THeader1, Lower) => u32,
    pub(super) children_len: Idx<2> => u32,
}];
impl<'a> Template<'a> {
    pub(super) fn argument_len(self) -> u32 {
        self.argument_count() * Argument::SIZE
    }
    #[inline]
    pub fn arguments(self) -> List<'a, Argument<'a>> {
        List::new(unsafe { self.0.offset(0, self.argument_len()) })
    }
    #[inline]
    pub fn methods(self) -> List<'a, Method<'a>> {
        List::new(unsafe { self.0.offset(self.argument_len(), self.methods_len()) })
    }
    #[inline]
    pub fn children(self) -> List<'a, Statement<'a>> {
        let offset = self.argument_len() + self.methods_len();
        List::new(unsafe { self.0.offset(offset, self.children_len()) })
    }
}
impl_header![Code, CodeHeader, 1, { pub name: (THeader0, Lower) => IdentOffset }];

type IdxT<T, const I: usize> = ((Idx<I>, Usplit<T, (), 0>), Upper);
impl_header![Import, ImportHeader, 2, {
    pub name: IdxT<IdentOffset, 0> => IdentOffset,
    pub alias: IdxT<OptIdentOffset, 1> => OptIdentOffset,
}];
impl_header![Argument, ArgumentHeader, 2, { pub start: Idx<0> => u32, pub end: Idx<1> => u32 }];

#[derive(Clone, Copy, Debug)]
pub struct IdentOffset {
    pub start: u32,
}
impl WriteHeader for IdentOffset {
    const SIZE: u32 = 1;
    type Buffer<'a> = build::Buffer<'a, 1>;

    fn write_header(self, buffer: build::Buffer<1>) {
        buffer.0[0].0 = self.start;
    }
}
impl<T, const N: usize> WriteHeader for (StKind, T)
where
    T: for<'a> WriteHeader<Buffer<'a> = build::Buffer<'a, N>>,
{
    const SIZE: u32 = T::SIZE;
    type Buffer<'a> = T::Buffer<'a>;

    fn write_header(self, builder: Self::Buffer<'_>) {
        let header = StatementHeader { discriminant: self.0 };
        let st_subslice = (&mut builder.0[..1]).try_into().unwrap();
        header.write_header(build::Buffer(st_subslice));
        self.1.write_header(builder);
    }
}
