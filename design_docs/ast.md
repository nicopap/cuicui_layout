# Block based AST layout

The AST is a single flat `Vec`, but it does store each AST node as a subslice.

This has a few advantages:

- Cache locality
- Allocation is minimal

Really, what the `Vec` contains is not "nodes" but rather "blocks" which may
be several or only part of a node.

In an AST, a node may contain a variable count of other nodes. To allow that in
our flat list, we need a header block containing:

- What is the node types if the current node may be one of several
- The size of the current node, so that we can compute the next node's header
  position

The header is the first block of the node. Each block is a `u32`.

In our current grammar we have the following AST nodes:

- `Use`: An import statement with an **identifier name**
  and an optional **identifier name** `as` binding
- `Fn`: Function with an **identifier name**, N **identifier name** parameters
  and a single inner `Spawn | Template | Code`
- `Spawn`: A statement with an optional **name**, N methods and N children `Spawn | Template | Code`
- `Template`: A template call with an **identifier name**, N arguments, N methods
  and N children `Spawn | Template | Code`
- `Method`: A method call with an **identifier name** and N argument
- `Argument`: Currently, an offset + length into the input stream.
- `Code`: A `code` **identifier name**

If we make use of an interner, we can compress what we call "identifier names".
The value, instead of being an offset in the input stream, is a small index number.
I don't expect more than a few thousand identifiers, they are only used as method names,
template names, and template arguments.

The lengths stored in headers is not "how many FOO does this contain" but rather
how many blocks the subslice of the relevant data contains. This allows using
`slice::split_at` to isolate the **node**'s subslice, interpret it, and get to
the next node.

## AST Layout

If we commit to a block-based approach we can do the following:

```
struct IdentOffset(u32); // Start offset in input stream of an Ident token.
struct OptIdentOffset(u32); // Optional version of `IdentOffset` where u32::MAX denotes "None"
struct NameOffset(u32); // Start offset in input stream of a (String | Ident) token.
struct OptNameOffset(u32); // Optional version of `NameOffset` where u32::MAX denotes "None"

# Node 1: ChirpFile (~ blocks)

ast_header: {
  import_count: u32,
  root_statement_offset: u32,
}
imports: [Import]
fn_declarations: [Fn]
root_statement: Spawn | Template | Code

# Node 2: Import (2 blocks)

name: IdentOffset
alias: OptIdentOffset

# Node 3: Fn (~ blocks)

// we have a hashmap somewhere that associate a name to an `Fn` index in the AST.

header: {
  parameter_count: u6,
  name: u26 as IdentOffset,
}
parameters: [IdentOffset]
body: Spawn | Template | Code

total_length:
  1 + header.argument_count * size_of::<NameOffset>
  + total_length(body[0])

# Node 4: Argument (2 blocks)

// the start and end of a `many_tts` syntax element

start: u32
end: u32

# Node 5: Method (~ blocks)

header: {
  argument_count: u6,
  name: u26 as IdentOffset,
}
arguments: [Argument]

total_length: 1 + header.argument_count * size_of::<Argument>

# Node 6: Spawn (~ blocks)

header: {
  discriminant: u4,
  name: u28 as OptNameOffset,
}
methods_len: u32
children_len: u32
methods: [Method]
children: [Spawn | Template | Code]

total_length: 3 + methods_blocks + children_blocks

# Node 7: Template (~ blocks)

header: {
  discriminant: u4,
  name: u28 as IdentOffset,
}
template_header: {
  argument_count: u6,
  methods_len: u26,
}
children_len: u32
arguments: [Argument]
methods: [Method]
children: [Spawn | Template | Code]

total_length:
  3 + template_header.argument_count * size_of::<Argument>
  + template_header.methods_blocks
  + children_blocks

# Node 8: Code (1 block)

header: {
  discriminant: u4,
  name: u28 as IdentOffset,
}
```

The way we access AST nodes is through view structs as follow:

```rust
pub struct FnIndex(u32);
pub struct Fn<'a>(*const FnHeader, PhantomData<&'a ()>);

pub struct Imports<'a>(&'a[Import]);

pub struct Statement<'a>(*const StatementHeader, PhantomData<&'a ()>);
pub struct Template<'a>(*const TemplateHeader, PhantomData<&'a ()>);

pub enum Node<'a> {
  Template(Template<'a>),
  Statement(Statement<'a>),
}

pub struct AstBuffer<'a>(&'a[u32]);

impl<'a> AstBuffer<'a> {
  pub fn get_fn(&self, fn_index: FnIndex) -> Fn<'a> {
    Fn(unsafe { self.0.as_ptr().offset(isize::from(fn_index.0)) }, PhantomData)
  }
  pub fn imports(&self) -> Imports<'a> {
    let len = self.0[0] as usize;
    let imports_slice = &self.0[2..2+(len*size_of::<Import>() / size_of::<u32>())];
    let imports_ptr = imports_slice.as_ptr();
    Imports(unsafe { slice::from_raw_parts(imports_ptr, len) })
  }
  pub fn root_node(&self) -> Node<'a> {
    // etc.
  }
}
```

Then, we would implement methods on `Template`, `Statement`, `Method`

### Pitfalls

So we may in the future support imports, which would require cross-referencing
data coming from different `ChirpFile` ASTs.

When having several `ChirpFile`, we would ideally be able to sample from nodes
of separate files without caring which file they are part of. At least "sampled"
nodes should be self-contained: ie: shouldn't depend on its immediate context.

Nodes with this architecture are (almost) self-contained. You don't need
additional context to interpret the node value. Well, almost.

1. In the context of a function, we need to re-interpret `Argument` values
   if they contain one or several of the function parameter names.
2. When encountering a template call, the template name should be taken from
   the context.
3. A single chirp file is associated with a single input stream, we need to
   keep the association between the file and the input stream, otherwise name
   offsets and argument offsets do not make any sense.

For (2), the solution is to replace template names by an index. It is probably
going to be `enum TemplateRef{Import(u32), Fn(u32)}`.

For (1), it's very tricky. Because we are facing a _variable_ amount of parameter
references within a single argument.

(3) I'm not even sure how to approach.

I think the way I'll handle it is by creating a second AST for exported functions,
and just call "insert chirp file" (with the `Handle<Chirp>`) when encoutering
a whole-file template.

## Alternative designs

### 48 bits nodes

With 48 bits per nodes, we'd be able to store two offsets in a single block with
24 bits per offset. (considering a maximum file size of 16MB).

This would reduce the size of our AST by 25%, but at the cost of introducing a
lot of bit twiddling.

### Specialized node buffers

This blog describes splitting AST nodes into several buffers: <https://alic.dev/blog/dense-enums>

I considered this, but it introduces a number of issues, and doesn't solve some pressing problems:

- Instead of storing a node count, you need to store offset in specialized buffer + count
- No more cache locality
