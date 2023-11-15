//! The chirp grammar, defined using winnow parsers.
//!
//! The current implementation is not done with a particular look for performance,
//! just functionality and readability.
//!
//! ```text
//! TokenTree
//!    = 'ident'
//!    | '(' (TokenTree)* ')'
//!    | '[' (TokenTree)* ']'
//!    | '{' (TokenTree)* '}'
//!    | StringLit
//!
//! Method = 'ident' ('(' (TokenTree)* ')')?
//!
//! Statement
//!    = 'code'      '(' 'ident' ')'
//!    | 'Entity'    StatementTail
//!    | 'ident' '!' '(' (TokenTree (',' TokenTree)*)? ')' (StatementTail)?
//!    | 'ident'     StatementTail
//!    | StringLit   StatementTail
//!
//! StatementTail
//!    = '(' (Method)* ')' ('{' (Statement)* '}')?
//!    | '{' (Statement)* '}'
//!
//! Use = 'use' Path '/' '{' ImportName( ImportName )* }
//! Fn = ('pub')? 'fn' 'ident' '(' ('ident' (',' 'ident')*)? ')' '{' Statement '}'
//! ChirpFile = (Use)* (Fn)* Statement
//! ```
#![allow(clippy::inline_always)]
// allow: The generated code is fine, it's in line with how winnow does things
// internally.
use winnow::PResult;

use super::ast::AstBuilder;
use super::stream::Input;
use super::Error;

pub use chirp_file::chirp_file;

mod chirp_file;
mod generic;
#[cfg(test)]
mod tests;
mod tokens;

type BlockResult = PResult<u32, Error>;

trait AddNodes {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult;
}
