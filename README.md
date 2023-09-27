[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![Latest version](https://img.shields.io/crates/v/cuicui_layout.svg)](https://crates.io/crates/cuicui_layout)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](./LICENSE)
[![Documentation](https://docs.rs/cuicui_layout/badge.svg)](https://docs.rs/cuicui_layout/)

# Cuicui Layout

A dumb layout algorithm you can rely on, built for and with bevy.

<details><summary><h2>The Cyberpunk 2077 showcase</h2></summary>

For some reasons, the Cyberpunk main menu has become the 7GUI of bevy, so here
is the Cyberpunk main menu using `cuicui_layout_bevy_ui`.

https://github.com/nicopap/cuicui_layout/assets/26321040/8a51f9a9-ffa7-4b60-a2ad-3947ff718e27.mp4

| **❗ Hot reloading disclaimer ❗** |
|------------------------------------|

Chirp hot reloading with `bevy_ui` components (ie: using `cuicui_layout_bevy_ui`)
is broken due to <https://github.com/bevyengine/bevy/pull/9621>.
You may want to work on a local patched version of bevy.
A workaround will probably be provided in cuicui 0.10.

### Code

```rust
use bevy::prelude::*;
use cuicui_layout_bevy_ui::UiDsl as Dsl;
use cuicui_layout::{LayoutRootCamera, dsl, dsl_functions::{px, pct, child}};

fn setup(mut cmds: Commands, serv: Res<AssetServer>) {

cmds.spawn((Camera2dBundle::default(), LayoutRootCamera));
let menu_buttons = [
    "CONTINUE",
    "NEW GAME",
    "LOAD GAME",
    "SETTINGS",
    "ADDITIONAL CONTENT",
    "CREDITS",
    "QUIT GAME",
];
let title_card = serv.load::<Image, _>("logo.png");
let bg = serv.load("background.png");
let board = serv.load("board.png");
let button = serv.load("button.png");

dsl! {
    &mut cmds.spawn_empty(),
    Root(layout(">dSaS") screen_root main_margin(100.) image(&bg)) {
        Menu(rules(px(310), pct(100)) main_margin(40.) image(&board) column) {
            TitleCard(image(&title_card) width(pct(100)))
            TitleCard2(ui(title_card) width(pct(50)))
            code(let cmds) {
                dsl!(cmds, Buttons(column height(child(2.)) width(pct(100))));
                cmds.with_children(|cmds|{
                    for n in &menu_buttons {
                        let name = format!("{n} button");
                        dsl!(
                            &mut cmds.spawn_empty(),
                            Entity(ui(*n) named(name) image(&button) height(px(33)))
                        );
                    }
                });
            }
        }
    }
};
}
```

</details>

## Running examples

Use the `cargo run --bin` command to list possible examples, and run them.

We do this because it allows us to have different dependencies between examples.

### Specific example docs

#### `chirpunk`

A clone of the cyberpunk 2077 main menu and settings menu.

Demonstrates full end-to-end usage of `.chirp`, including common patterns for
managining complexity.

This example requires additional steps to work properly.

Check the [example's README](./examples/chirpunk/) for more details.

#### `simple_menu`

A single menu made using `cuicui_dsl`.

#### `dsl_and_chirp`

Demonstrates the equivalence between the `dsl!` macro and the `.chirp` file
format. Also used as a test to make sure it is trully equivalent.

#### `sprite_debug`

Demonstrates usage of `cuicui_layout_bevy_sprite`. Due to a quirk in the way
cargo resolves workspace features, the debug overlay is specifically broken for
this. You need to use the following command line to run it with the layout debug
overlay:

```sh
cargo run --bin sprite_debug -p sprite_debug --features cuicui_layout/debug
```

## Stability

This crate is in expansion, use at your own risk, it is extremely likely that
a lot of things are going to break a lot.

## Using `cuicui_layout`

First, chose which crate you want to use:

- Interested in a ready-made UI library on top of `bevy_ui`? [`cuicui_layout_bevy_ui`] is for you.
- Want more flexibility? Using [`cuicui_layout`] on top of `bevy_sprite` will let you
  integrate your UI with a lot of 3rd party crates that only work with sprites.
  [`cuicui_layout_bevy_sprite`] is for you.
- Using a custom renderer or want your UI to be part of the 3D environment?
  Build on top of [`cuicui_layout`] itself then.
- Are you making a complex menu requiring a lot of iterations? Consider using
  [`cuicui_chirp`] and the `.chirp` file format!

Secondly, add your chosen integration crate to your `Cargo.toml`:

```toml
[dependencies]
cuicui_layout_bevy_ui = "0.9.0"
cuicui_layout = "0.9.0"
```

Then, use `cuicui_layout` in your crate with the [`dsl!`] macro:

```rust,no_run
use bevy::prelude::*;
use cuicui_layout::{dsl, LayoutRootCamera, dsl_functions::*};
use cuicui_layout_bevy_ui::UiDsl as Dsl;

fn main() {
    // Do not forget to add cuicui_layout_bevy_{ui,sprite}::Plugin
    App::new().add_plugins((DefaultPlugins, cuicui_layout_bevy_ui::Plugin))
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands) {
    // Use LayoutRootCamera to mark a camera as the screen boundaries.
    commands.spawn((Camera2dBundle::default(), LayoutRootCamera));
    dsl! { &mut commands.spawn_empty(),
        // Use screen_root to follow the screen's boundaries
        Entity(row screen_root) {
            Entity(row margin(9.) border(5, Color::CYAN) bg(Color::NAVY)) {
                Entity(ui("Hello world!"))
            }
        }
    };
}
```

That's it! You are now using `cuicui_layout`, congratulations!

Make sure to check the [`LayoutDsl`] docs to learn the current capabilities of
`cuicui_layout`.

### What's that [`dsl!`] macro?

The previous snippet can be translated to:

```rust,no_run
use bevy::prelude::*;
use cuicui_layout_bevy_ui::UiDsl as Dsl;

// ...

fn setup(mut commands: Commands) {
    use cuicui_dsl::DslBundle;
    let mut x = <Dsl>::default();
    x.row();
    x.screen_root();
    x.node(&mut commands.spawn_empty(), |cmds| {
      let mut x = <Dsl>::default();
      x.row();
      x.margin(9.);
      x.border(5, Color::CYAN);
      x.bg(Color::NAVY);
      x.node(&mut cmds.spawn_empty(), |cmds| {
        let mut x = <Dsl>::default();
        x.ui("Hello world!");
        x.insert(&mut cmds.spawn_empty());
      });
    });
}
```

In short, the identifiers between parenthesis **are just methods**. Check the
documentation on the relevant `FoobarDsl` struct to learn which methods you
can use!

Also check the [`cuicui_dsl`] crate documentation for details on the
`DslBundle` trait (the trait providing the `node` and `insert` methods)
and the `IntoEntityCommands` trait (for the `to_cmds` method).

### What's that `.chirp` file format?

See the [`cuicui_chirp` crate README](./chirp).

#### How do I use `.chirp` files?

Reproducing the previous example with `.chirp` files:

First, write the chirp file:

```ron
// file: <scene.chirp>
// Use screen_root to follow the screen's boundaries
Entity(screen_root row) {
    Entity(margin(9) border(5, cyan) bg(navy) row) {
        Entity(text("Hello world!"))
    }
}
```

Second, add the plugin and load the chirp file:

```rust,no_run
use bevy::prelude::*;
use cuicui_layout::{dsl, LayoutRootCamera, dsl_functions::*};
use cuicui_layout_bevy_ui::UiDsl;
use cuicui_chirp::Chirp;

fn main() {
    // Do not forget to add cuicui_layout_bevy_{ui,sprite}::Plugin
    // and cuicui_chirp::loader::Plugin with the wanted DSL as type parameter
    App::new().add_plugins((
        DefaultPlugins,
        cuicui_layout_bevy_ui::Plugin,
        cuicui_chirp::loader::Plugin::new::<UiDsl>(),
    ))
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), LayoutRootCamera));
    // Spawn the chirp scene as is. Yeah that's it.
    commands.spawn(assets.load::<Chirp, _>("scene.chirp"));
}
```

[`cuicui_layout_bevy_sprite`]: https://lib.rs/crates/cuicui_layout_bevy_sprite
[`cuicui_layout_bevy_ui`]: https://lib.rs/crates/cuicui_layout_bevy_ui
[`cuicui_layout`]: https://lib.rs/crates/cuicui_layout
[`cuicui_dsl`]: https://lib.rs/crates/cuicui_dsl
[`cuicui_chirp`]: https://lib.rs/crates/cuicui_chirp
[`LayoutDsl`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/dsl/struct.LayoutDsl.html
[`ReflectDsl`]: https://docs.rs/cuicui_chirp/latest/cuicui_chirp/reflect/struct.ReflectDsl.html
[`dsl!`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html

## `cuicui_layout` crates

This repository contains several crates:

- `cuicui_dsl` ([dsl](dsl)): The `dsl!` macro and `DslBundle`.
- `cuicui_chirp` ([chirp](chirp)): A parser for files that follow the `dsl!`
  syntax. It creates a scene identical to what the same text passed to the
  `dsl!` macro would produce.
  \
  It also includes a bevy plugin to load `.chirp` files defined in this format.
- `cuicui_layout` ([layout](layout)): The base algorithm and components, does not make any assumption
  about how it is used, beside the requirement that layout nodes be bevy `Entitiy` and
  uses `bevy_hierarchy`. Exports a `LayoutDsl` to use with the `dsl!` macro.
- `cuicui_layout_bevy_ui` ([ui](ui)): Integration with `bevy_ui`, including extension to `UiDsl`
  for `UiImage`, `Text`, background images and background colors.
- `cuicui_layout_bevy_sprite` ([sprite](sprite)): `bevy_sprite` integration, supports
  `Mesh2dHandle`, `Sprite` and `Text2d`. This isn't as good as the `bevy_ui`-based integration
  when it comes to content-driven sizes, but otherwise should work very much like the `bevy_ui`
  integration.

## Cargo features

- `cuicui_chirp/macros` (on by default): Define and export the `parse_dsl_impl`
  attribute macro. This allows deriving `ParseDsl` automatically from an `impl`
  block.
- `cuicui_chirp/fancy_errors` (on by default): Display error message with source
  code context and actionable messages when failing to load a `.chirp` file.
  Note that this is only used in the `ChirpLoader` bevy asset loader, and the
  `Chirp::interpret_logging`. If disabled, a more rudimentary message is shown
  instead.
- `cuicui_layout/debug` (**off** by default): An overlay showing layout outlines & the rule type used
  by nodes. See [debug.md] for a detailed feature list.
- `cuicui_layout/chirp` (on by default): Add `ParseDsl` (chirp file format) implementation for `LayoutDsl`.
- `cuicui_layout/dsl` (on by default): Add `DslBundle` (`dsl!` macro) implementation for `LayoutDsl`.
- `cuicui_layout/reflect` (on by default): Derive `bevy_reflect` traits for cuicui_layout
  types & register them.
- `cuicui_layout_bevy_ui/chirp` (on by default): Add `ParseDsl` (chirp file format) implementation for `UiDsl`.
- `cuicui_layout_bevy_sprite/chirp` (on by default): Add `ParseDsl` (chirp file format) implementation for `SpriteDsl`.
- `cuicui_layout_bevy_sprite/sprite_text` (on by default): implement content-sized layout nodes
  for `Text2dBundle`.

[debug.md]: https://docs.rs/cuicui_layout/latest/cuicui_layout/debug/index.html

## Why cuicui layout

- Friendly algo with less things to keep in your head and good defaults.
- Uses and takes full advantage of the bevy ECS.
- Only controls `LayoutRect`, not `Transform`, you need to add a system that sets
  `Transform` based on `LayoutRect`.
- Fully flexible and extensible, can be used with `bevy_ui`, `bevy_sprite`, your own stuff.
- Fantatstically easy to extend, like really.
- Helpful and fully detailed error messages when things are incoherent or broken.
  As opposed to FlexBox, which goes "this is fine 🔥🐶🔥" and leaves you to guess
  why things do not turn out as expected.
- `cuicui_layout`'s algo runs in `O(n)` where `n` is how many nodes you have.
- An extensive debugging overlay.
- Working hot reloading.
- The `chirp` (and `dsl!`) grammar is 15 lines, one of them only exists to make
  error reporting better.

## Why not Flexbox

You are writing text to get 2d visual results on screen.
The translation from text to screen should be trivial, easy to do in your head.
Otherwise you need visual feedback to get what you want.
Bevy, even with hot reloading or [`bevy-inspector-egui`]
will always have extremely slow visual feedback.

Flexbox has too many parameters and depends on implicit properties of UI elements,
it is not possible to emulate it in your head.

cuicui's layout, in contrast to Flexbox is easy to fit in your head.
In fact, I will forecefully push cuicui's layout algorithm in your head
in two short bullet points.

- A node can be a `Node::Container` and distribute its children
  along a `Direction` either by evenly spacing them (`Distribution::FillMain`)
  or putting them directly one after another (`Distribution::Start`).
- A `Container`'s size can be expressed as a static value, a fraction
  of the size of what contains it, or a multiple of what it contains.
- The content of a `Container` can be `Alignment` to the start, end or center
  of its parent (by default it's centered).

That's it. There are some edge cases, but cuicui will ~~yell at you~~
tell you nicely when you hit them and tell you how to handle them properly.

[`bevy-inspector-egui`]: https://lib.rs/crates/bevy-inspector-egui

### Flexbox FAQ

**Q**: Where is `padding`?
<br>**A**: `padding` is equivalent to `margin` in cuicui_layout. `margin` and `border`
doesn't make conceptual sense.

**Q**: Why not call it `padding` then?
<br>**A**: Look at the dictionary definition of "margin" and "padding".

**Q**: How do I center a node?
<br>**A**: nodes are centered by default, make sure the parent's container size
has the expected size.

**Q**: What is the equivalent of `flex_direction`?
<br>**A**: use `row` and `column`

**Q**: What are the equivalents of `column-reverse` and `row-reverse`?
<br>**A**: None. Use `Alignment::End` and swap your elements! Note that the `*-reverse`
flows in flexbox are very useful for internationalization. However,
when making a game, it is not enough to just swap the elements! Artistic control is
paramount and internationalization needs to be taken as a whole in the context of the UI.

**Q**: What is the equivalent of `flex_wrap`?
<br>**A**: None, do you really need it?

**Q**: What is the equivalent of `align_item`, `align_self`, `align_content`, `justify_content`?
<br>**A**: After 5 years of working with CSS, I still have no clue which one does what,
and whether they really do anything, so I wont' adventure an asnwer.

**Q**: What is the equivalent of `flex_grow`, `flex_shrink`, `flex_basis`, `gap`?
<br>**A**: Do you even know what they do?

**Q**: Why can't child container overflow their parents?
<br>**A**: It's likely you didn't expect this, so we report it as an error.

**Q**: How do I make a grid?
<br>**A**: `cuicui_layout` is currently not capable of managing a grid of nodes.
This might be added in the future.

#### Why not add \<Flexbox feature\> to `cuicui_layout`?

Each flexbox feature is useful taken in isolation, but when combined,
they make for a very difficult to grasp whole. It's the combinatorial explosion
of interactions between features that makes Flexbox impossible to emulate in
your head. In fact, I'm not so sure anything short of a Flexbox implementation
can predict what the final output of your CSS will look like.

With this settled, it is natural that I aim to make `cuicui_layout` as featureless
as possible. Ideally, there is exactly one way to do anything, even if it requires
a bit of head scratching to get there. Code with less feature is paradoxically
smarter. A narrow set of functionalities allow easier inference on the user's
expectations, enabling better error messages and suggestions.

Of course, as a library, `cuicui_layout` must at least have _some_ features.
Here is what I look in a new feature:

- The feature is inherent to layouting, ie: this isn't the job of a 3rd party
  integration plugin.
- The feature can only interact in meaningful and predictible ways with other
  existing features.
- The feature introduces only meaningful abstractions/concepts that are fully
  orthogonal with other features.
- The feature is not too complex to implement

Here is an example: `margin`. At first, I didn't even want margins.
After all, I can nest a container within another one with padding empty nodes.
Right? Well no.

Say you have the following layout:

```text
dsl! {
  Entity(rules(pct(100), pct(100)) row) {
    Entity(margins(10) rules(pct(100), pct(100)) row) {
      Entity(some_element)
    }
  }
}
```

We can't use padding nodes here. Because the inner `row` depends on the size
of the parent. Adding nodes would make the inner `row` always overflow the
outer row, because it's size will still be 100% that of its parent, in addition
to the two 10 pixel empty padding nodes.

The solution to this is to have a distinction between "outer" and "inner" sizes:

- outer size is the size as seen by the parent
- inner size is the size seen by the children
- inner size = outer size - margin * 2.

The `margin` now allows specifying the inner `row` as 100% the "size" of the
outer row. In fact it's specifying the size relative to the outer row's size
minus the given `margin`.

Now why limit `margin` to pixel specification, eschewing percent-based rules?

Paradoxically, it's for usability:
It is not clear what the percent is a percentage of. Is it the inner size?
The parent's size? children size? The full size after the application of
the margin?

We don't know, different people will have different expectations. This avoids
any confusion. In any case, _this is_ a situation where empty nodes can
be used, since you'll be able to compute the relative size of each node yourself.

### Change log

See the [./CHANGELOG.md] file.


### Version matrix

| bevy | latest supporting version      |
|------|-------|
| 0.11 | 0.9.0 |
| 0.10 | 0.3.0 |

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](licenses/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](licenses/LICENSE-MIT) or http://opensource.org/licenses/MIT)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.