//! Interpret the [`ast`] using an [`Interpreter`]
//!
//! # Architecture
//!
//! The AST starts at [`ChirpFile`], the API used outside of this module is:
//!
//! - Create a [`ChirpFile`] using [`ChirpFile::new`]
//! - Run an interpreter through the AST using [`ChirpFile::interpret`]
//!
//! [`Interpreter`] formalizes the operations available on the `ChirpFile` AST,
//! it is basically an implementation of the rust Visitor pattern.
//!
//! ## Templates
//!
//! Since chirp files support templates, we need to be able to call templates.
//! This requires both being able to interpret the nodes of the `fn` statements,
//! but also being able to replace the template parameters by the values provided
//! as argument to the template when it was called.
//!
//! An _extra_ tricky bit are the "template extras". One is able to add children
//! and methods to the statement of a template.
//!
//! It is particularly pernicious when the extras contain template parameters,
//! as it needs to expand according to the calling context's arguments, not
//! the final context.
//!
//! This is why we split [`ChirpTemplate`] off from [`ChirpFile`]. `ChirpTemplate`
//! only works for a single statement, and accmulates to that single statements
//! the "template extras" from the chain of templates that led to the current
//! statement.
//!
//! To do that, we need to insert methods and children at precise locations. So we
//! somehow need to keep track of that chain. We can do that without a single heap
//! allocation. What we do is track the call chain to the current template site
//! through a single `parent: Option<&Self>` field. When spawning the root statement,
//! we walk back the whole call stack (in the actual machine stack) and read all
//! the extras for each stack level. We can read the parent's `Parameters` field
//! to get the correct parameter substitution for that particular extra.
use bevy::log::trace;

use super::ast::{self, AstRef, FnIndex, Template};
use super::scope::{Arguments, Parameters};
use super::Input;

pub type Span = (u32, u32);
pub type Name<'a> = (&'a [u8], Span);

// TODO(clean): There is a bit of duplicate code between ChirpTemplate and ChirpFile
struct ChirpCall<'t, 'i, 'a> {
    input: Input<'i>,
    ast: AstRef<'a>,
    params: Parameters<'a>,
    parent: Option<&'t ChirpCall<'t, 'i, 'a>>,
    trailing_methods: ast::Methods<'a>,
    trailing_children: ast::Statements<'a>,
}
impl<'t, 'i, 'a> ChirpCall<'t, 'i, 'a> {
    fn with_parameters(&'t self, parameters: Parameters<'a>, template: Template<'a>) -> Self {
        ChirpCall {
            input: self.input,
            ast: self.ast,
            params: parameters,
            trailing_methods: template.methods(),
            trailing_children: template.children(),
            parent: Some(self),
        }
    }
    fn interpret_template(&self, tpl: Template<'a>, runner: &mut impl Interpreter<'i, 'a>) {
        let inp = &self.input;
        let (mut name, span) = tpl.name().read_spanned(inp);
        name = &name[..name.len() - 1];
        let Some(fn_index) = runner.get_template((name, span)) else {
            return;
        };
        let declr = fn_index.get();
        let parameters = self.params.scope(declr.parameters(), tpl.arguments(), inp);
        let inner_chirp = self.with_parameters(parameters, tpl);
        inner_chirp.interpret_root(declr.body(), runner);
    }
    // This function is similar to [`ChirpFile::interpret_spawn`] with the
    // difference that it inlines the passed "template extras" to the root expression.
    //
    // To do this is extra tricky, because:
    // 1. "template extras" should be evaulated with the scope of their parent
    // 2. We may inherit template extras from deeper ancestors than the direct parent.
    // 3. And of course, those deeper template extras need to be evaluated with their own
    //    parent.
    fn interpret_spawn(&self, spawn: ast::Spawn<'a>, runner: &mut impl Interpreter<'i, 'a>) {
        trace!("{} - {spawn:?}", spawn.block_index(self.ast));
        let inp = &self.input;
        if let Some(name) = spawn.name().get_with_span(inp) {
            runner.set_name(name);
        }
        for method in spawn.methods().iter() {
            trace!("{} - {method:?}", method.block_index(self.ast));
            let (name, arguments) = (method.name(), method.arguments());
            let arguments = Arguments::new(*inp, arguments, &self.params);
            runner.method(name.read_spanned(inp), &arguments);
        }
        let mut no_children = spawn.children().is_empty();
        let mut this = self;
        loop {
            for method in this.trailing_methods.iter() {
                trace!("{} - {method:?}", method.block_index(self.ast));
                let (name, arguments) = (method.name(), method.arguments());
                let empty_parameters = Parameters::empty();
                let parameters = this.parent.map_or(&empty_parameters, |p| &p.params);
                let arguments = Arguments::new(*inp, arguments, parameters);
                runner.method(name.read_spanned(inp), &arguments);
            }
            no_children &= this.trailing_children.is_empty();
            this = match this.parent {
                None => break,
                Some(v) => v,
            };
        }
        if no_children {
            runner.spawn_leaf();
        } else {
            runner.start_children();
            for statement in spawn.children().iter() {
                self.file().interpret_statement(statement, runner);
            }
            let mut this = self;
            loop {
                for statement in this.trailing_children.iter() {
                    let root_file = || ChirpFile::new(self.input, self.ast);
                    let parent = this.parent.map_or_else(root_file, Self::file);
                    parent.interpret_statement(statement, runner);
                }
                this = match this.parent {
                    None => break,
                    Some(v) => v,
                };
            }
            runner.complete_children();
        }
    }
    fn file(&self) -> ChirpFile<'i, 'a> {
        ChirpFile {
            input: self.input,
            ast: self.ast,
            params: self.params.clone(),
        }
    }
    fn interpret_root(&self, statement: ast::Statement<'a>, runner: &mut impl Interpreter<'i, 'a>) {
        match statement.typed() {
            ast::StType::Template(template) => self.interpret_template(template, runner),
            ast::StType::Spawn(spawn) => self.interpret_spawn(spawn, runner),
            // TODO(bug): Need to add the template extras here.
            ast::StType::Code(code) => runner.code(code.name().read_spanned(&self.input)),
        }
    }
}
pub struct ChirpFile<'i, 'a> {
    input: Input<'i>,
    ast: AstRef<'a>,
    params: Parameters<'a>,
}
impl<'i, 'a> ChirpFile<'i, 'a> {
    fn with_parameters(&self, ps: Parameters<'a>, template: Template<'a>) -> ChirpCall<'_, 'i, 'a> {
        ChirpCall {
            input: self.input,
            ast: self.ast,
            params: ps,
            trailing_methods: template.methods(),
            trailing_children: template.children(),
            parent: None,
        }
    }
    pub fn new(input: Input<'i>, ast: AstRef<'a>) -> Self {
        Self { input, ast, params: Parameters::empty() }
    }

    fn interpret_spawn(&self, spawn: ast::Spawn<'a>, runner: &mut impl Interpreter<'i, 'a>) {
        trace!("{} - {spawn:?}", spawn.block_index(self.ast));
        let inp = &self.input;
        if let Some(name) = spawn.name().get_with_span(inp) {
            runner.set_name(name);
        }
        for method in spawn.methods().iter() {
            trace!("{} - {method:?}", method.block_index(self.ast));
            let (name, arguments) = (method.name(), method.arguments());
            let arguments = Arguments::new(*inp, arguments, &self.params);
            runner.method(name.read_spanned(inp), &arguments);
        }
        if spawn.children().is_empty() {
            runner.spawn_leaf();
        } else {
            runner.start_children();
            for statement in spawn.children().iter() {
                self.interpret_statement(statement, runner);
            }
            runner.complete_children();
        }
    }
    fn interpret_template(&self, tpl: Template<'a>, runner: &mut impl Interpreter<'i, 'a>) {
        trace!("{} - {tpl:?}", tpl.block_index(self.ast));
        let inp = &self.input;
        let (mut name, span) = tpl.name().read_spanned(inp);
        name = &name[..name.len() - 1];
        let Some(fn_index) = runner.get_template((name, span)) else {
            return;
        };
        let declr = fn_index.get();
        let parameters = self.params.scope(declr.parameters(), tpl.arguments(), inp);
        let inner_chirp = self.with_parameters(parameters, tpl);
        inner_chirp.interpret_root(declr.body(), runner);
    }
    fn interpret_statement(&self, st: ast::Statement<'a>, runner: &mut impl Interpreter<'i, 'a>) {
        match st.typed() {
            ast::StType::Template(template) => self.interpret_template(template, runner),
            ast::StType::Spawn(spawn) => self.interpret_spawn(spawn, runner),
            ast::StType::Code(code) => runner.code(code.name().read_spanned(&self.input)),
        }
    }
    pub fn interpret(&self, runner: &mut impl Interpreter<'i, 'a>) {
        let inp = &self.input;
        let file = self.ast.chirp_file();
        trace!("{} - {file:?}", file.block_index(self.ast));
        for import in file.imports().iter() {
            trace!("{} - {import:?}", import.block_index(self.ast));
            let (name, alias) = (import.name(), import.alias());
            runner.import(name.read_spanned(inp), alias.read_spanned(inp));
        }
        for fn_declr in file.fn_declrs().iter() {
            trace!("{} - {fn_declr:?}", fn_declr.block_index(self.ast));
            let index = fn_declr.index();
            runner.register_fn(fn_declr.name().read_spanned(inp), index);
        }
        self.interpret_statement(file.root_statement(), runner);
    }
}
pub trait Interpreter<'i, 'a> {
    fn import(&mut self, name: Name<'i>, alias: Option<Name<'i>>);
    fn register_fn(&mut self, name: Name<'i>, index: FnIndex<'a>);
    fn get_template(&mut self, name: Name<'i>) -> Option<FnIndex<'a>>;
    fn code(&mut self, code: Name<'i>);
    fn spawn_leaf(&mut self) {
        self.start_children();
        self.complete_children();
    }
    fn set_name(&mut self, name: Name);
    fn start_children(&mut self);
    fn complete_children(&mut self);
    fn method(&mut self, name: Name<'i>, arguments: &Arguments);
}
impl<'a> Interpreter<'_, 'a> for () {
    fn code(&mut self, _: Name) {}
    fn import(&mut self, _: Name, _: Option<Name>) {}
    fn register_fn(&mut self, _: Name, _: FnIndex<'a>) {}
    fn get_template(&mut self, _: Name) -> Option<FnIndex<'a>> {
        None
    }
    fn set_name(&mut self, _: Name) {}
    fn start_children(&mut self) {}
    fn method(&mut self, _: Name, _: &Arguments) {}
    fn complete_children(&mut self) {}
}
