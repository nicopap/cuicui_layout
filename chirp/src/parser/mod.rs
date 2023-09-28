//! The actual parser for `chirp` files.
use std::fmt;

pub(crate) use grammar::{arg_token_tree, chirp_document};
use stream::TokenType;
pub use stream::{Input, StateCheckpoint, Token};

mod grammar;
mod lex;
mod stream;
#[cfg(test)]
mod tests;

pub type Span = (u32, u32);
pub(crate) trait Itrp<'a>: fmt::Debug + Clone {
    fn code(&self, input: (&[u8], Span));
    fn set_name(&self, span: Span, name: &[u8]);
    fn complete_children(&self);
    fn method(&self, name: &[u8], name_span: Span, args: &[u8], args_span: Span);
    fn t_method(&self, ((name, name_span), (args, args_span)): ((&[u8], Span), (&[u8], Span))) {
        self.method(name, name_span, args, args_span);
    }
    fn spawn_with_children(&self);
    fn insert_entity(&self) {
        self.spawn_with_children();
        self.complete_children();
    }
    fn import(&self, import_source: &[u8], import_span: Span, rename: Option<&[u8]>);
    fn register_fn(&self, name: &'a [u8], parser: StateCheckpoint);
    fn call_template(&self, name: &'a [u8], name_span: Span) -> Option<StateCheckpoint>;
}
impl<'a> Itrp<'a> for () {
    fn code(&self, _: (&[u8], Span)) {}
    fn set_name(&self, _: Span, _: &[u8]) {}
    fn complete_children(&self) {}
    fn method(&self, _: &[u8], _: Span, _: &[u8], _: Span) {}
    fn spawn_with_children(&self) {}
    fn import(&self, _: &[u8], _: Span, _: Option<&[u8]>) {}
    fn register_fn(&self, _: &'a [u8], _: StateCheckpoint) {}
    fn call_template(&self, _: &'a [u8], _: Span) -> Option<StateCheckpoint> {
        None
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("Expected {0}. Instead got {1}")]
    Expected(TokenType, TokenType),
    #[error("Unbalanced delimiter")]
    Unbalanced,
    #[error("Unexpected error: the crate author didn't expect it")]
    Unexpected,
    #[error("Expected '{{' for children statements or '(' for method list. Instead got {0}")]
    StatementDelimiter(TokenType),
    #[error("Expected Entity Name. Instead got {0}")]
    StartStatement(TokenType),
    #[error("Expected a method name (ident). Instead got {0}")]
    BadMethod(TokenType),
}
impl Error {
    pub(crate) const fn help(self) -> &'static str {
        match self {
            Error::Expected(..) => {
                "Check the other errors in the error output. If they are not \
                enough, unhelpful error messages are considered a bug, so please \
                open an issue: \
                https://github.com/nicopap/cuicui_layout/issues\n"
            }
            Error::Unexpected => {
                "This is a bug in the chirp parser, please open an issue at: \
                https://github.com/nicopap/cuicui_layout/issues\n"
            }
            Error::Unbalanced => {
                "Method arguments should balance their curly braces `{}`, \
                brackets `[]` and parenthesis `()`. If you want to pass a \
                string containing unbalanced delimiters, use a string literal \
                such as `\":)\"`.\n"
            }
            Error::StatementDelimiter(_) => {
                "After the `EntityName` beginning a statement, you should \
                specify a `(method list)` or a `{List() Of() Children()}`. \
                If neither apply, use an empty list. Example: `EntityName ()`\
                \n\
                IMPORTANT! You might be getting this error because you are \
                trying to put a space in your entity name. Use a string \
                literal for spaces. Example: `\"Entity Name\"(method list)`.\
                \n\
                Note: A reminder that the chirp file format has no concept of \
                separator, so you'll get this error if you put a `;` or `,` \
                at the end of the last statement as well :)\n"
            }
            Error::StartStatement(_) => {
                "After closing a statement, you should either start a new \
                statement with an `EntityName` or a close the parent's children \
                statements list with a `}`.\n"
            }
            Error::BadMethod(_) => {
                "Methods are declared between parenthesis after the `EntityName`. \
                A single method is either an identifier or a an identifier \
                followed by parenthesis (the arguments to the method).\
                \n\
                You might be getting this error because of an unbalanced \
                parenthesis in a method list.\n"
            }
        }
    }
}
impl<'a, I: Itrp<'a>> winnow::error::ParserError<Input<'_, I>> for Error {
    fn from_error_kind(_: &Input<I>, _: winnow::error::ErrorKind) -> Self {
        Self::Unexpected
    }

    fn append(self, _: &Input<I>, _: winnow::error::ErrorKind) -> Self {
        self
    }
}
impl<'a, I: Itrp<'a>> winnow::error::FromExternalError<Input<'_, I>, Error> for Error {
    fn from_external_error(_: &Input<'_, I>, _: winnow::error::ErrorKind, e: Error) -> Self {
        e
    }
}
