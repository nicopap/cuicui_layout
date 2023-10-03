# `cuicui_dsl`

`cuicui_dsl` is a crate exposing a single trait ([`DslBundle`]) and
a single macro ([`dsl!`]) to define bevy scenes within rust code.

It is used in `cuicui` for UI, but can be used for any kind of scene.

## When to use `cuicui_dsl`?

- You want an _extremely lightweight_ yet powerful scene definition DSL in bevy
  to replace the innane `cmds.spawn(…).insert(…).with_children(…)` dance.
- You don't care about having to re-compile the whole game each time you change
  your scene.

## How to use `cuicui_dsl`?

1. Define a type that implements `DslBundle`
2. Define methods with a `&mut self` receiver on this type
3. Use the methods of the type in question in the `dsl!` macro

```rust
# use cuicui_dsl::macros::__doc_helpers::*; // ignore this line pls
# use std::borrow::Cow;
use cuicui_dsl::{dsl, DslBundle, EntityCommands};

// DslBundle requires Default impl
#[derive(Default)]
pub struct MyDsl {
    style: Style,
    bg_color: Color,
    font_size: f32,
    inner: BaseDsl,
}
impl MyDsl {
    pub fn named(&mut self, name: impl Into<Cow<'static, str>>) {
        self.inner.named(name);
    }
    pub fn style(&mut self, style: Style) {
        self.style = style;
    }
    pub fn bg_color(&mut self, bg_color: Color) {
        self.bg_color = bg_color;
    }
    pub fn font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }
}
impl DslBundle for MyDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        cmds.insert(self.style.clone());
        cmds.insert(BackgroundColor(self.bg_color));
        self.inner.insert(cmds);
        // ...
        cmds.id()
    }
}
// Now you can use `MyDsl` in a `dsl!` macro
fn setup(mut cmds: Commands) {
    let height = px(32);
    dsl! {
        <MyDsl>
        &mut cmds.spawn_empty(),
        // The uppercase name at the start of a statement is the entity name.
        Root(style(Style { flex_direction: FlexDirection::Column, ..default()}) bg_color(Color::WHITE)) {
            Menu(style(Style { height, ..default()}) bg_color(Color::RED))
            Menu(style(Style { height, ..default()}) bg_color(Color::GREEN))
            Menu(style(Style { height, ..default()}) bg_color(Color::BLUE))
        }
    };
}
```

This seems a bit verbose, that's because you should be using [`cuicui_layout`] and
not bevy's native layouting algorithm (flexbox) for layouting :)

The [docs.rs page][`dsl!`] already has extensive documentation on the [`dsl!`] macro,
**with a lot of examples**.

* <https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html>

The short of it is:

`dsl!` accepts three arguments:

1. (optional) the `DslBundle` type you want to use as "builder" for the DSL.
2. The `&mut EntityCommands` to spawn the scene into.
3. A single statement

What is a statement? A statement is:

- An `EntityName` (which is a single identifier) followed by either:
   - several methods within `(parenthesis)`
   - several children statements within `{curly braces}`
   - both of the above

A statement creates a `Default::default()` of the choosen `DslBundle` type.
Then, each mehtod within parenthesis is called on the choosen `DslBundle` type.
Finally, an entity is spawned using the `DslBundle::insert` method on the
thus-constructed `DslBundle`.
The spawned entity has the `Name` component set to the identifier provided for `EntityName`.

Children are added to that entity if child statements are specified within
braces.

Still confused about it? I encourage you to either look at the [examples]
or check the docs at:

* <https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html>


### DSL-specific documentation

Since `dsl!` is just a wrapper around method calls, you can refer to the `docs.rs`
page for the [`DslBundle`] implementation you chose to use in your `dsl!`.

### Tips and tricks

#### Behind the veil

The `dsl!` macro is basically a way to translate an imperative sequential API
into a declarative functional API.

When you write:

```rust
# use cuicui_dsl::macros::__doc_helpers::*; // ignore this line pls
use cuicui_dsl::dsl;
# fn sys(mut cmds: EntityCommands) {
dsl! {
    <BlinkDsl>
    &mut cmds,
    Root {
        FastBlinker(frequency(0.5))
        SlowBlinker(amplitude(2.) frequency(3.0))
    }
}
# }
```
The [`dsl!`] macro translates it into:
```rust
# use cuicui_dsl::macros::__doc_helpers::*; // ignore this line pls
# fn sys(mut cmds: EntityCommands) {
let mut root = BlinkDsl::default();
root.named("Root");
root.node(&mut cmds, |cmds| {
    let mut fast_blinker = BlinkDsl::default();
    fast_blinker.named("FastBlinker");
    fast_blinker.frequency(0.5);
    fast_blinker.insert(&mut cmds.spawn_empty());

    let mut slow_blinker = BlinkDsl::default();
    slow_blinker.named("SlowBlinker");
    slow_blinker.amplitude(2.);
    slow_blinker.frequency(3.0);
    slow_blinker.insert(&mut cmds.spawn_empty());
});
# }
```

The [`DslBundle::insert`] impl of `BlinkDsl` takes care of converting itself
into a set of components it will insert on an entity.

See the [`dsl!`] documentation for more details and examples.


#### Inheritance

The `cuicui` crates _compose_ different `DslBundle`s with a very filthy trick.

Using `DerefMut`, you can get both the methods of your custom `DslBundle` and
the methods of another `DslBundle` embedded into your custom `DslBundle`
(and this works recursively).

Use the bevy `Deref` and `DerefMut` derive macros to accomplish this:

```rust
# use cuicui_dsl::macros::__doc_helpers::*; // ignore this line pls
use cuicui_dsl::DslBundle;

// `= ()` means that if not specified, there is no inner DslBundle
#[derive(Default, Deref, DerefMut)]
pub struct MyDsl<D = ()> {
    #[deref]
    inner: D,
    style: Style,
    bg_color: Color,
    font_size: f32,
}
impl<D: DslBundle> DslBundle for MyDsl<D> {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        cmds.insert(self.style.clone());
        // ... other components to insert ...
        // Always call the inner type at the end so that insertion order follows
        // the type declaration order.
        self.inner.insert(cmds)
    }
}
// Both the methods defined on `MyDsl`
// and the provided `D` are available in the `dsl!` macro for `<MyDsl<D>>`
```

#### Performance

The downside of the aforementioned trick is the size of your `DslBundle`s.
Very large `DslBundle`s tend to generate a lot of machine code just to move them
in and out of functions.

Try keeping the size of your `DslBundle`s down using `bitsets` crates such as
[`enumset`] or [`bitflags`] instead of `bool` fields.

Consider also `Box`ing some large components such as `Style` to avoid the cost of
moving them.

#### Storing a dynamic set of bundles in your `DslBundle`

If you are a lazy butt like me, you don't need to add a field per bundles/component
managed by your `DslBundle`, you can store a `Vec` of bundle spawners as follow:

```rust
# use cuicui_dsl::macros::__doc_helpers::*; // ignore this line pls
use cuicui_dsl::{EntityCommands, DslBundle};

#[derive(Default)]
pub struct MyDynamicDsl(Vec<Box<dyn FnOnce(&mut EntityCommands)>>);

impl MyDynamicDsl {
    pub fn named(&mut self, name: &str) {
        let name = name.to_string();
        self.0.push(Box::new(move |cmds| {cmds.insert(Name::new(name));}));
    }
    pub fn transform(&mut self, transform: Transform) {
        self.0.push(Box::new(move |cmds| {cmds.insert(transform);}));
    }
    pub fn style(&mut self, style: Style) {
        self.0.push(Box::new(move |cmds| {cmds.insert(style);}));
    }
    // ... Hopefully you get the idea ...
}
impl DslBundle for MyDynamicDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        for spawn in self.0.drain(..) {
            spawn(cmds);
        }
        cmds.id()
    }
}
```

## What is the relationship between `cuicui_dsl` and `cuicui_chirp`?

`cuicui_dsl` is a macro (`dsl!`), while `cuicui_chirp` is a scene file format,
parser and bevy loader. `cuicui_chirp` builds on top of `cuicui_dsl`, and has
different features than `cuicui_dsl`. Here is a feature matrix:

|features|`cuicui_dsl`|`cuicui_chirp`|
|--------|------------|--------------|
|statements & methods                |   ✅   | ✅ |
|`code` blocks with in-line rust code|   ✅   |    |
|`code` calling registered functions |        | ✅ |
|`fn` templates                      |rust[^1]| ✅ |
|import from other files             |rust[^2]|    |
|hot-reloading                       |        | ✅ |
|reflection-based methods            |        | ✅ |
|special syntax for colors, rules    |        | ✅ |
|lightweight                         |   ✅   |    |

You may use `cuicui_dsl` in combination with `cuicui_chirp`, both crates fill
different niches.

[^1]: A `fn` template is equivalent to defining a function that accepts an
[`EntityCommands`] and directly calls `dsl!` with it
\
```rust
# use cuicui_dsl::macros::__doc_helpers::*; // ignore this line pls
use cuicui_dsl::{dsl, EntityCommands};

fn rust_template(cmds: &mut EntityCommands, serv: &AssetServer) {
  dsl! {
    cmds,
    Root(screen_root column) {
      Menu(image(&serv.load("menu1.png")))
      Menu(image(&serv.load("menu2.png")))
    }
  }
}
```

[^2]: You can — of course — import functions from other files in rust and use
that instead.

[`bitflags`]: https://docs.rs/bitflags/latest/bitflags/
[`cuicui_layout`]: https://docs.rs/crate/cuicui_layout/0.9.0
[`dsl!`]: https://docs.rs/cuicui_dsl/0.9.0/cuicui_dsl/macro.dsl.html
[`DslBundle`]: https://docs.rs/cuicui_dsl/0.9.0/cuicui_dsl/trait.DslBundle.html
[`EntityCommands`]: https://docs.rs/bevy/0.11/bevy/ecs/system/struct.EntityCommands.html
[`enumset`]: https://docs.rs/enumset/latest/enumset/
[examples]: https://github.com/nicopap/cuicui_layout/tree/cuicui_dsl-v0.9.0/examples
[`DslBundle::insert`]: https://docs.rs/cuicui_dsl/0.9.0/cuicui_dsl/trait.DslBundle.html#tymethod.insert
