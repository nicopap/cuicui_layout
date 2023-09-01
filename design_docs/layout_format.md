# `cuicui_layout` File format

## Requirements

* Hot-reloadable (probs using `bevy-scene-hook`)
* Looks like the `dsl!` macro (if not identical)
* Allow calling rust code through a callback system (store a `UiRegistry`)
* Allows for same extensibility as the base `dsl!` macro.
* Is not a burden to extend your own impls to support the file format
* Only depends on `cuicui_dsl`.

```rust
type UiRegistry = HashMap<
  String,
  Box<dyn FnMut(&mut EntityCommands) -> Entity + Send + Sync + 'static>,
>;
```

## Grammar

```ungrammar
Method
   = StringLiteral            // name literal
   | 'ident'                  // bare method
   | 'ident' 'expr'           // single argument method
   | 'ident' '(' ExprList ')' // multiple argument method

StatementHead
   = 'code'  '(' 'ident' ')'
   | 'spawn' '(' MethodList ')'
   | 'ident' '(' MethodList ')'

Statement     = StatementHead statementTail
StatementTail = ';' | '{' Statement* '}'
MethodList    = Method (',' Method)* ','?
ExprList      = 'expr' (',' 'expr')* ','?
StringLiteral = 'string'
```

Quirks:
* `'ident'` (identifiers) are any sequence of characters
  that is not whitespaces or any of `;",()\{}`.
  Note that they should be rust identifiers
  if `ParseDsl` is implemented using `parse_dsl_impl`.
* Only accepts 1 or 0 space between `'ident'` and `(`
* Only accepts 1 or 0 space between `)` and `{`
* Only accepts 1 or 0 space between `)` and `;`
* `'string'`s accept only quote and backslash escaping
* chirp expressions (`'expr'`) are very naive. They act like the `exposed`
  node in [`cuicui_richtext`'s grammar][richgram]

[richgram]: https://github.com/nicopap/cuicui/blob/12ad8f1fb36e13ee389ac162c61d1638e45503c1/design_doc/richtext/informal_grammar.md#L4

### Why not KDL

This looks very similar to KDL, so why not directly use KDL?

1. Methods may have 0, 1, N arguments, this is not well encoded in KDL
2. "bare methods" would need to be quoted
3. All KDL libraries require a two step processes: (1) creating the KDL node
   tree (document). And (2) reading it. With a custom parser, we can directly
   interpret the format, zero (additional) allocation necessary.
4. We can assume ASCII and get much better perfs.
5. We can re-use the `dsl!` syntax (well, we could use a proc macro)
6. name literals work
7. I already have a grammar and I've enough experience with winnow to know I can
   complete it.
8. It turned out that implementing a custom parser is less code than exploring
   the KDL document AST.

### Why KDL

1. I don't have to write a parser, I can focus on the interesting bits.
2. People might already be familiar with KDL
3. I can piggyback on KDL's error reporting.
4. The `kdl` crate is ready for two-way editing.

**==> I'll start with KDL and switch afterward**

## Implementing `ParseDsl` on existing `DslBundle`s

The most difficult aspect is `FromStr` for enums, but seems we could use
`parse-display` crate?

`parse-display` seems like a large crate and it might add a bit of compile time.

### How to handle `spawn_ui` & generic methods

Leaf nodes are problematic. They have a different way to interpret the
method list (the first is passed as argument) and are called differently.
They also require access to an `EntityCommands`.

Methods with generic arguments are difficult to handle, because we take
advantage of the `FromStr` trait to implicitly parse the `str` we get from
parsing the file format.

### How to handle references & function handles

The `AssetLoader` lives separately from the call site, and there is no way to
associate metadata with specific loaded files.

I think it's worthwhile to write a wrapper around `AssetLoader` that updates
some shared global state. Something like:

```rust
#[derive(SystemParam)]
struct CuicuiLoader {
  loader: Res<'w, AssetServer>,
  loader_request: Res<'w, CuicuiLoaderRequest>,
}
#[derive(Resource)]
struct CuicuiLoaderRequest {
  channel: Sender<LoaderConfigRequest>,
}
```

So we need to associate the load request with the specific `load` invocation:
* May have several loads per systems
* System may run several times

huuuee, this is too complex! For now I'll content my self with a global registry.

### Handle `Handle`s

It's a bit easier to figure out `Handle`:

- `LoadContext::get_hanlde(&self)` (non-mutable!) is a thing
- We only need an asset path to refer to handles (string literal)

However, the problem lies in going from `&str` (or `&[u8]`) to a `Handle<T>`.
Currently we use `FromStr` which can't take a `LoadContext`.

Ideally, we could support a `Reflect` deserialization variant as well.

Options:

- A trait similar to `FromStr`, but also accepts a `LoadContext`, blanket-impl
  the trait on `FromStr`.
- Wrapper types (with pub fields and `Deref` + `DerefMut`) to let users control
  how their method arguments get interpreted
  --> Downside is: require changing type of the method to make it work. Which
  is really bad for the `dsl!` macro ergonomics I don't like it.
- Add an attribute to control how to deserialize specific arguments (takes a
  function to call as argument)
- Why not both put together?

```rust
#[parse_dsl_impl(
  delegate = inner,
  set_params <D: ParseDsl>,
  type_parsers(Rule = from_str),
)]
impl<D> LayoutDsl<D> {
  // `from_reflect` and `from_str` could be just functions. Imported in the macro,
  // and user can provide their own.
  // Another option is to accept an expression instead and it would be possible
  // to define a closure in-line!
  #[parse_dsl(args(height = from_str, width = from_str))]
  fn some_method(&mut self, offset: u16, name: &'a str, height: Rule, width: Rule) {
    todo!();
  }
  #[parse_dsl(args(Rule = from_str))]
  fn some_method(&mut self, offset: u16, name: &'a str, height: Rule, width: Rule) {
    todo!();
  }
  #[parse_dsl(arg(height = from_str))]
  #[parse_dsl(arg(width = from_str))]
  fn some_method(&mut self, offset: u16, name: &'a str, height: Rule, width: Rule) {
    todo!();
  }
  fn some_method(
    &mut self,
    offset: u16,
    name: &'a str,
    #[parse_dsl(from_str)]
    height: Rule,
    #[parse_dsl(from_str)]
    width: Rule,
  ) {
    todo!();
  }
  fn some_method(
    &mut self,
    offset: u16,
    name: &'a str,
    #[from_str]
    height: Rule,
    #[from_str]
    width: Rule,
  ) {
    todo!();
  }
  fn some_method(
    &mut self,
    offset: FromStrArg<u16>,
    name: &'a str,
    rule: FromReflectArg<Rule>,
    image: Handle<Image>
  ) {
    // gawd that's horrible
    todo!();
  }
}
```

Considered designs:

- Inline the attribute to mark the argument specifically rather than at the item-level.
  This would avoid allocation.
- Inline AND do not nest into a `parse_dsl`. This reduces boilerplate, making the
  required code the minimal possible
- Use the attribute at the item level, nested in a `parse_dsl`, considered names were:
  - `parser`: Bit redundant with `parse_dsl`
  - `arguments`: A bit too long, and either way, we know what `arg` is
  - `arg`: blarg—!
  - `args`
- Use the attribute at the item level, declaring a list of `name = ident` of which parser
  to use for given argument.
- Use the attribute at the item level, having several attributes instead of having
  a list in a single attribute.
- Use the attribute at the item level, but associate a **type** instead of an argument
- Associate a type to a parser at the `impl` block level. Considered names:
  - `method_arguments`: It's not very descript isn't it?
  - `type_parsers`: I like this!
- Use trait reflection, read from the type registry. **Does not work** since it would
  require the `FromStr` bound on every type

# New syntax

[Cart's proposal][cpro] gave me ideas:

* Remove the "leaf node" syntax (using arbitrary methods where `spawn` and `code`
  are valid) It's confusing to have an two ways of doing the same thing.
* In place of this, use the "leaf node" as the entity name
* Remove the name literal method syntax
* "Reserve" rust keywords for future compatibility.

[cpro]: https://github.com/bevyengine/bevy/discussions/9538

