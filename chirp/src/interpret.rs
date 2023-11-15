//! Interpret `.chirp` files, spawning entities with a provided [`Commands`].

use std::borrow::Cow;
use std::{any, fmt, fmt::Debug, mem, str};

use bevy::asset::LoadContext;
use bevy::ecs::prelude::{Commands, Entity};
use bevy::hierarchy::BuildChildren;
use bevy::log::{error, trace};
use bevy::reflect::TypeRegistry;
use bevy::utils::HashMap;
use cuicui_dsl::EntityCommands;
use miette::{Diagnostic, NamedSource, SourceSpan};
use smallvec::SmallVec;
use thiserror::Error;
use winnow::BStr;

use crate::parse_dsl::{self, MethodCtx, ParseDsl};
use crate::parser::{self, chirp_file, Arguments, ChirpFile, FnIndex, Input, Name};
use crate::InterpretResult;

type Span = (u32, u32);

/// An error occuring when adding a [`crate::Chirp`] to the world.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error)]
pub enum InterpError {
    // TODO(err): show available handles suggest close ones.
    #[error("Didn't find the code handle '{0}' in provided code handles")]
    CodeNotPresent(Box<str>),
    #[error(transparent)]
    DslError(#[from] anyhow::Error),
    #[error(transparent)]
    ParseError(#[from] parser::Error),
    #[error("The method name is invalid UTF8")]
    BadUtf8MethodName,
    #[error("The method arguments is invalid UTF8")]
    BadUtf8Argument,
    #[error("Method '{0}' is uppercase.")]
    UppercaseMethod(Box<str>),
    #[error("Imports are not supported as of cuicui 0.10")]
    Import,
    #[error("Tried to call {}!, but this template doesn't exist.", BStr::new(&.0))]
    TemplateNotFound(Box<[u8]>),
}
const UTF8_ERROR: &str =
    "Chirp requires UTF8, your file is either corrupted or saved with the wrong encoding.";
impl InterpError {
    fn help_message<D>(&self) -> Option<Box<str>> {
        use crate::parse_dsl::DslParseError;
        use InterpError::{BadUtf8Argument, BadUtf8MethodName, Import, TemplateNotFound};

        match self {
            Self::CodeNotPresent(_) | TemplateNotFound(_) | Import => None,
            Self::DslError(err) => Some(if err.downcast_ref::<DslParseError>().is_some() {
                format!(
                    "{} doesn't contain a method with this name.",
                    any::type_name::<D>()
                )
                .into()
            } else {
                "The error comes from the ParseDsl implementation.".into()
            }),
            Self::ParseError(err) => Some(err.help().into()),
            Self::UppercaseMethod(_) => {
                Some("You probably forgot to close a parenthesis in the last method list.".into())
            }
            BadUtf8MethodName | BadUtf8Argument => Some(UTF8_ERROR.into()),
        }
    }
    fn dsl_offset(&self) -> Option<u32> {
        use crate::parse_dsl::args::ReflectDslDeserError as ReflectError;

        let Self::DslError(err) = self else {
            return None;
        };
        err.downcast_ref().and_then(ReflectError::maybe_offset)
    }
}

pub(crate) fn interpret<'a, D: ParseDsl>(
    input_u8: &[u8],
    builder: &'a mut EntityCommands<'_, '_, 'a>,
    load_ctx: Option<&'a mut LoadContext>,
    reg: &'a TypeRegistry,
    handles: &'a Handles,
) -> InterpretResult<()> {
    let input = Input::new(input_u8, ());
    let ast = match chirp_file(input) {
        parser::ParseResult::Ast(ast) => ast,
        parser::ParseResult::TemplateLibrary(template_library) => {
            return InterpretResult::TemplateLibrary(template_library);
        }
        parser::ParseResult::Err(err, span) => {
            let error = SpannedError::new::<D>(err, span);
            return InterpretResult::Err(Errors::new(vec![error], input_u8, load_ctx.as_deref()));
        }
    };
    let chirp_file = ChirpFile::new(input, ast.as_ref());
    let mut interpreter = Interpreter::<D>::new(builder, load_ctx, reg, handles);
    chirp_file.interpret(&mut interpreter);
    if interpreter.errors.is_empty() {
        InterpretResult::Ok(())
    } else {
        let ctx = interpreter.load_ctx.as_deref();
        InterpretResult::Err(Errors::new(interpreter.errors, input_u8, ctx))
    }
}

// TODO(feat): print call stack.
#[derive(Debug, Error, Diagnostic)]
#[error("{error}")]
struct SpannedError {
    #[label]
    span: SourceSpan,
    error: InterpError,
    #[help]
    help: Option<Box<str>>,
}
impl SpannedError {
    fn new<D>(error: impl Into<InterpError>, (mut start, mut end): Span) -> Self {
        let as_usize = |x: u32| usize::try_from(x).unwrap();
        let error: InterpError = error.into();
        let help = error.help_message::<D>();
        if let Some(offset) = error.dsl_offset() {
            start += offset;
            end = start;
        }
        let span = (as_usize(start)..as_usize(end)).into();
        Self { span, error, help }
    }
}
/// Describe errors encountered while parsing and interpreting a chirp file.
#[derive(Debug, Error, Diagnostic)]
#[diagnostic()]
#[error("Invalid chirp file: {}", NiceErrors(&self.errors))]
pub struct Errors {
    #[source_code] // TODO(perf): Probably can get away without allocation
    source_code: NamedSource,
    #[related]
    errors: Vec<SpannedError>,
}
impl Errors {
    fn new(errors: Vec<SpannedError>, input: &[u8], load_ctx: Option<&LoadContext>) -> Self {
        let str = Cow::Borrowed("Static str");
        let file_name = load_ctx.map_or(str, |l| l.path().to_string_lossy());
        let input = String::from(String::from_utf8_lossy(input));
        let source_code = NamedSource::new(file_name, input);
        Self { source_code, errors }
    }
}
struct NiceSpan(SourceSpan);
impl fmt::Display for NiceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}, {}]",
            self.0.offset(),
            self.0.offset() + self.0.len()
        )
    }
}

struct NiceErrors<'e>(&'e [SpannedError]);
impl fmt::Display for NiceErrors<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, error) in self.0.iter().enumerate() {
            write!(f, "({i}) {error}, ")?;
        }
        Ok(())
    }
}

/// A function called by the `chirp` interpreter when encountering a `code` statement.
///
/// The arguments are as follow:
/// - `&TypeRegistry`: the main app type registry.
/// - `Option<&LoadContext>`: The load context, if in the context of asset loading.
///   this can be used to get arbitrary `Handle<T>`s.
/// - `&mut EntityCommands`: Entity to use for this `code` function.
pub type CodeFunctionBox =
    Box<dyn Fn(&TypeRegistry, Option<&LoadContext>, &mut EntityCommands) + Send + Sync>;

/// Reference-based pendant of [`CodeFunctionBox`]. See `CodeFunctionBox` docs for details.
pub type CodeFunctionRef<'a> =
    &'a (dyn Fn(&TypeRegistry, Option<&LoadContext>, &mut EntityCommands) + Send + Sync);

/// Registry of functions used in `code` block in [`crate::Chirp`]s.
#[derive(Default)]
pub struct Handles {
    funs: HashMap<Box<[u8]>, CodeFunctionBox>,
}
impl Handles {
    /// Create a new empty chirp handle registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Associate `name` with `function` in `chirp` code statements.
    ///
    /// `function` may be called from a `chirp` file from a `code` statement if
    /// `name` is passed as argument.
    ///
    /// Returns any function already associated with provided name, if present.
    pub fn add_function(
        &mut self,
        name: impl Into<String>,
        function: impl Fn(&TypeRegistry, Option<&LoadContext>, &mut EntityCommands)
            + Send
            + Sync
            + 'static,
    ) -> Option<CodeFunctionBox> {
        let name = name.into().into_bytes().into_boxed_slice();
        self.funs.insert(name, Box::new(function))
    }
    /// Get function registered with provided `name`.
    pub fn get_function(&self, name: &impl AsRef<str>) -> Option<CodeFunctionRef> {
        self.funs.get(name.as_ref().as_bytes()).map(Box::as_ref)
    }
    fn get_function_u8(&self, name: &[u8]) -> Option<CodeFunctionRef> {
        self.funs.get(name).map(Box::as_ref)
    }
}

struct LoadCtx<'h, 'r> {
    reg: &'r TypeRegistry,
    handles: &'h Handles,
}
impl Debug for LoadCtx<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadCtx")
            .field("reg", &"&TypeRegistry")
            .finish()
    }
}
struct Interpreter<'w, 's, 'a, 'l, D> {
    ctx: LoadCtx<'a, 'a>,
    cmds: &'a mut Commands<'w, 's>,
    parent_chain: SmallVec<[Entity; 2]>,
    /// The entity on which we are spawning the chirp scene.
    ///
    /// Or the current parent if we are not on the root entity.
    root_entity: Entity,
    templates: HashMap<&'a [u8], FnIndex<'a>>,
    errors: Vec<SpannedError>,
    load_ctx: Option<&'a mut LoadContext<'l>>,
    dsl: D,
}
impl<'w, 's, 'a, 'l, D> fmt::Debug for Interpreter<'w, 's, 'a, 'l, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interpreter")
            .field("cmds", &"BevyCmds(Commands)")
            .field("current", &self.parent_chain)
            .field("errors", &self.errors)
            .field("dsl", &std::any::type_name::<D>())
            .field("ctx", &self.ctx)
            .finish()
    }
}

impl<'w, 's, 'a, 'l, D: ParseDsl> Interpreter<'w, 's, 'a, 'l, D> {
    fn new(
        builder: &'a mut EntityCommands<'w, 's, 'a>,
        load_ctx: Option<&'a mut LoadContext<'l>>,
        reg: &'a TypeRegistry,
        handles: &'a Handles,
    ) -> Self {
        let root_entity = builder.id();
        let cmds = builder.commands();
        Interpreter {
            ctx: LoadCtx { reg, handles },
            cmds,
            parent_chain: SmallVec::new(),
            templates: HashMap::new(),
            errors: Vec::new(),
            dsl: D::default(),
            load_ctx,
            root_entity,
        }
    }
    #[cold]
    fn push_error(&mut self, span: Span, error: impl Into<InterpError>) {
        self.errors.push(SpannedError::new::<D>(error, span));
    }

    fn statement_spawn(&mut self) -> Option<Entity> {
        trace!("Inserting DSL");

        let mut dsl = mem::take(&mut self.dsl); // we set to the default D

        // - no parent: we are root, use root_entity
        // - parent, but equal to root_entity: means we have a single parent use any
        // - parent, different to root_entity: use root_entity
        let mut cmds = if self.parent_chain.last().is_none() {
            self.cmds.entity(self.root_entity)
        } else {
            let mut cmds = self.cmds.spawn_empty();
            cmds.set_parent(self.root_entity);
            cmds
        };
        self.errors.is_empty().then(|| {
            dsl.insert(&mut cmds);
            cmds.id()
        })
    }
}
impl<'w, 's, 'a, 'l, D: ParseDsl> parser::Interpreter<'a, 'a> for Interpreter<'w, 's, 'a, 'l, D> {
    fn spawn_leaf(&mut self) {
        self.statement_spawn();
    }
    fn method(&mut self, (name, name_span): Name<'a>, arguments: &Arguments) {
        use crate::parse_dsl::DslParseError;

        let Ok(name) = str::from_utf8(name) else {
            let error = InterpError::BadUtf8MethodName;
            self.push_error(name_span, error);
            return;
        };
        if name.starts_with(char::is_uppercase) {
            let error = InterpError::UppercaseMethod(name.into());
            self.push_error(name_span, error);
            return;
        }
        trace!("Method: {name}{arguments}");
        let Self { load_ctx, dsl, .. } = self;
        let args_span = arguments.span().unwrap_or(name_span);
        let ctx = MethodCtx {
            name,
            arguments: arguments.into(),
            ctx: load_ctx.as_deref_mut(),
            registry: self.ctx.reg,
        };
        if let Err(err) = dsl.method(ctx) {
            let is_name_err = err.downcast_ref::<DslParseError>().is_some();
            let span = if is_name_err { name_span } else { args_span };
            self.push_error(span, err);
        }
    }
    fn start_children(&mut self) {
        let inserted = self.statement_spawn();
        trace!(">>> Going deeper now…");
        let Self { root_entity, parent_chain, .. } = self;
        // for the `statement_spawn` to correctly pick a parent:
        // - no parent: push inserted, set root_entity to inserted.
        // - parent equal to root_entity: set root_entity to inserted
        // - parent not equal to root_entity: push root_entity, set root_entity to inserted
        let inserted = inserted.unwrap_or(*root_entity);
        match parent_chain.last() {
            None => {
                parent_chain.push(inserted);
                *root_entity = inserted;
            }
            Some(entity) if entity == root_entity => *root_entity = inserted,
            Some(_) => parent_chain.push(mem::replace(root_entity, inserted)),
        }
    }
    fn code(&mut self, (identifier, span): Name<'a>) {
        let b_name = BStr::new(identifier);
        trace!("Calling registered function {b_name}");
        let Some(code) = self.ctx.handles.get_function_u8(identifier) else {
            let name = String::from_utf8_lossy(identifier);
            self.push_error(span, InterpError::CodeNotPresent(name.into()));
            return;
        };
        let load_ctx = self.load_ctx.as_deref();
        let mut cmds = self.cmds.spawn_empty();
        cmds.set_parent(self.root_entity);
        code(self.ctx.reg, load_ctx, &mut cmds);
    }

    fn set_name(&mut self, (name, span): Name) {
        trace!("= node {} =", BStr::new(name));
        let ctx = MethodCtx {
            name: "named",
            arguments: parse_dsl::Arguments::for_name(name),
            ctx: self.load_ctx.as_deref_mut(),
            registry: self.ctx.reg,
        };
        if let Err(err) = self.dsl.method(ctx) {
            self.push_error(span, err);
        }
    }
    fn complete_children(&mut self) {
        let Self { root_entity, parent_chain, .. } = self;
        trace!("<<< Ended spawning entities within statements block, continuing");
        let pop_msg = "MAJOR cuicui_chirp BUG: please open an issue 🥺 plleaaaaasse\n\
            The parser called the interpreter's pop function more times than it \
            called its push function, which should never happen.";
        let entity = parent_chain.pop().expect(pop_msg);
        // for the `statement_spawn` to still be in "child of root entity" mode,
        // we need to add back the last parent of the chain.
        if parent_chain.is_empty() {
            parent_chain.push(entity);
        }
        *root_entity = entity;
    }

    fn import(&mut self, _file: Name<'a>, name: Name<'a>, _alias: Option<Name>) {
        self.push_error(name.1, InterpError::Import);
    }

    fn register_fn(&mut self, (name, _): Name<'a>, index: FnIndex<'a>) {
        self.templates.insert(name, index);
    }

    fn get_template(&mut self, (name, span): Name<'a>) -> Option<FnIndex<'a>> {
        if let Some(key) = self.templates.get(name) {
            trace!("<<--- {}", BStr::new(name));
            return Some(*key);
        }
        self.push_error(span, InterpError::TemplateNotFound(name.into()));
        None
    }
}

#[cfg(never)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        let output = escape_literal(br#"ab\\c\\\d\e"#);
        assert_eq!(BStr::new(br#"ab\c\de"#), BStr::new(&output));
    }
    #[test]
    fn test_escape_escape_escape_first() {
        let output = escape_literal(br#"\\ab\\c\\\d\ef\g\h\\"#);
        assert_eq!(BStr::new(br#"\ab\c\defgh\"#), BStr::new(&output));
    }
    #[test]
    fn test_escape_escape_first() {
        let output = escape_literal(br#"\ab\\c\\\de\\"#);
        assert_eq!(BStr::new(br#"ab\c\de\"#), BStr::new(&output));
    }
}
