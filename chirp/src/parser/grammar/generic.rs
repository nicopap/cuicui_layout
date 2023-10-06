use std::marker::PhantomData;

use winnow::stream::Stream;
use winnow::{error::ErrMode, Parser};

use super::{AddNodes, BlockResult, Error};
use crate::parser::ast::AstBuilder;
use crate::parser::stream::{tokens, Input, Token};

pub(super) struct Delimited<T, P1, P2>(PhantomData<(T, P1, P2)>);
impl<T: AddNodes, P1, P2> AddNodes for Delimited<T, P1, P2>
where
    for<'i> P1: Default + Parser<Input<'i>, Token<'i>, Error>,
    for<'i> P2: Default + Parser<Input<'i>, Token<'i>, Error>,
{
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        P1::default().parse_next(input)?;
        let builder = T::add_node(input, builder)?;
        P2::default().parse_next(input)?;
        Ok(builder)
    }
}

pub(super) struct Terminated<T, P>(PhantomData<(T, P)>);
impl<T: AddNodes, P> AddNodes for Terminated<T, P>
where
    for<'i> P: Default + Parser<Input<'i>, Token<'i>, Error>,
{
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        let builder = T::add_node(input, builder)?;
        P::default().parse_next(input)?;
        Ok(builder)
    }
}

pub(super) struct SepList<T>(PhantomData<T>);
impl<T: AddNodes> AddNodes for SepList<T> {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        let start = input.checkpoint();
        let mut count = match T::add_node(input, builder) {
            Err(ErrMode::Backtrack(_)) => {
                input.reset(start);
                return Ok(0);
            }
            Err(e) => return Err(e),
            Ok(count) => count,
        };
        loop {
            let start = input.checkpoint();
            let len = input.eof_offset();
            match tokens::Comma.parse_next(input) {
                Err(ErrMode::Backtrack(_)) => {
                    input.reset(start);
                    return Ok(count);
                }
                Err(e) => return Err(e),
                Ok(_) if input.eof_offset() == len => panic!("Infinite loop in parser"),
                Ok(_) => {}
            }
            match T::add_node(input, builder) {
                Err(ErrMode::Backtrack(_)) => {
                    input.reset(start);
                    return Ok(count);
                }
                Err(e) => return Err(e),
                Ok(incr) => count += incr,
            }
        }
    }
}

pub(super) struct Many<T>(PhantomData<T>);
impl<T: AddNodes> AddNodes for Many<T> {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        let mut count = 0;
        loop {
            let start = input.checkpoint();
            let len = input.eof_offset();
            match T::add_node(input, builder) {
                Err(ErrMode::Backtrack(_)) => {
                    input.reset(start);
                    return Ok(count);
                }
                Err(e) => return Err(e),
                Ok(_) if input.eof_offset() == len => panic!("Infinite loop in parser"),
                Ok(incr) => count += incr,
            }
        }
    }
}

impl<T: AddNodes> AddNodes for Option<T> {
    fn add_node(input: &mut Input, builder: &mut AstBuilder) -> BlockResult {
        let start = input.checkpoint();
        match T::add_node(input, builder) {
            Ok(o) => Ok(o),
            Err(ErrMode::Backtrack(_)) => {
                input.reset(start);
                Ok(0)
            }
            Err(e) => Err(e),
        }
    }
}
