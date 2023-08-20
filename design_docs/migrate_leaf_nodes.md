# How to handle leaf node migration?

What did leaf nodes solve?

1. The ability to "short circuit" nested DSLs so that I can apply afterward
   the leaf_node methods. This way it can give full control of the spawned
   components to the method. For example, with `LayoutDsl::ui` I could overwrite
   the color set by `UiDsl`.
2. It made it easier to give semantic meaning to leaf nodes. `spawn_ui` typically
   used not for setting an attribute, but to show we are spawning a particular
   thingy. The fact it accepts an argument as first parameter was also useful
   since we are spawning _specific_ stuff.

I can't store the `UiBundle` from `IntoUiBundle` into the `Layout` struct.
Well actually I can store a closure `Fn(&mut EntityCommands)`.

What are the problems they introduced?

1. The argument being mixed with the other methods is confusing.
2. The fact it doesn't act like other methods is also confusing.

Unconvincing argument: parsing is a bit harder if I need to parse differently
the first method based on the head identifier value. → not valid, as this is
also true of `code`.

Should we replace it with a different syntax?

- The fact it provides semantic meaning to leaf nodes is _very_ useful.

```rust
ui ("Foobar", "Foobar button", margin 10.);
spawn (ui "Foobar", "Foobar button", margin 10.);
spawn ui("Foobar") ("Foobar button", margin 10.);
> (ui "Foobar", "Foobar button", margin 10.);
mod (ui "Foobar", "Foobar button", margin 10.);
use (ui "Foobar", "Foobar button", margin 10.);
@ (ui "Foobar", "Foobar button", margin 10.);
let (ui "Foobar", "Foobar button", margin 10.);
for (ui "Foobar", "Foobar button", margin 10.);
box (ui "Foobar", "Foobar button", margin 10.);
new (ui "Foobar", "Foobar button", margin 10.);
```

Now looking at it, I don't think it's too bad to just set it as the first method.

Maybe use a keyword so that it is highlighted? What about:

- `mod`: "module" ie: a single contained thing that may contain other things.
  Feel like it's weird/bad
- `use`: ie: "The DSL Bundle uses those methods" I think it makes more sense,
  but it's still weird/bad
- `let`: Since it delimits a _statement_, it could make sense. But we aren't
  assinging anything.
- `final`/`do`/`box`: Not usually syntax-highlighted, but since they don't carry
  inherant meaning, they could have been useful (especially box)
- `@`, `>`: A sigil both highlights the position & have no inherant meaning, so
  we can ascribe it any. But people just hate sigils, regardless of the reasoning
  behind them, so I'll avoid them :(
- `new`: not a keyword, but it has as much meaning as `spawn`.
- `default`: not a keyword, but it can be used as a hint that we are creating a
  new `Default::default` for the choosen DSL, and some highlighters pretend it
  is a keyword somehow.
- `entity`: explicits the fact that each statement creates an entity. Though so
  does `new` and `spawn`, and they are shorter.

> I'll keep `spawn`, and move to something else (maybe) later. It's already a
> very breaking change.

I still think `spawn` is superior though.