use std::{cell::RefCell, fmt, mem};

use bevy::{
    asset::LoadContext,
    prelude::{trace, BuildChildren, ChildBuilder, Commands, Entity},
    reflect::TypeRegistryInternal as TypeRegistry,
    utils::HashMap,
};
use smallvec::SmallVec;
use thiserror::Error;
use winnow::{stream::AsChar, BStr, PResult, Parser};

use crate::{
    parse::{scoped_text, MethodCtx},
    ParseDsl,
};

/// An error occuring when adding a [`crate::Chirp`] to the world.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error)]
pub enum InterpError {
    // TODO(err): Integrate parse spans for nice error reporting.
    #[error("'code' should have exactly one string argument, none were given")]
    BadCode,
    #[error("'code' should be a rust identifier, found '{0}'")]
    CodeNonIdent(String),
    #[error("Didn't find the code handle '{0}' in provided code handles")]
    CodeNotPresent(String),
    #[error("leaf nodes should have at least one argument to be passed as as value")]
    LeafNoArgs,
    #[error(transparent)]
    DslError(#[from] anyhow::Error),
}

// TODO(clean) TODO(feat): Consider replacing this with a trait that takes
// `handle(&str, &mut ChildBuilder)`, so that it is concievable of not relying
// on dynamic dispatch.
/// Registry of functions used in `code` block in [`crate::Chirp`]s.
pub type Handles<'h> = HashMap<String, &'h dyn Fn(&mut ChildBuilder)>;

struct BevyCmds<'w, 's, 'a>(&'a mut Commands<'w, 's>);
impl fmt::Debug for BevyCmds<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BevyCmds(Commands)")
    }
}
struct LoadCtx<'l, 'll, 'r>(Option<&'l LoadContext<'ll>>, &'r TypeRegistry);
impl fmt::Debug for LoadCtx<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_some() {
            write!(f, "LoadCtx(Some(&LoadContext), &TypeRegistry)")
        } else {
            write!(f, "LoadCtx(None, &TypeRegistry)")
        }
    }
}
#[derive(Debug)]
struct InnerInterpreter<'w, 's, 'a, D> {
    cmds: BevyCmds<'w, 's, 'a>,
    current: SmallVec<[Entity; 3]>,
    dsl: D,
}
// TODO(clean): Use named struct.
// TODO(perf): Can use an UnsafeCell instead, since we'll never access this
// concurrently, as the parsing is linear.
#[derive(Debug)]
pub(crate) struct Interpreter<'w, 's, 'a, 'l, 'll, 'r, D>(
    RefCell<InnerInterpreter<'w, 's, 'a, D>>,
    LoadCtx<'l, 'll, 'r>,
);

impl<'w, 's, 'a, 'l, 'll, 'r> Interpreter<'w, 's, 'a, 'l, 'll, 'r, ()> {
    pub fn new<D: ParseDsl>(
        builder: &'a mut Commands<'w, 's>,
        load_ctx: Option<&'l LoadContext<'ll>>,
        registry: &'r TypeRegistry,
    ) -> Interpreter<'w, 's, 'a, 'l, 'll, 'r, D> {
        Interpreter(
            RefCell::new(InnerInterpreter {
                cmds: BevyCmds(builder),
                current: SmallVec::new(),
                dsl: D::default(),
            }),
            LoadCtx(load_ctx, registry),
        )
    }
}
impl<'w, 's, 'a, 'l, 'll, 'r, D: ParseDsl> Interpreter<'w, 's, 'a, 'l, 'll, 'r, D> {
    fn method(&self, (method, mut args): (&[u8], &[u8])) {
        if args.first() == Some(&b'(') {
            args = &args[1..args.len() - 1];
        }
        let name = String::from_utf8_lossy(method);
        let args = String::from_utf8_lossy(args);
        trace!("Method: {name} '{args}'");
        let ctx = MethodCtx { name, args, ctx: self.1 .0, registry: self.1 .1 };
        let dsl = &mut self.0.borrow_mut().dsl;
        dsl.method(ctx)
            .expect("TODO(err): Handle user parsing errors");
    }
    fn statement_spawn(&self) -> Entity {
        trace!("Spawning an entity with provided methods!");
        let interp = &mut *self.0.borrow_mut();

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
        self.0.borrow_mut().current.push(current);
    }
    fn pop_children(&self) {
        trace!("<<< Ended spawning entities within statements block, continuing");
        self.0.borrow_mut().current.pop();
    }
    pub fn statements(&self, input: &mut &BStr) -> PResult<()> {
        use winnow::combinator::{alt, delimited, opt, preceded, separated0, success};
        use winnow::{ascii, token};

        let (spc, spc1, opt) = (ascii::multispace0, ascii::multispace1, || opt(b' '));
        // TODO(bug): Escape sequences
        let str_literal = delimited(b'"', token::take_till0(b'"'), b'"');
        let ident = || token::take_while(1.., (<u8 as AsChar>::is_alphanum, b'_'));
        let args = alt((
            preceded(spc1, ident()),
            // TOOD(perf): split this in a sane way, re-parsing might be costly
            preceded(spc, scoped_text.recognize()),
        ));

        let empty = success::<&BStr, &[u8], _>(b"");
        let named: &[u8] = b"named";
        let method = alt((
            str_literal.map(|i| (named, i)),
            (ident(), alt((args, empty))),
        ))
        .map(|i| self.method(i));
        let comma_list = |p| separated0::<&BStr, _, (), _, _, _, _>(p, (b',', spc));
        let methods = delimited((b'(', spc), comma_list(method), (spc, b')'));
        let nest = |i: &mut _| {
            self.push_children();
            let ret = self.statements(i);
            self.pop_children();
            ret
        };
        let terminal = |_| {
            self.statement_spawn();
        };
        let spawn_head = |(head, _, _)| {
            if head != b"spawn" {
                self.method((head, b""));
            }
        };
        let head = (ident(), opt(), methods).map(spawn_head);
        let tail = alt((b';'.map(terminal), delimited(b'{', nest, b'}')));
        let statement = (head, opt(), tail);

        let space_list = |p| separated0::<&BStr, _, (), _, _, _, _>(p, spc);
        let mut statements = delimited(spc, space_list(statement), spc);
        statements.parse_next(input)
    }
}
