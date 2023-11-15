use winnow::combinator::{alt, preceded};
use winnow::error::ErrMode::{Backtrack, Cut};
use winnow::token::any;
use winnow::Parser;

use super::generic::{Delimited, Many, SepList, Terminated};
use super::tokens::{ident, many_tts};
use super::{AddNodes, BlockResult};
use crate::parser::ast::{self, CodeHeader, FnHeader, SpawnHeader, StKind, TemplateHeader};
use crate::parser::ast::{ArgumentHeader, IdentOffset, ImportHeader, ImportItemHeader};
use crate::parser::ast::{Ast, AstBuilder, ChirpFileHeader, MethodHeader, WriteHeader};
use crate::parser::stream::{tokens as t, Input, Token};
use crate::parser::Error;

type Opt<T> = Option<T>;
type Sep<T> = SepList<T>;
type Paren<T> = Delimited<T, t::Lparen, t::Rparen>;
type Curly<T> = Delimited<T, t::Lcurly, t::Rcurly>;

#[rustfmt::skip]
macro_rules! Tokens {
    ($inner:ty, ')') => { Terminated<$inner, t::Rparen> };
    ($inner:ty, ']') => { Terminated<$inner, t::Rbracket> };
    ($inner:ty, '}') => { Terminated<$inner, t::Rcurly> };
}
#[rustfmt::skip]
macro_rules! tokens {
    ('(' $inner:expr, ')') => { winnow::combinator::delimited(t::Lparen, $inner, t::Rparen) };
}
#[rustfmt::skip]
macro_rules! token {
    ($first:tt $(| $many:tt)*) => { token!(@ $first) $(| token!(@ $many))* };
    (@ "ident")  => { Token::Ident(_) };
    (@ "string") => { Token::String(_) };
    (@ '(') => { Token::Lparen };
    (@ ')') => { Token::Rparen };
    (@ '{') => { Token::Lcurly };
    (@ '}') => { Token::Rcurly };
    (@ '[') => { Token::Lbracket };
    (@ ']') => { Token::Rbracket };
    (@ ',') => { Token::Comma };
    (@ '=') => { Token::Equal };
}

struct ImportItem;
impl AddNodes for ImportItem {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        let import = |input: &mut Input| {
            alt((
                tokens!('('(ident, preceded(t::As, ident).map(Some)), ')'),
                ident.map(|i| (i, None)),
            ))
            .parse_next(input)
        };
        let (name, alias) = import(input)?;
        builder.write_header(ImportItemHeader { name, alias: alias.into() });
        Ok(ImportItemHeader::SIZE)
    }
}

struct Import;
impl AddNodes for Import {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        use Token::{Ident, String as TStr};

        t::Use.parse_next(input)?;
        let header = builder.reserve_header();

        let start = input.next_start();
        match any.parse_next(input)? {
            TStr(_) | Ident(_) => {}
            bad_token => return Err(Cut(Error::FileName(Some(bad_token).into()))),
        };
        let item_len = Paren::<Many<ImportItem>>::add_node(input, builder)?;
        let item_count = item_len / ast::ImportItemHeader::SIZE;
        builder.write(header, ImportHeader { item_count, name: start.into() });
        Ok(ImportHeader::SIZE + item_len)
    }
}

struct Argument;
impl AddNodes for Argument {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        let (start, end) = many_tts::<true>(input)?;
        builder.write_header(ArgumentHeader { start, end });
        Ok(ArgumentHeader::SIZE)
    }
}
impl AddNodes for IdentOffset {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        let ident = ident(input)?;
        builder.write_header(ident);
        Ok(Self::SIZE)
    }
}

struct Fn;
impl AddNodes for Fn {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        t::Fn.parse_next(input)?;
        let header = builder.reserve_header();

        let name = ident(input)?;
        let parameter_len = Paren::<Sep<IdentOffset>>::add_node(input, builder)?;
        let body_len = Curly::<Sep<St>>::add_node(input, builder)?;

        let parameter_count = parameter_len / IdentOffset::SIZE;
        builder.write(header, FnHeader { parameter_count, name });
        Ok(FnHeader::SIZE + parameter_len + body_len)
    }
}

struct St;
impl AddNodes for St {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        use Token::{Ident, String as TStr};

        let start = input.next_start();
        match any.parse_next(input)? {
            TStr(name) | Ident(name) if name.ends_with(b"!") => {
                add_template(start.into(), input, builder)
            }
            TStr(name) | Ident(name) if name == b"code" => {
                let name = tokens!('(' ident, ')').parse_next(input)?;
                builder.write_header((StKind::Code, CodeHeader { name }));
                Ok(CodeHeader::SIZE)
            }
            TStr(name) | Ident(name) => {
                let not_empty = ![b"Entity", &b"spawn"[..]].contains(&name);
                add_spawn(not_empty.then_some(start), input, builder)
            }
            token!('}') => Err(Backtrack(Error::Unexpected)),
            bad_token => Err(Cut(Error::StartStatement(Some(bad_token).into()))),
        }
    }
}
fn add_template(name: IdentOffset, input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
    let header = builder.reserve_header();

    let argument_len = Paren::<Sep<Argument>>::add_node(input, builder)?;
    let methods_len = Opt::<Paren<Many<Method>>>::add_node(input, builder)?;
    let children_len = Opt::<Curly<Many<St>>>::add_node(input, builder)?;

    let argument_count = argument_len / ArgumentHeader::SIZE;
    let writer = (
        StKind::Template,
        TemplateHeader { name, argument_count, methods_len, children_len },
    );
    builder.write(header, writer);
    Ok(TemplateHeader::SIZE + argument_len + methods_len + children_len)
}

fn add_spawn(name: Option<u32>, input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
    let header = builder.reserve_header();

    let (methods_len, children_len) = match any.parse_next(input)? {
        token!('(') => {
            let methods = <Tokens![Many<Method>, ')']>::add_node(input, builder)?;
            let children = Opt::<Curly<Many<St>>>::add_node(input, builder)?;
            (methods, children)
        }
        token!('{') => (0, <Tokens![Many<St>, '}']>::add_node(input, builder)?),
        bad_token => return Err(Cut(Error::StatementDelimiter(Some(bad_token).into()))),
    };
    let name = name.into();
    let writer = (
        StKind::Spawn,
        SpawnHeader { name, methods_len, children_len },
    );
    builder.write(header, writer);
    Ok(SpawnHeader::SIZE + methods_len + children_len)
}

struct Method;
impl AddNodes for Method {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        // Note: header created after name so that we don't create it when closing method zone
        let name = ident.parse_next(input)?;
        let header = builder.reserve_header();

        let argument_len = Opt::<Paren<SepList<Argument>>>::add_node(input, builder)?;

        let argument_count = argument_len / ArgumentHeader::SIZE;
        builder.write(header, MethodHeader { argument_count, name });
        Ok(MethodHeader::SIZE + argument_len)
    }
}

struct ChirpFile;
impl AddNodes for ChirpFile {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        let header = builder.reserve_header();

        let import_len = Many::<Import>::add_node(input, builder)?;
        let fn_len = Many::<Fn>::add_node(input, builder)?;
        let root_statement_len = St::add_node(input, builder)?;

        let root_statement_offset = ChirpFileHeader::SIZE + import_len + fn_len;
        builder.write(
            header,
            ChirpFileHeader { import_len, root_statement_offset },
        );
        Ok(root_statement_offset + root_statement_len)
    }
}

pub fn chirp_file(mut input: Input) -> Result<Ast, (Error, (u32, u32))> {
    let mut ast_builder = AstBuilder::new();
    let result = ChirpFile::add_node(&mut input, &mut ast_builder);
    let offset = input.current_offset();

    match result {
        Ok(_) if input.is_empty() => Ok(ast_builder.build()),
        Ok(_) => Err((Error::TrailingText, (offset, offset))),
        Err(Cut(err) | Backtrack(err)) => Err((err, (offset, offset))),
        _ => unreachable!(),
    }
}
