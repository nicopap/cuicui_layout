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

```scala
method
   := '"' [quoted_string] '"' // name literal
    | ident                   // bare method
    | ('.' ident)+ expr       // field access (may be left out)
    | ident expr              // single argument method
    | ident '(' expr (',' expr)* ')' // multiple argument method

statement_head
   := 'spawn' '(' (method),* ')'
    | 'code' '(' ident ')'
    | ident '(' expr (',' method )* ')'

statement_tail:= ';' | '{' statement* '}'
ident         := RUST_IDENTIFIER
expr          := TBD
statement     := statement_head statement_tail
```

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

### How to handle references & function handles & asset `Handle`s

The `AssetLoader` lives separately from the call site, and there is no way to
associate metadata with specific loaded files.

I think it's worthwhile to write a wrapper around `AssetLoader` that updates
some shared global state. Something like:

```rust
#[derive(SystemParam)]
struct CuicuiLoader {
  loader: Res<'w, AssetLoader>,
  loader_request: Res<'w, CuicuiLoaderRequest>,
}
#[derive(Resource)]
struct CuicuiLoaderRequest {
  channel: Sender<LoaderConfigRequest>,
}
```
