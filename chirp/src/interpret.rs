//! Interpret `.chirp` files, spawning entities with a provided [`Commands`].

use std::{borrow::Cow, cell::RefCell, fmt, fmt::Debug, mem, ops::Range, str};

use bevy::asset::LoadContext;
use bevy::ecs::prelude::{Commands, Entity};
use bevy::hierarchy::BuildChildren;
use bevy::log::{error, trace};
use bevy::reflect::{Reflect, TypeRegistryInternal as TypeRegistry};
use bevy::utils::HashMap;
use miette::{Diagnostic, NamedSource, SourceSpan};
use smallvec::SmallVec;
use thiserror::Error;
use winnow::BStr;

use crate::grammar;
use crate::parse::{MethodCtx, ParseDsl};

/// An error occuring when adding a [`crate::Chirp`] to the world.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error)]
pub enum InterpError {
    // TODO(err): show available handles suggest close ones.
    #[error("Didn't find the code handle '{0}' in provided code handles")]
    CodeNotPresent(String),
    #[error(transparent)]
    DslError(#[from] anyhow::Error),
    // TODO(err): better error messages
    #[error("Bad syntax")]
    ParseError,
    #[error("The method name is invalid UTF8")]
    BadUtf8MethodName,
    #[error("The method arguments is invalid UTF8")]
    BadUtf8Argument,
}
const UTF8_ERROR: &str =
    "Chirp requires UTF8, your file is either corrupted or saved with the wrong encoding.";
const PARSE_ERROR: &str = "\
More actionable error messages are coming, until then, check the grammar at:

https://github.com/nicopap/cuicui_layout/blob/712f19d58eea48d50dde6ed4ed4c1b42ac6f2544/design_docs/layout_format.md#grammar";
impl InterpError {
    const fn help_message(&self) -> Option<&'static str> {
        use InterpError::{BadUtf8Argument, BadUtf8MethodName};
        match self {
            InterpError::CodeNotPresent(_) => None,
            InterpError::DslError(_) => Some("The error comes from the ParseDsl implementation"),
            InterpError::ParseError => Some(PARSE_ERROR),
            BadUtf8MethodName | BadUtf8Argument => Some(UTF8_ERROR),
        }
    }
}
#[derive(Debug, Error, Diagnostic)]
#[error("{error} {}", NiceSpan(self.span))]
struct SpannedError {
    #[label]
    span: SourceSpan,
    error: InterpError,
    #[help]
    help: Option<&'static str>,
}
impl SpannedError {
    fn new(error: impl Into<InterpError>, span: impl Into<SourceSpan>) -> Self {
        let (error, span) = (error.into(), span.into());
        let help = error.help_message();
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

// TODO(feat): Consider replacing this with a trait that takes
// `handle(&str, &mut ChildBuilder)`, so that it is concievable of not relying
// on dynamic dispatch.
/// A function called by the `chirp` interpreter when encountering a `code` statement.
///
/// The arguments are as follow:
/// - `&TypeRegistry`: the main app type registry.
/// - `Option<&LoadContext>`: The load context, if in the context of asset loading.
///   this can be used to get arbitrary `Handle<T>`s.
/// - `&mut Commands`: Commands to spawn entities.
/// - `Option<Entity>`: the current parent, if there is one.
// TODO(0.10): Replace with &mut EntityCommands, since we now require 0-1 entity per statement.
pub type CodeFunctionBox =
    Box<dyn Fn(&TypeRegistry, Option<&LoadContext>, &mut Commands, Option<Entity>) + Send + Sync>;

/// Reference-based pendant of [`CodeFunctionBox`]. See `CodeFunctionBox` docs for details.
pub type CodeFunctionRef<'a> =
    &'a (dyn Fn(&TypeRegistry, Option<&LoadContext>, &mut Commands, Option<Entity>) + Send + Sync);

/// Registry of functions used in `code` block in [`crate::Chirp`]s.
#[derive(Default)]
pub struct Handles {
    funs: HashMap<Box<[u8]>, CodeFunctionBox>,
    refs: HashMap<Box<[u8]>, Box<dyn Reflect>>,
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
        name: String,
        function: impl Fn(&TypeRegistry, Option<&LoadContext>, &mut Commands, Option<Entity>)
            + Send
            + Sync
            + 'static,
    ) -> Option<CodeFunctionBox> {
        let name = name.into_bytes().into_boxed_slice();
        self.funs.insert(name, Box::new(function))
    }
    /// Associate `name` with provided `value`.
    ///
    /// Note that the name pool for functions and values are distinct.
    ///
    /// This is currently unused, you can call [`Self::get_ref`] to get back
    /// the registered value.
    ///
    /// Returns any value already associated with provided name, if present.
    pub fn add_ref(&mut self, name: String, value: impl Reflect) -> Option<Box<dyn Reflect>> {
        let name = name.into_bytes().into_boxed_slice();
        self.refs.insert(name, Box::new(value))
    }
    /// Get function registered with provided `name`.
    pub fn get_function(&self, name: &impl AsRef<str>) -> Option<CodeFunctionRef> {
        self.funs.get(name.as_ref().as_bytes()).map(Box::as_ref)
    }
    /// Get value registered with provided `name`.
    pub fn get_ref(&self, name: &impl AsRef<str>) -> Option<&dyn Reflect> {
        self.refs.get(name.as_ref().as_bytes()).map(Box::as_ref)
    }
    fn get_function_u8(&self, name: &[u8]) -> Option<CodeFunctionRef> {
        self.funs.get(name).map(Box::as_ref)
    }
}

struct BevyCmds<'w, 's, 'a>(&'a mut Commands<'w, 's>);
impl Debug for BevyCmds<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BevyCmds(Commands)")
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
            .field("handles", &"&Handles")
            .finish()
    }
}
struct InnerInterpreter<'w, 's, 'a, 'l, 'll, D> {
    cmds: BevyCmds<'w, 's, 'a>,
    current: SmallVec<[Entity; 3]>,
    errors: Vec<SpannedError>,
    load_ctx: Option<&'l mut LoadContext<'ll>>,
    dsl: D,
}
impl<D> Debug for InnerInterpreter<'_, '_, '_, '_, '_, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_load_ctx = self.load_ctx.is_some();
        let load_ctx = if is_load_ctx { "Some(&mut LoadContext)" } else { "None" };
        f.debug_struct("InnerInterpreter")
            .field("cmds", &self.cmds)
            .field("current", &self.current)
            .field("errors", &self.errors)
            .field("load_ctx", &load_ctx)
            .field("dsl", &std::any::type_name::<D>())
            .finish()
    }
}

impl<'w, 's, 'a, 'l, 'll, D> InnerInterpreter<'w, 's, 'a, 'l, 'll, D> {
    #[cold]
    fn push_error(&mut self, span: Range<usize>, error: impl Into<InterpError>) {
        self.errors.push(SpannedError::new(error, span));
    }
}
pub(crate) struct Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D> {
    // TODO(perf): Can use an UnsafeCell instead, since we'll never access this
    // concurrently, as the parsing is linear.
    mutable: RefCell<InnerInterpreter<'w, 's, 'a, 'l, 'll, D>>,
    ctx: LoadCtx<'h, 'r>,
}
impl<'w, 's, 'a, 'h, 'l, 'll, 'r, D> fmt::Debug for Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interpreter")
            .field("mutable", &self.mutable)
            .field("ctx", &self.ctx)
            .finish()
    }
}

impl<'w, 's, 'a, 'h, 'l, 'll, 'r> Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, ()> {
    pub fn new<D: ParseDsl>(
        builder: &'a mut Commands<'w, 's>,
        load_ctx: Option<&'l mut LoadContext<'ll>>,
        reg: &'r TypeRegistry,
        handles: &'h Handles,
    ) -> Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D> {
        Interpreter {
            mutable: RefCell::new(InnerInterpreter {
                cmds: BevyCmds(builder),
                current: SmallVec::new(),
                errors: Vec::new(),
                dsl: D::default(),
                load_ctx,
            }),
            ctx: LoadCtx { reg, handles },
        }
    }
}
impl<'w, 's, 'a, 'h, 'l, 'll, 'r, D: ParseDsl> Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D> {
    fn statement_spawn(&self) -> Entity {
        trace!("Spawning an entity with provided methods!");
        let interp = &mut *self.mutable.borrow_mut();

        let parent = interp.current.last().copied();
        let mut dsl = mem::take(&mut interp.dsl); // we set to the default D
        let mut cmds = interp.cmds.0.spawn_empty();
        if let Some(parent) = parent {
            cmds.set_parent(parent);
        }
        dsl.insert(&mut cmds)
    }
    fn push_children(&self) {
        let current = self.statement_spawn();
        trace!(">>> Going deeper now…");
        self.mutable.borrow_mut().current.push(current);
    }
    fn pop_children(&self) {
        trace!("<<< Ended spawning entities within statements block, continuing");
        self.mutable.borrow_mut().current.pop();
    }
    pub fn interpret(&mut self, input: &[u8]) -> Result<(), Errors> {
        let stateful_input = crate::lex::Stateful::new(BStr::new(input), &*self);
        let parse_error = grammar::chirp_document(stateful_input);
        let mut errors = mem::take(&mut self.mutable.borrow_mut().errors);
        if let Err(err) = parse_error {
            let input = err.input();
            let start = err.offset();
            let end = start + input.len();
            let error = SpannedError::new(InterpError::ParseError, start..end);
            errors.push(error);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            let str = Cow::Borrowed("Static str");
            let interp = self.mutable.borrow();
            let load_ctx = interp.load_ctx.as_ref();
            let file_name = load_ctx.map_or(str, |l| l.path().to_string_lossy());
            let source_code = String::from_utf8_lossy(input).to_string();
            let source_code = NamedSource::new(file_name, source_code);
            Err(Errors { source_code, errors })
        }
    }
}
impl<'i, 'w, 's, 'a, 'h, 'l, 'll, 'r, D: ParseDsl> grammar::Itrp
    for &'i Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D>
{
    fn method(&self, span: Range<usize>, method: &[u8], args: Option<&[u8]>) {
        let Ok(name) = str::from_utf8(method) else {
            let error = InterpError::BadUtf8MethodName;
            self.mutable.borrow_mut().push_error(span, error);
            return;
        };
        let Ok(args) = args.map_or(Ok(""), str::from_utf8) else {
            let error = InterpError::BadUtf8Argument;
            self.mutable.borrow_mut().push_error(span, error);
            return;
        };
        trace!("Method: {name} '{args}'");
        let interp = &mut *self.mutable.borrow_mut();
        let InnerInterpreter { load_ctx, dsl, .. } = interp;
        let ctx = MethodCtx {
            name,
            args,
            ctx: load_ctx.as_deref_mut(),
            registry: self.ctx.reg,
        };
        if let Err(err) = dsl.method(ctx) {
            interp.push_error(span, err);
        }
    }

    fn spawn(&self) {
        self.push_children();
    }
    fn spawn_leaf(&self) {
        self.statement_spawn();
    }
    fn code(&self, (identifier, span): (&[u8], Range<usize>)) {
        let b_name = BStr::new(identifier);
        trace!("Calling registered function {b_name}");
        let Some(code) = self.ctx.handles.get_function_u8(identifier) else {
            let name = String::from_utf8_lossy(identifier).to_string();
            let interp = &mut self.mutable.borrow_mut();
            interp.push_error(span, InterpError::CodeNotPresent(name));
            return;
        };
        let interp = &mut *self.mutable.borrow_mut();
        let parent = interp.current.last().copied();
        let load_ctx = interp.load_ctx.as_deref();
        code(self.ctx.reg, load_ctx, interp.cmds.0, parent);
    }

    fn entity(&self, span: Range<usize>, name: Option<&[u8]>) {
        if let Some(name) = name {
            self.method(span, b"named", Some(name));
        }
    }

    fn complete(self) {
        self.pop_children();
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
