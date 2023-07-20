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

### Code

```rust
use bevy::prelude::*;
use cuicui_layout_bevy_ui::UiDsl as Dsl;
use cuicui_layout::{LayoutRootCamera, dsl, dsl_functions::{px, pct}};

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
    &mut cmds,
    row(screen_root, "root", main_margin 100., distrib_start, align_start, image &bg) {
        column("menu", width px(310), height pct(100), main_margin 40., image &board) {
            spawn(image &title_card, "Title card", width pct(100));
            spawn_ui(title_card, "Title card 2", width pct(50));
            code(let cmds) {
                for n in &menu_buttons {
                    let name = format!("{n} button");
                    dsl!(cmds, spawn_ui(*n, named name, image &button, height px(33)););
                }
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

Secondly, add your chosen integration crate to your `Cargo.toml`:

```toml
[dependencies]
cuicui_layout_bevy_ui = "0.8.0"
cuicui_layout = "0.8.0"
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
    dsl! { &mut commands,
        // Use screen_root to follow the screen's boundaries
        row(screen_root) {
            row(margin 9., border(5, Color::CYAN), bg Color::NAVY) {
                spawn_ui("Hello world!");
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
    use cuicui_dsl::{IntoEntityCommands, DslBundle};
    let mut x = <Dsl>::default();
    x.screen_root();
    x.node(commands.to_cmds(), |cmds| {
      let mut x = <Dsl>::default();
      x.margin(9.);
      x.border(5, Color::CYAN);
      x.bg(Color::NAVY);
      x.node(cmds.to_cmds(), |cmds| {
        let mut x = <Dsl>::default();
        let mut cmds = cmds.to_cmds();
        x.insert(&mut cmds);
        x.spawn_ui("Hello world!", &mut cmds);
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

[`cuicui_layout_bevy_sprite`]: https://lib.rs/crates/cuicui_layout_bevy_sprite
[`cuicui_layout_bevy_ui`]: https://lib.rs/crates/cuicui_layout_bevy_ui
[`cuicui_layout`]: https://lib.rs/crates/cuicui_layout
[`cuicui_dsl`]: https://lib.rs/crates/cuicui_dsl
[`LayoutDsl`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/dsl/struct.LayoutDsl.html
[`dsl!`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html

## `cuicui_layout` crates

This repository contains several crates:

- `cuicui_dsl` ([dsl](dsl)): The `dsl!` macro and `DslBundle`.
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

- `cuicui_layout/debug` (off by default): An overlay showing layout outlines & the rule type used
  by nodes. See [debug.md] for a detailed feature list.
- `cuicui_layout_bevy_sprite/sprite_text` (on by default): implement content-sized layout nodes
  for `Text2dBundle`.
- `cuicui_layout/reflect` (on by default): Derive `bevy_reflect` traits for cuicui_layout
  types & register them.

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
  row(rules(pct(100), pct(100))) {
    row(margins 10, rules(pct(100), pct(100))) {
      some_element();
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


### Version matrix

| bevy | latest supporting version      |
|------|-------|
| 0.11 | 0.8.0 |
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