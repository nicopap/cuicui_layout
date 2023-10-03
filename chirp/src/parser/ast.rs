//! The AST parsed by the chirp grammar.

use winnow::stream::Stream;

use super::{Input, Token};

fn as_u32(usize: usize) -> u32 {
    usize.try_into().unwrap()
}

#[derive(Clone, Copy, Debug)]
pub struct Argument {
    pub(super) start: u32,
    pub(super) end: u32,
}
impl Argument {
    pub(super) fn new(start: u32, len: usize) -> Self {
        Self { start, end: start + as_u32(len) }
    }
    pub fn read<'i>(self, input: &Input<'i>) -> &'i [u8] {
        let (start, end) = (self.start as usize, self.end as usize);
        &input.input_u8()[start..end]
    }
}
pub struct Method {
    pub name: IdentOffset,
    pub arguments: Vec<Argument>,
}
impl Method {
    pub(super) fn new((name, arguments): (IdentOffset, Vec<Argument>)) -> Self {
        Self { name, arguments }
    }
}

pub struct Template {
    pub name: IdentOffset,
    pub arguments: Vec<Argument>,
    pub methods: Vec<Method>,
    pub children: Vec<Node>,
}
impl Template {
    pub(super) fn new(
        (name, arguments, methods, children): (IdentOffset, Vec<Argument>, Vec<Method>, Vec<Node>),
    ) -> Self {
        Self { name, arguments, methods, children }
    }
}
pub struct Statement {
    pub name: OptNameOffset,
    pub methods: Vec<Method>,
    pub children: Vec<Node>,
}
impl Statement {
    pub(super) fn both((name, methods, children): (OptNameOffset, Vec<Method>, Vec<Node>)) -> Self {
        Self { name, methods, children }
    }
    pub(super) fn children((name, children): (OptNameOffset, Vec<Node>)) -> Self {
        Self { name, methods: Vec::new(), children }
    }
}

pub struct Import {
    pub name: IdentOffset,
    pub alias: OptIdentOffset,
}
impl Import {
    pub(super) fn new((name, alias): (IdentOffset, Option<IdentOffset>)) -> Self {
        Self { name, alias: alias.into() }
    }
}
pub struct Function {
    pub name: IdentOffset,
    pub arguments: Vec<IdentOffset>,
    pub body: Node,
}
impl Function {
    pub(super) fn new((name, arguments, body): (IdentOffset, Vec<IdentOffset>, Node)) -> Self {
        Self { name, arguments, body }
    }
}
pub enum Node {
    Template(Template),
    Statement(Statement),
    Code(IdentOffset),
}
pub struct ChirpFile {
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
    pub root: Node,
}
impl ChirpFile {
    pub(super) fn new((imports, functions, root): (Vec<Import>, Vec<Function>, Node)) -> Self {
        Self { imports, functions, root }
    }
}

/// Offset in an [`Input`] of an entity name, may be an identifier or string literal,
/// and **is optional**.
#[derive(Clone, Copy, Debug)]
pub struct OptNameOffset {
    start: u32,
}
impl OptNameOffset {
    pub const fn new(offset: u32) -> Self {
        Self { start: offset }
    }
    pub const fn empty() -> Self {
        Self { start: u32::MAX }
    }
    pub fn get_with_span<'i>(self, input: &Input<'i>) -> Option<(&'i [u8], (u32, u32))> {
        if self.start == u32::MAX {
            return None;
        }
        let next_token = input.starting_at(self.start).next_token();
        if let Some(Token::Ident(ident) | Token::String(ident)) = next_token {
            let end = self.start + as_u32(ident.len());
            Some((ident, (self.start, end)))
        } else {
            unreachable!()
        }
    }
}
impl From<Option<u32>> for OptNameOffset {
    fn from(value: Option<u32>) -> Self {
        value.map_or(Self::empty(), Self::new)
    }
}

#[derive(Clone, Debug, Copy)]
pub struct IdentOffset {
    start: u32,
}
impl IdentOffset {
    pub const fn new(start: u32) -> Self {
        Self { start }
    }
    pub fn read_spanned<'i>(self, input: &Input<'i>) -> (&'i [u8], (u32, u32)) {
        if let Some(Token::Ident(ident)) = input.starting_at(self.start).next_token() {
            (ident, (self.start, self.start + as_u32(ident.len())))
        } else {
            unreachable!()
        }
    }
    pub fn read<'i>(self, input: &Input<'i>) -> &'i [u8] {
        self.read_spanned(input).0
    }
}
impl From<u32> for IdentOffset {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}
#[derive(Clone, Debug, Copy)]
pub struct OptIdentOffset {
    start: u32,
}
impl OptIdentOffset {
    pub const fn new(start: u32) -> Self {
        Self { start }
    }
    pub const fn empty() -> Self {
        Self { start: u32::MAX }
    }
    pub fn read_spanned<'i>(self, input: &Input<'i>) -> Option<(&'i [u8], (u32, u32))> {
        if self.start == u32::MAX {
            return None;
        }
        Some(IdentOffset::new(self.start).read_spanned(input))
    }
}
impl From<Option<IdentOffset>> for OptIdentOffset {
    fn from(value: Option<IdentOffset>) -> Self {
        value.map(|v| v.start).into()
    }
}
impl From<Option<u32>> for OptIdentOffset {
    fn from(value: Option<u32>) -> Self {
        value.map_or(Self::empty(), Self::new)
    }
}
