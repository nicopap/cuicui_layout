//! Interpret `.chirp` files, spawning entities with a provided [`Commands`].

use std::{borrow::Cow, cell::RefCell, fmt, mem, ops::Range, str};

use bevy::asset::LoadContext;
use bevy::ecs::prelude::{Commands, Entity};
use bevy::hierarchy::BuildChildren;
use bevy::log::{error, trace, warn};
use bevy::reflect::{Reflect, TypeRegistryInternal as TypeRegistry};
use bevy::utils::HashMap;
use miette::{Diagnostic, NamedSource, SourceSpan};
use smallvec::SmallVec;
use thiserror::Error;
use winnow::{BStr, Located, PResult, Parser};

use crate::parse::{scoped_text, MethodCtx, ParseDsl};

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
impl fmt::Debug for BevyCmds<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BevyCmds(Commands)")
    }
}
struct LoadCtx<'h, 'r> {
    reg: &'r TypeRegistry,
    handles: &'h Handles,
}
impl fmt::Debug for LoadCtx<'_, '_> {
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
impl<D: fmt::Debug> fmt::Debug for InnerInterpreter<'_, '_, '_, '_, '_, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_load_ctx = self.load_ctx.is_some();
        let load_ctx = if is_load_ctx { "Some(&mut LoadContext)" } else { "None" };
        f.debug_struct("InnerInterpreter")
            .field("cmds", &self.cmds)
            .field("current", &self.current)
            .field("errors", &self.errors)
            .field("load_ctx", &load_ctx)
            .field("dsl", &self.dsl)
            .finish()
    }
}

impl<'w, 's, 'a, 'l, 'll, D> InnerInterpreter<'w, 's, 'a, 'l, 'll, D> {
    #[cold]
    fn push_error(&mut self, span: Range<usize>, error: impl Into<InterpError>) {
        self.errors.push(SpannedError::new(error, span));
    }
}
#[derive(Debug)]
pub(crate) struct Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D> {
    // TODO(perf): Can use an UnsafeCell instead, since we'll never access this
    // concurrently, as the parsing is linear.
    mutable: RefCell<InnerInterpreter<'w, 's, 'a, 'l, 'll, D>>,
    ctx: LoadCtx<'h, 'r>,
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
    fn method(&self, span: Range<usize>, method: &[u8], args: impl AsRef<[u8]>) {
        let mut args = args.as_ref();
        if args.first() == Some(&b'(') {
            args = &args[1..args.len() - 1];
        }
        let Ok(name) = str::from_utf8(method) else {
            let error = InterpError::BadUtf8MethodName;
            self.mutable.borrow_mut().push_error(span, error);
            return;
        };
        let Ok(args) = str::from_utf8(args) else {
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
    fn code(&self, span: Range<usize>, name: &[u8]) {
        let b_name = BStr::new(name);
        trace!("Calling registered function {b_name}");
        let Some(code) = self.ctx.handles.get_function_u8(name) else {
            let name = String::from_utf8_lossy(name).to_string();
            let interp = &mut self.mutable.borrow_mut();
            interp.push_error(span, InterpError::CodeNotPresent(name));
            return;
        };
        let interp = &mut *self.mutable.borrow_mut();
        let parent = interp.current.last().copied();
        let load_ctx = interp.load_ctx.as_deref();
        code(self.ctx.reg, load_ctx, interp.cmds.0, parent);
    }
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
        let spanned_input = Located::new(BStr::new(input));
        let mut parser = |i: &mut _| self.statements(i);
        let parse_error = parser.parse(spanned_input);
        let mut errors = mem::take(&mut self.mutable.borrow_mut().errors);
        if let Err(err) = parse_error {
            let error = SpannedError::new(InterpError::ParseError, err.offset()..err.offset());
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
    // TODO(perf): Can get away with a custom stream type that stores span
    // in a (u32, u32) so that it only occupies
    // 16 bytes (1 pointer: usize + start: u32 + end: u32) instead of
    // 32 bytes (2 times (pointer: usize + len: usize))
    // concievably, we could store the pointer in this struct and only hold
    // spans in the parser state.
    fn statements(&self, input: &mut Located<&BStr>) -> PResult<(), ()> {
        use winnow::{
            ascii::{escaped, multispace0, multispace1},
            combinator::{
                alt, cut_err, delimited as delim, dispatch, opt, preceded as starts, repeat,
                separated0, separated_pair, terminated,
            },
            token::{one_of, take_till1 as until},
        };
        // Note: we use `void` to reduce the size of input/output types. It's
        // a major source of performance problems in winnow.
        let line_comment = || starts(b"//", until(b'\n').void());
        let repeat = repeat::<_, _, (), _, _>;
        let spc_trail = || repeat(.., (line_comment(), multispace0));
        let (spc, spc1, opt_spc) = (
            || (multispace0, spc_trail()).void(),
            || multispace1.void(),
            || opt(b' ').void(),
        );
        let ident = || until(b" \n\t;\",()\\{}");

        let methods = &|| {
            let str_literal = || {
                delim(
                    b'"',
                    escaped(until(b"\\\""), '\\', one_of(b"\\\"")).recognize(),
                    b'"',
                )
            };
            let args = alt((
                starts(spc1(), ident()),
                // TODO(perf): split this in a sane way, re-parsing might be costly
                starts(spc(), scoped_text),
                starts(spc1(), str_literal()),
            ));
            let method = alt((
                str_literal()
                    .with_span()
                    .map(|(i, span)| self.method(span, b"named", escape_literal(i))),
                (ident(), opt(args))
                    .with_span()
                    .map(|((n, arg), span)| self.method(span, n, arg.unwrap_or(b""))),
            ));
            let comma_list = |p| separated0::<_, _, (), _, _, _, _>(p, (b',', spc()));
            cut_err(delim(
                b'(',
                delim(spc(), comma_list(cut_err(method)), spc()),
                b')',
            ))
        };
        let nest = |i: &mut _| {
            self.push_children();
            let ret = self.statements(i);
            self.pop_children();
            ret
        };
        let terminal = |_| {
            self.statement_spawn();
        };
        let tail = || alt((b';'.map(terminal), delim(b'{', nest, b'}')));
        let statement = dispatch! { ident();
            reserved_keyword @ (
                b"abstract" | b"as" | b"async" | b"await" | b"become"
                | b"box" | b"break" | b"const" | b"continue" | b"crate"
                | b"do" | b"dyn" | b"else" | b"enum" | b"extern"
                | b"false" | b"final" | b"fn" | b"for" | b"if"
                | b"impl" | b"in" | b"let" | b"loop" | b"macro"
                | b"match" | b"mod" | b"move" | b"mut" | b"override"
                | b"priv" | b"pub" | b"ref" | b"return" | b"self"
                | b"static" | b"struct" | b"super" | b"trait" | b"true"
                | b"try" | b"type" | b"typeof" | b"unsafe" | b"unsized"
                | b"use" | b"virtual" | b"where" | b"while" | b"yeet"
            ) => {
                let show_keyword = BStr::new(reserved_keyword);
                warn!(
                    "Encountered a '{show_keyword}' statement! \
                    All rust keywords are reserved in statement position. \
                    Currently this is equivalent to 'spawn', but may change \
                    without notice in the future."
                );
                let head = starts(opt_spc(), methods());
                separated_pair(head, opt_spc(), tail()).void()
            },
            b"code" => {
                let head = starts(opt_spc(), delim(b'(', ident(), b')'));
                let head = head.with_span().map(|(i, span)| self.code(span, i));
                terminated(head, (opt_spc(), b';'))
            },
            b"spawn" | b"entity" => {
                let head = starts(opt_spc(), methods());
                separated_pair(head, opt_spc(), tail()).void()
            },
            method => {
                let head = starts(opt_spc(), methods());
                let head = head.with_span().map(|(_, span)| self.method(span, method, b""));
                separated_pair(head, opt_spc(), tail()).void()
            },
        };
        let space_list = |p| separated0::<_, _, (), _, _, _, _>(p, spc());
        let mut statements = cut_err(delim(spc(), space_list(statement), spc()));
        statements.parse_next(input)
    }
}

type Swar = u32;
const LANES: usize = 8;
const SWAR_BYTES: usize = (Swar::BITS / 8) as usize;

#[allow(clippy::verbose_bit_mask)] // what a weird lint
fn contains_swar(mut xored: Swar) -> bool {
    // For a position, nothing easier: pos = 0; pos += ret; ret &= xored & 0xff != 0;
    let mut ret = false;
    for _ in 0..SWAR_BYTES {
        ret |= xored & 0xff == 0;
        xored >>= 8;
    }
    ret
}

fn fast_contains<const WHAT: u8>(check: &[u8]) -> bool {
    let mask = Swar::from_le_bytes([WHAT; SWAR_BYTES]);

    // SAFETY: [u8; SWAR_BYTES] is a valid Swar
    let (head, body, tail) = unsafe { check.align_to::<[Swar; LANES]>() };

    head.iter().chain(tail).any(|c| *c == WHAT)
        || body
            .iter()
            .map(|vs| {
                vs.iter()
                    .fold(false, |acc, &v| acc | contains_swar(v ^ mask))
            })
            .any(Into::into)
}
fn escape_literal(to_escape: &[u8]) -> Cow<[u8]> {
    #[cold]
    fn owned(bytes: &[u8]) -> Cow<[u8]> {
        let mut ret = bytes.to_vec();
        let mut prev_bs = false;
        ret.retain(|c| {
            let is_bs = c == &b'\\';
            let keep = !is_bs | prev_bs;
            prev_bs = !keep & is_bs;
            keep
        });
        Cow::Owned(ret)
    }
    if fast_contains::<b'\\'>(to_escape) {
        owned(to_escape)
    } else {
        Cow::Borrowed(to_escape)
    }
}
#[cfg(test)]
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
