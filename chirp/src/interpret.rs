use std::{borrow::Cow, cell::RefCell, fmt, mem};

use bevy::{
    asset::LoadContext,
    prelude::{trace, BuildChildren, Commands, Entity},
    reflect::{Reflect, TypeRegistryInternal as TypeRegistry},
    utils::HashMap,
};
use smallvec::SmallVec;
use thiserror::Error;
use winnow::{BStr, PResult, Parser};

use crate::{
    parse::{scoped_text, MethodCtx},
    ParseDsl,
};

/// An error occuring when adding a [`crate::Chirp`] to the world.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error)]
pub enum InterpError {
    // TODO(err): Integrate parse spans for nice error reporting.
    #[error("Didn't find the code handle '{0}' in provided code handles")]
    CodeNotPresent(String),
    #[error(transparent)]
    DslError(#[from] anyhow::Error),
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
struct LoadCtx<'h, 'l, 'll, 'r> {
    load: Option<&'l LoadContext<'ll>>,
    reg: &'r TypeRegistry,
    handles: &'h Handles,
}
impl fmt::Debug for LoadCtx<'_, '_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_load = self.load.is_some();
        f.debug_struct("LoadCtx")
            .field("load", &if is_load { "Some(&LoadContext)" } else { "None" })
            .field("reg", &"&TypeRegistry")
            .field("handles", &"&Handles")
            .finish()
    }
}
#[derive(Debug)]
struct InnerInterpreter<'w, 's, 'a, D> {
    cmds: BevyCmds<'w, 's, 'a>,
    current: SmallVec<[Entity; 3]>,
    errors: Vec<InterpError>,
    dsl: D,
}

impl<'w, 's, 'a, D> InnerInterpreter<'w, 's, 'a, D> {
    #[cold]
    fn push_error(&mut self, error: impl Into<InterpError>) {
        self.errors.push(error.into());
    }
}
#[derive(Debug)]
pub(crate) struct Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D> {
    // TODO(perf): Can use an UnsafeCell instead, since we'll never access this
    // concurrently, as the parsing is linear.
    mutable: RefCell<InnerInterpreter<'w, 's, 'a, D>>,
    ctx: LoadCtx<'h, 'l, 'll, 'r>,
}

impl<'w, 's, 'a, 'h, 'l, 'll, 'r> Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, ()> {
    pub fn new<D: ParseDsl>(
        builder: &'a mut Commands<'w, 's>,
        load: Option<&'l LoadContext<'ll>>,
        reg: &'r TypeRegistry,
        handles: &'h Handles,
    ) -> Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D> {
        Interpreter {
            mutable: RefCell::new(InnerInterpreter {
                cmds: BevyCmds(builder),
                current: SmallVec::new(),
                errors: Vec::new(),
                dsl: D::default(),
            }),
            ctx: LoadCtx { load, reg, handles },
        }
    }
}
impl<'w, 's, 'a, 'h, 'l, 'll, 'r, D: ParseDsl> Interpreter<'w, 's, 'a, 'h, 'l, 'll, 'r, D> {
    fn method(&self, method: &[u8], args: impl AsRef<[u8]>) {
        let mut args = args.as_ref();
        if args.first() == Some(&b'(') {
            args = &args[1..args.len() - 1];
        }
        let name = String::from_utf8_lossy(method);
        let args = String::from_utf8_lossy(args);
        trace!("Method: {name} '{args}'");
        let ctx = MethodCtx {
            name,
            args,
            ctx: self.ctx.load,
            registry: self.ctx.reg,
        };
        let interp = &mut self.mutable.borrow_mut();
        if let Err(err) = interp.dsl.method(ctx) {
            interp.push_error(err);
        }
    }
    fn code(&self, name: &[u8]) {
        let b_name = BStr::new(name);
        trace!("Calling registered function {b_name}");
        let Some(code) = self.ctx.handles.get_function_u8(name) else {
            let name = String::from_utf8_lossy(name).to_string();
            let interp = &mut self.mutable.borrow_mut();
            interp.push_error(InterpError::CodeNotPresent(name));
            return;
        };
        let interp = &mut self.mutable.borrow_mut();
        let parent = interp.current.last().copied();
        code(self.ctx.reg, self.ctx.load, interp.cmds.0, parent);
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
    pub fn statements(&self, input: &mut &BStr) -> PResult<(), ()> {
        use winnow::{
            ascii::{escaped, multispace0, multispace1},
            combinator::{
                alt, delimited, dispatch, opt, preceded, repeat, separated0, separated_pair,
                success, terminated,
            },
            token::{one_of, take_till1},
        };
        // Note: we use `void` to reduce the size of input/output types. It's
        // a major source of performance problems in winnow.
        let line_comment = || preceded(b"//", take_till1(b'\n').void());
        let repeat = repeat::<_, _, (), _, _>;
        let spc_trail = || repeat(.., (line_comment(), multispace0));
        let (spc, spc1, opt) = (
            || (multispace0, spc_trail()).void(),
            || multispace1.void(),
            || opt(b' ').void(),
        );
        let ident = || take_till1(b" \n\t;\",()\\{}");

        let methods = &|| {
            let str_literal = delimited(
                b'"',
                escaped(take_till1(b"\\\""), '\\', one_of(b"\\\"")).recognize(),
                b'"',
            );
            let args = alt((
                preceded(spc1(), ident()),
                // TODO(perf): split this in a sane way, re-parsing might be costly
                preceded(spc(), scoped_text),
            ));
            let empty = success::<&BStr, &[u8], _>(b"");
            let method = alt((
                str_literal.map(|i| self.method(b"named", escape_literal(i))),
                (ident(), alt((args, empty))).map(|(n, arg)| self.method(n, arg)),
            ));
            let comma_list = |p| separated0::<&BStr, _, (), _, _, _, _>(p, (b',', spc()));
            delimited(b'(', delimited(spc(), comma_list(method), spc()), b')')
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
        let tail = || alt((b';'.map(terminal), delimited(b'{', nest, b'}')));
        let statement = dispatch! { ident();
            b"code" => {
                let head = preceded(opt(), delimited(b'(', ident(), b')'));
                let head = head.map(|i| self.code(i));
                terminated(head, (opt(), b';'))
            },
            b"spawn" => {
                let head = preceded(opt(), methods());
                separated_pair(head, opt(), tail()).void()
            },
            method => {
                let head = preceded(opt(), methods());
                let head = head.map(|_| self.method(method, b""));
                separated_pair(head, opt(), tail()).void()
            },
        };
        let space_list = |p| separated0::<&BStr, _, (), _, _, _, _>(p, spc());
        let mut statements = delimited(spc(), space_list(statement), spc());
        statements.parse_next(input)
    }
}

fn fast_contains(check: &[u8], contains: u8) -> bool {
    // SAFETY: [u8;4] is a valid u32
    let (head, body, tail) = unsafe { check.align_to::<u32>() };
    let out_of_body = head.iter().chain(tail).any(|c| *c == contains);
    let mask0 = u32::from_le_bytes([0, 0, 0, contains]);
    let mask1 = u32::from_le_bytes([0, 0, contains, 0]);
    let mask2 = u32::from_le_bytes([0, contains, 0, 0]);
    let mask3 = u32::from_le_bytes([contains, 0, 0, 0]);
    out_of_body
        || body.iter().any(|&value| {
            (value & mask0 == mask0)
                | (value & mask1 == mask1)
                | (value & mask2 == mask2)
                | (value & mask3 == mask3)
        })
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
    if fast_contains(to_escape, b'\\') {
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
