//! The AST parsed by the chirp grammar.
//!
//! # Architecture
//!
//! So this isn't a casual AST. It is designed as a single contiguous buffer.
//!
//! Basically, each node is a variable size slice of this contiguous buffer.
//!
//! The type of the buffer is [`Block`](header::Block) where `Block` is just a `u32`. Each node
//! has:
//!
//! 1. A fixed size "header", that takes a fixed number of blocks and has a fixed layout.
//! 2. Optionally a variable size list of sub-items. Such as arguments to methods
//!    or methods and children to statements.
//!
//! The "header" contains information proper to the node (such as the `name` of
//! a statement, and the size of the sub-item lists of AST nodes within this node.
//!
//! A "sub-item" list is a variable length homogenous list (think of a `Vec<T>`) of a given node.
//!
//! For example, `Template` may have arguments, methods, or children statements.
//! `ChirpFile` has multiple imports, fn declarations, and a single root statement.
//!
//! We have two kind of sub-item lists:
//!
//! - Fixed size item lists, when the item has a fixed size (such as `Argument` or `Import`)
//! - Dynamic size item lists, when the item has a dynamic size (such as `Methods` or `Statements`)
//!
//! The size of the sub-item list is expressed as the item count when the item
//! has a fixed size. While it is expressed as the number of blocks occupied by
//! the list when the item has a dynamic size.
//!
//! The distinction is carefully kept in this module:
//!
//! - Methods returning a block length are suffixed with **`_len`**.
//! - Methods returning item counts are suffixed with **`_count`**.
//!
//! This is important to express it as the number of blocks for dynamic size lists
//! because then we can crate a slice with the complete list within it and safely
//! iterate through dynamic-size stuff, which is usually completely impossible in rust.
//!
//! ## Node layout
//!
//! Currently the layout (header order and sub-item list order) is defined in
//! `design_docs/ast.md`.
//!
//! - In [`header`], we define helpers to define headers
//! - In [`node`], we define all the AST nodes, using [`header`] helpers inside macros
//! - In [`build`], we define a way to build AST nodes on top of a buffer.
//!
//! ## Creating the AST
//!
//! This setup has a major flaw: It doesn't play nice with the `winnow` API.
//! Most "many" combinators in `winnow` work by creating an accumulator, pushing
//! items to it and returning that iterator.
//!
//! The single buffer architecture requires keeping a single buffer and accumulate
//! new blocks into it. The accumulator cannot be created from nothing.
//!
//! Thankfully, `winnow` is very flexible, and allows imperative style. We define
//! a trait, `AddBlocks`[^1], which is basically `winnow::Parser` with an additional
//! `&mut Vec<Block>` argument. And instead of returning the AST node, it pushs it to
//! the buffer. It's a lot more code than the original `winnow` parser,
//! but it works very similarly.
//!
//! [^1]: Not to be confused with `AdBlock`, in any case, prefer ublock origin
#![allow(clippy::missing_const_for_fn)]

use core::slice;

pub(super) use build::{AstBuilder, WriteHeader};
pub use ident::*;
pub(super) use list::List;
pub(super) use node::{Argument, Fn, IdentOffset, Spawn, StKind, StType, Statement, Template};
pub(super) use node::{ArgumentHeader, ChirpFileHeader, FnHeader, ImportHeader, MethodHeader};
pub(super) use node::{CodeHeader, SpawnHeader, TemplateHeader};

pub(super) type Methods<'a> = List<'a, node::Method<'a>>;
pub(super) type Statements<'a> = List<'a, node::Statement<'a>>;
pub(super) type Arguments<'a> = List<'a, node::Argument<'a>>;
pub(super) type IdentOffsets<'a> = List<'a, node::IdentOffset>;

mod build;
mod header;
mod ident;
mod list;
mod node;

#[derive(Clone, Copy)]
pub struct FnIndex(*const header::Block);

#[derive(Clone, Copy)]
pub struct RefAst<'a>(&'a [header::Block]);
impl<'a> RefAst<'a> {
    pub(super) fn get_fn(&self, fn_index: FnIndex) -> Fn<'a> {
        let self_ptr = self.0.as_ptr();
        assert!(self_ptr <= fn_index.0 && fn_index.0 <= self_ptr.wrapping_add(self.0.len()));
        // SAFETY:
        // - Within our AST
        // - We UNSOUNDLY assume this means it comes from the same allocation
        // - non null: within &T means non-null
        let slice = unsafe { slice::from_raw_parts(self_ptr, 0) };
        unsafe { Fn::new_unchecked(slice) }
    }
    pub(super) fn chirp_file(self) -> node::ChirpFile<'a> {
        node::ChirpFile::new(self.0)
    }
}

pub struct Ast(Vec<header::Block>);
impl Ast {
    pub fn as_ref(&self) -> RefAst {
        RefAst(&self.0)
    }
}

#[allow(clippy::inline_always)]
#[inline(always)]
fn as_u32(usize: usize) -> u32 {
    usize.try_into().unwrap()
}

#[allow(clippy::inline_always)]
#[inline(always)]
fn as_usize(u32: u32) -> usize {
    u32.try_into().unwrap()
}
