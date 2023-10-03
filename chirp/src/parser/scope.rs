use std::{borrow::Cow, fmt};

use winnow::BStr;

use super::{ast, Input, Span};

/// Values with special behavior when substituing
#[derive(Debug, Clone)]
pub(super) struct Special<'a>(Option<&'a ast::Argument>);

#[derive(Debug, Clone)]
pub(super) struct Parameters<'a> {
    pub(super) idents: &'a [ast::IdentOffset],
    pub(super) values: &'a [ast::Argument],
    pub(super) special_values: Box<[Special<'a>]>,
}

impl<'a> Parameters<'a> {
    pub(super) fn empty() -> Self {
        let special_values = Box::new([]);
        Self { idents: &[], values: &[], special_values }
    }
    fn replace<'i>(&self, inp: &Input<'i>, arg: &'i [u8]) -> Option<&'i [u8]> {
        // TODO(bug): Need to replace also when identifer is not root
        let idents = self.idents.iter().copied();
        let values = self.values.iter().copied();

        if self.special_values.is_empty() {
            let mut iter = idents.zip(values);
            iter.find_map(|(ident, value)| (ident.read(inp) == arg).then(|| value.read(inp)))
        } else {
            let mut iter = idents.zip(values.zip(self.special_values.iter()));
            iter.find_map(|(ident, (value, special))| {
                let value = special.0.map_or(value, |t| *t);
                (ident.read(inp) == arg).then(|| value.read(inp))
            })
        }
    }

    fn values(&self) -> impl Iterator<Item = &'a ast::Argument> + '_ {
        let get_special = |i: usize| self.special_values.get(i).and_then(|a| a.0);
        self.values
            .iter()
            .enumerate()
            .map(move |(i, v)| get_special(i).unwrap_or(v))
    }

    // TODO(clean): This function is a mess.
    // Edge cases:
    // - We are "forwarding" a parameter. But lo! that parameter itself is forwarded,
    //   so we need to search it in the "special values" thingy.
    pub(crate) fn scope(
        &self,
        idents: &'a [ast::IdentOffset],
        values: &'a [ast::Argument],
        inp: &Input,
    ) -> Self {
        let any_special_values = self.idents.iter().any(|caller_parameter| {
            let param = caller_parameter.read(inp);
            values.iter().any(|value| param == value.read(inp))
        });
        let special_values = if any_special_values {
            let idents = self.idents.iter().copied();
            let map_special = |value: &ast::Argument| {
                let mut iter = idents.clone().zip(self.values());
                let special = iter.find_map(|(param, scope_value)| {
                    (value.read(inp) == param.read(inp)).then_some(Special(Some(scope_value)))
                });
                special.unwrap_or(Special(None))
            };
            values.iter().map(map_special).collect()
        } else {
            Box::default()
        };
        Self { idents, values, special_values }
    }
}

pub struct Arguments<'i, 'a> {
    pub(super) input: Input<'i>,
    pub(super) method_args: &'a [ast::Argument],
    parameters: &'a Parameters<'a>,
}
impl<'i, 'a> Arguments<'i, 'a> {
    pub(super) const fn new(
        input: Input<'i>,
        method_args: &'a [ast::Argument],
        parameters: &'a Parameters<'a>,
    ) -> Self {
        Self { input, method_args, parameters }
    }
    pub const fn len(&self) -> usize {
        self.method_args.len()
    }
    pub fn get(&self, index: usize) -> Option<Cow<'i, [u8]>> {
        let content = self.method_args.get(index)?.read(&self.input);
        Some(Cow::Borrowed(self.replace(content)))
    }

    pub(crate) fn span(&self) -> Option<Span> {
        let start = self.method_args.first()?.start;
        let end = self.method_args.last()?.end;
        Some((start, end))
    }

    fn replace(&self, method_arg: &'i [u8]) -> &'i [u8] {
        self.parameters
            .replace(&self.input, method_arg)
            .unwrap_or(method_arg)
    }
}

impl fmt::Display for Arguments<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.method_args.is_empty() {
            return Ok(());
        }
        f.write_str("(")?;
        let mut first_in_list = true;
        for method_arg in self.method_args {
            if !first_in_list {
                f.write_str(", ")?;
            }
            first_in_list = false;
            let content = method_arg.read(&self.input);
            let tree = BStr::new(self.replace(content));
            write!(f, "{tree}")?;
        }
        f.write_str(")")
    }
}
