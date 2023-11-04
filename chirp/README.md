# `cuicui_chirp`

[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)
[![Documentation](https://docs.rs/cuicui_chirp/badge.svg)](https://docs.rs/cuicui_chirp/)

`cuicui_chirp` defines a file format for text-based bevy scene description.

It is used in `cuicui` for UI, but can describe any kind of scene.

It includes:

- A parser for the file format.
- A bevy loader to load those files in bevy, with [`loader::Plugin`].
- A trait ([`ParseDsl`]) to use your own type's methods as chirp methods
- A macro to automatically implement this trait ([`parse_dsl_impl`])

The syntax is very close to that of [`cuicui_dsl`]'s `dsl!` macro,
with [some additions](#import-statements).

## When to use `cuicui_chirp`?

- You want a powerful and extensible scene definition format for bevy
  to replace the innane `cmds.spawn(…).insert(…).with_children(…)` dance.
- You want quick iteration time using hot reloading and useful error messages.
- You want to minimize the amount of rust code you write to manage scenes.
- You want a re-usable scene definition format.

Be aware that `cuicui_chirp`, by its nature, is not a small dependency.
Consider using [`cuicui_dsl`] if dependency size matters to you.

Also, as of `0.10`, `cuicui_chirp` doesn't support WASM for image and font assets.

## How to use `cuicui_chirp`?

### Cargo features

- **`fancy_errors`** (default): Print parsing error messages in a nicely formatted way.
- **`macros`** (default): Define and export the [`parse_dsl_impl`] macro
- **`load_font`** (default): load `Handle<Font>` as method argument
- **`load_image`** (default): load `Handle<Image>` as method argument
- **`more_unsafe`**: Convert some runtime checks into unsafe assumptions.
  In theory, this is sound, but `cuicui_chirp` is not tested enough to my taste
  for making those assumptions by default.

### Usage

`cuicui_chirp` reads files ending with the `.chirp` extension. To load a `.chirp`
file, use `ChirpBundle` as follow:

```rust,noplayground
# #[cfg(feature = "doc_and_test")] mod test {
# use cuicui_chirp::__doc_helpers::*; // ignore this line pls
use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;

fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));
    cmds.spawn(ChirpBundle::from(assets.load("my_scene.chirp")));
}
# }
```

You need however to add the loader pluging ([`loader::Plugin`]) for this to work.
The plugin is parametrized over the DSL type.
The DSL type needs to implement the [`ParseDsl`] trait.

Here is an example using `cuicui_layout_bevy_ui`'s DSL:

```rust,no_run,noplayground
# #[cfg(feature = "doc_and_test")] mod test {
# use cuicui_chirp::__doc_helpers::*; // ignore this line pls
# fn setup() {}
use bevy::prelude::*;
use cuicui_layout_bevy_ui::UiDsl;

fn main() {
    App::new()
      .add_plugins((
        DefaultPlugins,
        cuicui_chirp::loader::Plugin::new::<UiDsl>(),
      ))
      .add_systems(Startup, setup)
      .run();
}
# }
```

> **Documentation**
>
> Methods available in `chirp` files are the methods available in the choosen
> DSL type (in this case, it would be the `UiDsl` methods). Check the documentation
> page for the corresponding type you are using as DSL. All methods that accept
> an `&mut self` are candidate.

### DSL-specific documentation

The identifiers within parenthesis are methods on the [`ParseDsl`].

Since the `chirp` format is a wrapper over a [`ParseDsl`], refer to the methods
on the `ParseDsl` impl you added as `loader::Plugin`.

### Making a `DslBundle` compatible with `cuicui_chirp`

Let's re-use the example in [`cuicui_dsl`] and extend it to work with `cuicui_chirp`.

We had a `MyDsl` that implements `DslBundle`, now we need to also implement [`ParseDsl`]
for it. So that methods are accessible in `ParseDsl`, use the [`parse_dsl_impl`]
attribute macro, and add it to the `impl` block where all the DSL's methods are
defined:

```diff
     font_size: f32,
 }
+#[cuicui_chirp::parse_dsl_impl]
 impl MyDsl {
     pub fn style(&mut self, style: Style) {
         self.style = style;
```

Yep, for the simple case that's it. Just avoid panicking inside methods if you
want to take advantage of hot reloading.

### `.chirp` file format

The basic syntax is similar to the [`cuicui_dsl`] `dsl!` macro.

One major difference is that `code` blocks are replaced with a function registry.
You can register a function using the [`WorldHandles`] resource. Registered
functions are global to all chirp files loaded using `cuicui_chirp`.

The other differences are the addition of import statements (`use`),
template definitions (`fn`), and template calls (`template!()`).

#### Import statements

They are currently not implemented, so please proceed to the next section.

<details><summary><b>Draft design</b></summary>

> **Note**
> Imports ARE NOT IMPLEMENTED

In `cuicui_chirp` you are not limited to a single file. You can _import_ other
chirp files.

To do so, use an import statement. Import statements **are the first statements
in the file**; They start with the `use` keyword, are followed by the source
path of the file to import and an optional "`as` imported_name", this is the
name with which the import will be refered in this file.

```ron
use different/file
// ...
```

You have two ways to use imports:

1. As **whole file imports**. You can import any file and directly use it as
   if it was a template without parameters. This is useful if you want to compose
   several complex menus you write in different files.
2. As **template collections**. You can import individual templates defined in
   a file. Just complete the path with a `.template_name`.

Similarly to rust, you can combine imports, but only templates from the same file,
so the following is valid:

```ron
use different/file.template
use different/file.{template1, template2}
// ...
```

Wild card imports are not supported.

#### Publicity

However, to be able to import templates, you need to mark them as `pub` in the
source template. Just prefix the `fn` with `pub` and that's it.

</details>

#### Template definitions

chirp files admit a series of `fn` definitions at the very beginning of the
file. A `fn` definition looks very similar to rust function definitions.
It has a name and zero or several parameters. Their body is a single statement:

```rust,ignore,noplayground
// file: <scene.chirp>
// template name
//   ↓
// vvvvvv
fn spacer() {
	Spacer(height(10px) width(10%) bg(coral))
}
//             parameter
// template name  ↓
//    ↓           ↓
// vvvvvv vvvvvvvvvvv
fn button(button_text) {
    Entity(named(button_text) width(95%) height(200px) bg(purple) row) {
        ButtonText(text(button_text) rules(0.5*, 0.5*))
    }
}
```

You can call a template like you would call a rust macro, by writing the template
name followed by `!` and parenthesis:

```rust,ignore,noplayground
# fn sys(cmds: &mut cuicui_dsl::EntityCommands) { cuicui_dsl::dsl!(cmds,
// file: <scene.chirp> (following)
Menu(screen_root row bg(darkgrey)) {
    TestSpacer(width(30%) height(100px) bg(pink))
    spacer!()
    button!("Hello world")
}
# )}
```

When a template is called, it will be replaced by the single root statement
defined as body of the `fn` definition for that template.

#### Template Extras

Template calls can be followed by **template extras**.

```rust,ignore,noplayground
# fn sys(cmds: &mut cuicui_dsl::EntityCommands) { cuicui_dsl::dsl!(cmds,
// file: <scene.chirp> (following)
Menu(screen_root row bg(darkgrey)) {
    TestSpacer(width(30%) height(100px) bg(pink))

    // Additional method list after the template arguments list
    //       vvvvvvvvvvvvvvvvvvvvvv
    spacer!()(width(50%) bg(coral))

    // Both additional methods and additional children added after the argument list
    //                    vvvvvvvvvv
    button!("Hello world")(column) {
        MoreChildren(text("Hello"))
        MoreChildren(text("World"))
    }
}
# )}
```

The additional methods will be added at the end of template's root statement
method list. While the additional children statements will be added as children
of the template's root statement.

Take for example this chirp file:

```rust,ignore,noplayground
fn deep_trailing2(line, color) {
    Trailing2Parent {
        Trailing2 (text(line) bg(color) width(1*))
    }
}
fn deep_trailing1(line2, line1) {
    deep_trailing2!(line1, red) {
        Trailing1 (text(line2) bg(green) width(2*))
    }
}
deep_trailing1!("Second line", "First line") (column bg(beige) rules(1.1*, 2*) margin(20)) {
    Trailing0 (text("Third line") bg(blue))
}
```

It is equivalent to:

```rust,ignore,noplayground
# fn sys(cmds: &mut cuicui_dsl::EntityCommands) { cuicui_dsl::dsl!(cmds,
Trailing2Parent(column bg(beige) rules(1.1*, 2*) margin(20)) {
    Trailing2 (text("First line") bg(red) width(1*))
    Trailing1 (text("Second line") bg(green) width(2*))
    Trailing0 (text("Third line") bg(blue))
}
# )}
```

#### Parameter substitution

> **Note**
> "argument" here may refer to two things: (1) the value passed as
> argument to a template, in `template!(foo_bar)`, `foo_bar` is an argument.
> (2) arguments passed to _methods_, in `Entity(text(method_argument))`,
> `method_argument` is a method argument.
>
> The name declared between parenthesis in the `fn` name is a **parameter**. In
> `fn button(button_text)`, `button_text` is a template parameter.

When a template is called, the body of the `fn` is inserted where the call
is made, arguments passed to the template are inlined within the statement
of the template body.

Please pay close attention to how parameters are inlined:

- Parameters are only inlined **in method arguments**
- Parameters are **not inlined within quotes**
- Parameters are only inlined **if they are the whole argument**

|                             **❗ Compatibility Notice ❗**                          |
|-------------------------------------------------------------------------------------|
| In the future, parameters will be allowed in more contexts: <ul><li>in method lists (such as `Entity(parameter)`)</li><li>As template names (such as `parameter!()`)</li><li>Embedded in a more complex method argument (such as `Entity(mehod({ width: parameter }))`)</li></ul> |
| To avoid painfull breaking changes, avoid naming parameters the same as DSL methods or templates. |

```rust,ignore,noplayground
# fn sys(cmds: &mut EntityCommands) { dsl!(cmds,
fn button(button_text) {
    // Will spawn an entity without name, with tooltip set to whatever
    // was passed to `button!`.
    Entity(tooltip(button_text) width(95%) height(200px) bg(purple) row) {
        // Will spawn an entity named "button_text" with text "button_text"
        button_text(text("button_text") rules(0.5*, 0.5*))

        // Current limitation:
        // `gizmo` method will be called with `GizmoBuilder(button_text)` as first
        // argument and whatever was passed to `button!` as second argument
        Gizmo(gizmo(GizmoBuilder(button_text), button_text) rules(0.5*, 0.5*))
    }
}
# )}
```

### Tips and tricks

See the [dedicated documentation page][`parse_dsl_impl`] for all available
configuration options on `parse_dsl_impl`.

#### Performance

Consider explicitly depending on the `log` and `tracing` crates, and enable the
`"release_max_level_debug"` features of those crates, so that log messages are
elided from release builds.

`cuicui_chirp` contains **a lot** of trace logging in very hot loops.
`"release_max_level_debug"` will remove trace logging **at compile time** and
not only make the code faster (code that would otherwise read a lock atomic),
but it enables **more optimization**, inlining and loop unrolling.

First, find the version of the `log` and `tracing` crates in your dependency tree with:

```sh
cargo tree -p log -p tracing
```

Then, add them to your `Cargo.toml` and enable a `max_level` feature. Note
that they are already in your dependency tree, so there is no downside to doing so:

```toml
log = { version = "<version found with `cargo tree`>", features = ["release_max_level_debug"] }
tracing = { version = "<version found with `cargo tree`>", features = ["release_max_level_debug"] }
# Note: I recommend `release_max_level_warn` instead.
# `debug` is specific for performance regarding `cuicui_chirp`
```

Next time you compile your game, you'll probably have to recompile the whole
dependency tree, since `tracing` and `log` are usually fairly deep.


#### Inheritance

Remember the inheritance trick from [`cuicui_dsl`][dsl-inheritance]? `parse_dsl_impl` is
compatible with it. Use the `delegate` argument to specify the field to which
to delegate methods not found on the `MyDsl` impl.

```rust,ignore
// pub struct MyDsl<D = ()> {
//     #[deref]
//     inner: D,
// }
#[parse_dsl_impl(delegate = inner)]
impl<D: DslBundle> MyDsl<D> {
    // ...
}
```

See [`parse_dsl_impl::delegate`].

#### `ReflectDsl`

Unlike `cuicui_dsl`, it is possible to use [`Reflect`] to define DSLs. See the
[`ReflectDsl`] docs for details.

#### Custom parsers

Since `.chirp` files are in text format, we need to convert text into method
arguments. `parse_dsl_impl` parses differently method arguments depending on
their type.

See [`parse_dsl_impl::type_parsers`] for details.

## What is the relationship between `cuicui_dsl` and `cuicui_chirp`?

`cuicui_dsl` is a macro (`dsl!`), while `cuicui_chirp` is a scene file format,
parser and bevy loader. `cuicui_chirp` builds on top of `cuicui_dsl`, and has
different features than `cuicui_dsl`. Here is a feature matrix:

|features|`cuicui_dsl`|`cuicui_chirp`|
|--------|------------|--------------|
|statements & methods                | ✅ | ✅ |
|`code` blocks with in-line rust code| ✅ |    |
|`code` calling registered functions |    | ✅ |
|`fn` templates                      |rust| ✅ |
|import from other files             |rust|    |
|hot-reloading                       |    | ✅ |
|reflection-based methods            |    | ✅ |
|special syntax for colors, rules    |    | ✅ |
|lightweight                         | ✅ |    |
|Allows for non-`Reflect` components | ✅ |    |

You may use `cuicui_dsl` in combination with `cuicui_chirp`, both crates fill
different niches.

[`cuicui_dsl`]: https://lib.rs/crates/cuicui_dsl
[dsl-inheritance]: https://lib.rs/crates/cuicui_dsl#inheritance
[`loader::Plugin`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/loader/struct.Plugin.html
[`parse_dsl::args::Arguments`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl/args/struct.Arguments.html
[`parse_dsl::args::from_reflect`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl/args/fn.from_reflect.html
[`parse_dsl::args`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl/args/index.html
[`parse_dsl::args::quoted`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl/args/fn.quoted.html
[`parse_dsl::args::to_handle`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl/args/fn.to_handle.html
[`parse_dsl_impl`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl_impl/index.html
[`parse_dsl_impl::delegate`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl_impl/fn.delegate.html
[`parse_dsl_impl::type_parsers`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl_impl/fn.type_parsers.html
[`ParseDsl`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/parse_dsl/trait.ParseDsl.html
[`ReflectDsl`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/reflect/struct.ReflectDsl.html
[`Reflect`]: https://docs.rs/bevy/0.12/bevy/reflect/trait.Reflect.html
[`WorldHandles`]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/loader/struct.WorldHandles.html
