impl AstNode {
    pub const fn argument(tto: TokenTreeOffset) -> Self {
        AstNode::Argument(Argument(tto))
    }
    fn add_methods(&mut self, count: u16) {
        match self {
            AstNode::Template(Template { methods, .. })
            | AstNode::Statement(Statement { methods, .. }) => *methods += count,
            AstNode::Method(_) | AstNode::Argument(_) | AstNode::Code(_) => todo!(),
        }
    }
    fn incr_arguments(&mut self) {
        match self {
            AstNode::Template(Template { arguments, .. }) => *arguments += 1,
            AstNode::Method(Method { arguments, .. }) => *arguments += 1,
            _ => todo!(),
        }
    }
    fn add_statements(&mut self, count: u16) {
        match self {
            AstNode::Template(Template { statements, .. })
            | AstNode::Statement(Statement { statements, .. }) => *statements += count,
            AstNode::Method(_) | AstNode::Argument(_) | AstNode::Code(_) => todo!(),
        }
    }
    const fn node_count(&self) -> usize {
        match self {
            AstNode::Template(Template { arguments, methods, statements, .. }) => {
                1 + *arguments as usize + *methods as usize + *statements as usize
            }
            AstNode::Statement(Statement { methods, statements, .. }) => {
                1 + *methods as usize + *statements as usize
            }
            AstNode::Method(Method { arguments, .. }) => 1 + *arguments as usize,
            AstNode::Argument(_) | AstNode::Code(_) => 1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Ast<'a> {
    pub ast: &'a [AstNode],
}

impl<'a> Ast<'a> {
    fn methods(mut self, interpreter: &mut impl Interpreter) {
        while let Some((node, remaining)) = self.ast.split_first() {
            let node = unsafe { node.assume_method() };
            let (args, remaining) = remaining.split_at(node.arguments as usize);
            interpreter.call_method(node.name, MethodArguments(args));

            self.ast = remaining;
        }
    }
    fn statements(mut self, interpreter: &mut impl Interpreter) {
        while let Some((node, remaining)) = self.ast.split_first() {
            match unsafe { node.assume_statement_or_template() } {
                Ok(statement) => {
                    interpreter.new_statement(statement.name);

                    let (methods, remaining) = remaining.split_at(usize::from(statement.methods));
                    Ast::new(methods).methods(interpreter);

                    let (statements, remaining) =
                        remaining.split_at(usize::from(statement.statements));
                    Ast::new(statements).statements(interpreter);
                    self.ast = remaining;

                    interpreter.complete_statement();
                }
                Err(template) => {
                    let (args, remaining) = remaining.split_at(usize::from(template.arguments));
                    interpreter.call_template(template.name, TemplateArguments(args));

                    let (methods, remaining) = remaining.split_at(usize::from(template.methods));
                    Ast::new(methods).methods(interpreter);

                    let (_st, remaining) = remaining.split_at(usize::from(template.statements));
                    // TODO(feat): support statement template extensions.
                    // Ast::new(statements).template_statements(interpreter);
                    self.ast = remaining;

                    interpreter.complete_template();
                }
            }
        }
    }
}
struct ParentStack(SmallVec<[u16; 8]>);
impl ParentStack {
    fn new() -> Self {
        ParentStack(SmallVec::new())
    }
    fn add_statements(&self, nodes: &mut [AstNode], count: u16) {
        for &parent in &self.0 {
            nodes[parent as usize].add_statements(count);
        }
    }
    fn add_methods(&self, nodes: &mut [AstNode], count: u16) {
        let Some((&last, remaining)) = self.0.split_last() else {
            return;
        };
        nodes[last as usize].add_methods(count);
        for &parent in remaining {
            nodes[parent as usize].add_statements(count);
        }
    }
    fn incr_arguments(&self, nodes: &mut [AstNode]) {
        let Some((&last, remaining)) = self.0.split_last() else {
            return;
        };
        nodes[last as usize].incr_arguments();

        let Some((&last, remaining)) = self.0.split_last() else {
            return;
        };
        nodes[last as usize].add_methods(1);

        for &parent in remaining {
            nodes[parent as usize].add_statements(1);
        }
    }
}
#[derive(Debug)]
pub struct OwnAst {
    ast: Vec<AstNode>,
}
impl OwnAst {
    pub fn new_template(
        name: NameOffset,
        args: Vec<AstNode>,
        methods: Vec<ParsedMethod>,
        statements: Option<OwnAst>,
    ) -> Self {
        let methods_ast: Vec<_> = methods.into_iter().flatten().collect();
        let root_node = AstNode::Template(Template {
            arguments: as_u16(args.len()),
            methods: as_u16(methods_ast.len()),
            statements: as_u16(statements.as_ref().map_or(0, |s| s.ast.len())),
            name,
        });
        let ast = iter::once(root_node)
            .chain(args)
            .chain(methods_ast)
            .chain(statements.into_iter().flat_map(|s| s.ast));
        OwnAst { ast: ast.collect() }
    }
    pub fn new_statement(
        name: OptNameOffset,
        methods: Vec<ParsedMethod>,
        statements: Option<OwnAst>,
    ) -> Self {
        let methods_ast: Vec<_> = methods.into_iter().flatten().collect();
        let root_node = AstNode::Statement(Statement {
            methods: as_u16(methods_ast.len()),
            statements: as_u16(statements.as_ref().map_or(0, |s| s.ast.len())),
            name,
        });
        let ast = iter::once(root_node)
            .chain(methods_ast)
            .chain(statements.into_iter().flat_map(|s| s.ast));
        OwnAst { ast: ast.collect() }
    }
}
#[derive(Clone, Debug, Copy)]
pub struct Argument(pub TokenTreeOffset);

#[derive(Clone, Copy, Debug, Default)]
pub struct Statement {
    /// How many nodes are occupied by the methods.
    pub(super) methods: u16,
    /// How many nodes are occupied by the children statements.
    pub(super) statements: u16,
    /// The name of this statement, if one.
    pub(super) name: OptNameOffset,
}
#[derive(Clone, Debug, Copy)]
pub struct Template {
    /// How many arguments this template invocation has.
    pub(super) arguments: u16,
    /// How many nodes are occupied by the methods.
    pub(super) methods: u16,
    /// How many nodes are occupied by the children statements.
    pub(super) statements: u16,
    /// The name of this template.
    pub(super) name: NameOffset,
}
#[derive(Clone, Debug, Copy)]
pub struct Method {
    /// How many arguments this method invocation has.
    pub(super) arguments: u32,
    pub(super) name: NameOffset,
}

pub struct MethodArguments<'a>(&'a [AstNode]);
