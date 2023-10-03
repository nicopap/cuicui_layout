//! The actual parser for `chirp` files.

use stream::TokenType;

pub(crate) use grammar::chirp_file;
pub use interpret::{ChirpFile, FnIndex, Interpreter, Name, Span};
pub use scope::Arguments;
pub use stream::{Input, StateCheckpoint, Token};

mod ast;
mod grammar;
mod interpret;
mod lex;
mod scope;
mod stream;
#[cfg(test)]
mod tests;

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
    #[error("The chirp file is invalid: got additional text after the root statement")]
    TrailingText,
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
            Error::TrailingText => {
                "Chirp files define a single entity. This means that there can \
                only be a single root statement. Try wrapping your statements \
                inside a single root statement."
            }
        }
    }
}
impl winnow::error::ParserError<Input<'_>> for Error {
    fn from_error_kind(_: &Input, _: winnow::error::ErrorKind) -> Self {
        Self::Unexpected
    }

    fn append(self, _: &Input, _: winnow::error::ErrorKind) -> Self {
        self
    }
}
impl winnow::error::FromExternalError<Input<'_>, Error> for Error {
    fn from_external_error(_: &Input<'_>, _: winnow::error::ErrorKind, e: Error) -> Self {
        e
    }
}
