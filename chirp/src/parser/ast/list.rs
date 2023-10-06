//! Iterators over a list of AST nodes.
//!
//! The AST is a single large buffer. Some [`Node`]s have a variable size (typically
//! nodes that contain a list of other nodes).
//!
//! [`List`] is an iterator over a list of such variable-size `Node`.
//!
//! Other nodes, such as `Import` or `Argument` have a known fixed size, [`SimpleNode`],
//! This allows `List<T>` to provide more methods, those methods are size-based.

use std::marker::PhantomData;

use super::node::{Argument, Code, IdentOffset, Import, StType};
use super::node::{ChirpFile, Fn, Method, Spawn, Statement, Template};
use super::{as_usize, header::Block};

macro_rules! dummy {
    ($($_:tt)*) => {};
}
macro_rules! impl_node {
    ($($ty:ident : $($header:ident)? |$it:ident| $compute:expr),* $(,)?)=>{
        $(impl<'a> Node<'a> for $ty<'a> {
            #[inline] fn len(self) -> u32 { let $it = self; $compute }
            #[inline] fn first(blocks: &'a [Block]) -> Self { Self::new(blocks) }
            $(dummy!{$header} #[inline] fn len_from_header(self) -> Option<u32> {Some(self.len())})?
        })*}
}
macro_rules! impl_simple_node {
    ($($ty:ident: $size:literal),* $(,)?)=>{
        $(impl SimpleNode for $ty<'_> { const SIZE: u32 = $size; })*
        impl_node! {$( $ty : header |_it| Self::SIZE ,)*}
    }
}

// TODO(clean): Figure out a way to not expose Block in this trait, so that I
// don't need to make Block pub(in crate::parser)
pub(in crate::parser) trait Node<'a>: Copy {
    /// The length.
    ///
    /// `Some` if it is possible to compute it based only on reading the header,
    /// `None` otherwise. This should always return the same as [`Node::len`].
    fn len_from_header(self) -> Option<u32> {
        None
    }
    /// The number of blocks this node requires.
    fn len(self) -> u32;
    fn first(blocks: &'a [Block]) -> Self;
}
pub(in crate::parser) trait SimpleNode {
    const SIZE: u32;
}
impl_node! {
    ChirpFile: |it| it.root_statement_offset() + it.root_statement().len(),
    Fn:        |it| Self::HEADER_SIZE + it.parameter_len() + it.body().len(),
    Method:    header |it| Self::HEADER_SIZE + it.argument_len(),
    Template:  header |it| Self::HEADER_SIZE + it.argument_len() + it.methods_len() + it.children_len(),
    Spawn:     header |it| Self::HEADER_SIZE + it.methods_len() + it.children_len(),
    Statement: header |it| match it.typed() {
        StType::Spawn(s) => s.len(),
        StType::Template(s) => s.len(),
        StType::Code(_) => Code::SIZE,
    },
}
impl_simple_node! {Import: 2, Argument: 2, Code: 1}

#[rustfmt::skip] impl SimpleNode for IdentOffset { const SIZE: u32 = 1; }
#[rustfmt::skip] impl<'a> Node<'a> for IdentOffset {
    fn len(self) -> u32 { Self::SIZE }
    fn first(blocks: &'a [Block]) -> Self { Self { start: blocks[0].0 } }
}

#[derive(Clone, Copy)]
pub(in crate::parser) struct List<'a, T: Node<'a>>(&'a [Block], PhantomData<T>);

#[derive(Clone)]
pub(in crate::parser) struct ListIter<'a, T: Node<'a>>(List<'a, T>);

impl<'a, T: Node<'a>> List<'a, T> {
    pub fn empty() -> Self {
        Self(&[], PhantomData)
    }
    #[inline]
    pub(super) fn new(blocks: &'a [Block]) -> Self {
        Self(blocks, PhantomData)
    }
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }
    #[inline]
    pub fn first(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        Some(T::first(self.0))
    }
    #[inline]
    pub fn iter(self) -> ListIter<'a, T> {
        ListIter(self)
    }
}
impl<'a, T: Node<'a>> Iterator for ListIter<'a, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.0.first()?;
        self.0 .0 = &self.0 .0[as_usize(ret.len())..];
        Some(ret)
    }
}
impl<'a, T: SimpleNode + Node<'a>> List<'a, T> {
    pub fn count(&self) -> usize {
        self.0.len() / as_usize(T::SIZE)
    }
    pub fn get(&self, index: usize) -> Option<T> {
        let start = index * as_usize(T::SIZE);
        let end = (index + 1) * as_usize(T::SIZE);
        let blocks = self.0.get(start..end)?;
        Some(T::first(blocks))
    }
    pub fn last(&self) -> Option<T> {
        let index = self.count().checked_sub(1)?;
        self.get(index)
    }
}
