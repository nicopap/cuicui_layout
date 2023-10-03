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

use super::ast::{self, Statement, Template};
use super::scope::{Arguments, Parameters};
use super::Input;

pub type Span = (u32, u32);
pub type Name<'a> = (&'a [u8], Span);
#[derive(Clone, Copy)]
pub struct FnIndex(usize);

// TODO(clean): There is a bit of duplicate code between ChirpTemplate and ChirpFile
struct ChirpTemplate<'i, 'a> {
    input: Input<'i>,
    ast: &'a ast::ChirpFile,
    parameters: Parameters<'a>,
    parent: Option<&'a ChirpTemplate<'i, 'a>>,
    trailing_methods: &'a [ast::Method],
    trailing_nodes: &'a [ast::Node],
}
impl<'i, 'a> ChirpTemplate<'i, 'a> {
    fn with_parameters(&'a self, parameters: Parameters<'a>, template: &'a Template) -> Self {
        ChirpTemplate {
            input: self.input,
            ast: self.ast,
            parameters,
            trailing_methods: &template.methods,
            trailing_nodes: &template.children,
            parent: Some(self),
        }
    }
    fn interpret_template(&self, tpl: &Template, runner: &mut impl Interpreter<'i>) {
        let inp = &self.input;
        let (mut name, span) = tpl.name.read_spanned(inp);
        name = &name[..name.len() - 1];
        let Some(fn_index) = runner.get_template((name, span)) else {
            return;
        };
        let declr = &self.ast.functions[fn_index.0];
        let parameters = self.parameters.scope(&declr.arguments, &tpl.arguments, inp);
        let inner_chirp = self.with_parameters(parameters, tpl);
        inner_chirp.interpret_root(&declr.body, runner);
    }
    // This function is similar to [`ChirpFile::interpret_statement`] with the
    // difference that it inlines the passed "template extras" to the root expression.
    //
    // To do this is extra tricky, because:
    // 1. "template extras" should be evaulated with the scope of their parent
    // 2. We may inherit template extras from deeper ancestors than the direct parent.
    // 3. And of course, those deeper template extras need to be evaluated with their own
    //    parent.
    fn interpret_statement(&self, st: &Statement, runner: &mut impl Interpreter<'i>) {
        let inp = &self.input;
        if let Some(name) = st.name.get_with_span(inp) {
            runner.set_name(name);
        }
        for ast::Method { name, arguments } in &st.methods {
            let arguments = Arguments::new(*inp, arguments, &self.parameters);
            runner.method(name.read_spanned(inp), &arguments);
        }
        let mut no_children = st.children.is_empty();
        let mut this = self;
        loop {
            for ast::Method { name, arguments } in this.trailing_methods {
                let empty_parameters = Parameters::empty();
                let parameters = this.parent.map_or(&empty_parameters, |p| &p.parameters);
                let arguments = Arguments::new(*inp, arguments, parameters);
                runner.method(name.read_spanned(inp), &arguments);
            }
            no_children &= this.trailing_nodes.is_empty();
            this = match this.parent {
                None => break,
                Some(v) => v,
            };
        }
        if no_children {
            runner.spawn_leaf();
        } else {
            runner.start_children();
            for node in &st.children {
                self.file().interpret_node(node, runner);
            }
            let mut this = self;
            loop {
                for node in this.trailing_nodes {
                    let parent = this
                        .parent
                        .map_or_else(|| ChirpFile::new(self.input, self.ast), Self::file);
                    parent.interpret_node(node, runner);
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
            parameters: self.parameters.clone(),
        }
    }
    fn interpret_root(&self, node: &ast::Node, runner: &mut impl Interpreter<'i>) {
        match node {
            ast::Node::Template(template) => self.interpret_template(template, runner),
            ast::Node::Statement(statement) => self.interpret_statement(statement, runner),
            // TODO(bug): Need to add the template extras here.
            ast::Node::Code(ident) => runner.code(ident.read_spanned(&self.input)),
        }
    }
}
pub struct ChirpFile<'i, 'a> {
    input: Input<'i>,
    ast: &'a ast::ChirpFile,
    parameters: Parameters<'a>,
}
impl<'i, 'a> ChirpFile<'i, 'a> {
    fn with_parameters(
        &'a self,
        parameters: Parameters<'a>,
        template: &'a Template,
    ) -> ChirpTemplate<'i, 'a> {
        ChirpTemplate {
            input: self.input,
            ast: self.ast,
            parameters,
            trailing_methods: &template.methods,
            trailing_nodes: &template.children,
            parent: None,
        }
    }
    pub fn new(input: Input<'i>, ast: &'a ast::ChirpFile) -> Self {
        Self { input, ast, parameters: Parameters::empty() }
    }

    fn interpret_statement(&self, st: &Statement, runner: &mut impl Interpreter<'i>) {
        let inp = &self.input;
        if let Some(name) = st.name.get_with_span(inp) {
            runner.set_name(name);
        }
        for ast::Method { name, arguments } in &st.methods {
            let arguments = Arguments::new(*inp, arguments, &self.parameters);
            runner.method(name.read_spanned(inp), &arguments);
        }
        if st.children.is_empty() {
            runner.spawn_leaf();
        } else {
            runner.start_children();
            for node in &st.children {
                self.interpret_node(node, runner);
            }
            runner.complete_children();
        }
    }
    fn interpret_template(&self, tpl: &Template, runner: &mut impl Interpreter<'i>) {
        let inp = &self.input;
        let (mut name, span) = tpl.name.read_spanned(inp);
        name = &name[..name.len() - 1];
        let Some(fn_index) = runner.get_template((name, span)) else {
            return;
        };
        let declr = &self.ast.functions[fn_index.0];
        let parameters = self.parameters.scope(&declr.arguments, &tpl.arguments, inp);
        let inner_chirp = self.with_parameters(parameters, tpl);
        inner_chirp.interpret_root(&declr.body, runner);
    }
    fn interpret_node(&self, node: &ast::Node, runner: &mut impl Interpreter<'i>) {
        match node {
            ast::Node::Template(template) => self.interpret_template(template, runner),
            ast::Node::Statement(statement) => self.interpret_statement(statement, runner),
            ast::Node::Code(ident) => runner.code(ident.read_spanned(&self.input)),
        }
    }
    pub fn interpret(&self, runner: &mut impl Interpreter<'i>) {
        let inp = &self.input;
        for ast::Import { name, alias } in &self.ast.imports {
            runner.import(name.read_spanned(inp), alias.read_spanned(inp));
        }
        for (i, ast::Function { name, .. }) in self.ast.functions.iter().enumerate() {
            runner.register_fn(name.read_spanned(inp), FnIndex(i));
        }
        self.interpret_node(&self.ast.root, runner);
    }
}
pub trait Interpreter<'i> {
    fn import(&mut self, name: Name<'i>, alias: Option<Name<'i>>);
    fn register_fn(&mut self, name: Name<'i>, index: FnIndex);
    fn get_template(&mut self, name: Name<'i>) -> Option<FnIndex>;
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
impl Interpreter<'_> for () {
    fn code(&mut self, _: Name) {}
    fn import(&mut self, _: Name, _: Option<Name>) {}
    fn register_fn(&mut self, _: Name, _: FnIndex) {}
    fn get_template(&mut self, _: Name) -> Option<FnIndex> {
        None
    }
    fn set_name(&mut self, _: Name) {}
    fn start_children(&mut self) {}
    fn method(&mut self, _: Name, _: &Arguments) {}
    fn complete_children(&mut self) {}
}
