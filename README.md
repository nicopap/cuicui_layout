[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![Latest version](https://img.shields.io/crates/v/cuicui_layout.svg)](https://crates.io/crates/cuicui_layout)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](./LICENSE)
[![Documentation](https://docs.rs/cuicui_layout/badge.svg)](https://docs.rs/cuicui_layout/)

# Cuicui Layout

A dumb layout algorithm you can rely on, built for and with bevy.

<details><summary><h2>The Cyberpunk 2077 showcase</h2></summary>

For some reasons, the Cyberpunk main menu has become the 7GUI of bevy, so here
is the Cyberpunk main menu using `cuicui_layout_bevy_ui`.

<video controls>
  <source
    src="https://github.com/nicopap/cuicui_layout/assets/26321040/8a51f9a9-ffa7-4b60-a2ad-3947ff718e27.mp4"
    type="video/mp4"
  />
</video>

### Code

```rust
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
let font = serv.load("adobe_sans.ttf");
let bg = serv.load("background.png");
let board = serv.load("board.png");
let button = serv.load("button.png");

layout! {
    &mut cmds,
    row(screen_root, "root", main_margin 100., align_start, image &bg) {
        column("menu", width px 300, fill_main_axis, image &board) {
            spawn_ui(title_card, "Title card", height px 100, width %100);
            code(let cmds) {
                for n in &menu_buttons {
                    let name = format!("{n} button");
                    layout!(cmds, spawn_ui(*n, named name, image &button, height px 30););
                }
            }
        }
    }
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

1. Chose which crate you want to use:
    - Interested in a ready-made UI library on top of `bevy_ui`? [`cuicui_layout_bevy_ui`] is for you!
    - Want more flexibility? Using `cuicui_layout` on top of `bevy_sprite` will let you
      integrate your UI with a lot of 3rd party crates that only work with sprites!
      [`cuicui_layout_bevy_sprite`] is for you!
    - Using a custom renderer or want your UI to be part of the 3D environment?
      Build on top of [`cuicui_layout`] itself then!
2. Add the chosen crate as a dependency to your crate. `cargo add cuicui_layout_bevy_ui`
3. Use the [`layout!`] macro to build a UI (text representation coming soon).
4. That's it! You are now using `cuicui_layout`, congratulations!
   Make sure to check the [`LayoutType`]
   docs to learn the current capabilities of `cuicui_layout`.

Please note that `cuicui_layout` won't magically make sprite components work in
UI nodes.

[`cuicui_layout_bevy_sprite`]: https://lib.rs/crates/cuicui_layout_bevy_sprite
[`cuicui_layout_bevy_ui`]: https://lib.rs/crates/cuicui_layout_bevy_ui
[`cuicui_layout`]: https://lib.rs/crates/cuicui_layout
[`LayoutType`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/dsl/struct.LayoutType.html
[`layout!`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/macro.layout.html

## `cuicui_layout` crates

This repository contains several crates:

- `cuicui_layout` ([layout](layout)): The base algorithm and components, does not make any assumption
  about how it is used, beside the requirement that layout nodes be bevy `Entitiy` and
  uses `bevy_hierarchy`.
- `cuicui_layout_bevy_ui` ([ui](ui)): Integration with `bevy_ui`, including extension to `LayoutType`
  for `UiImage`, `Text`, background images and background colors.
- `cuicui_layout_bevy_sprite` ([sprite](sprite)): `bevy_sprite` integration, supports
  `Mesh2dHandle`, `Sprite` and `Text2d`. This isn't as good as the `bevy_ui`-based integration
  when it comes to content-driven sizes, but otherwise should work very much like the `bevy_ui`
  integration.

(maybe `cuicui_layout_spec` in the future)

## Cargo features

- `cuicui_layout_bevy_sprite/sprite_text` (on by default): implement content-sized layout nodes
  for `Text2dBundle`.
- `cuicui_layout/reflect` (on by default): Derive `bevy_reflect` traits for cuicui_layout
  types & register them.

## Why cuicui layout

- Friendly algo with less things to keep in your head and good defaults.[^1]
- Uses and takes full advantage of the bevy ECS.
- Only controls `PosRect`, not `Transform`, you need to add a system that sets
  `Transform` based on `PosRect`.
- Fully flexible and extensible, can be used with `bevy_ui`, `bevy_sprite`, your own stuff.
- Fantatstically easy to extend, like really.
- Helpful and fully detailed error messages when things are incoherent or broken.[^1]
  As opposed to FlexBox, which goes "this is fine 🔥🐶🔥" and leaves you to guess
  why things do not turn out as expected.

[^1]: aspirational, currently not really the case.

## Why not Flexbox

You are writing text to get 2d visual results on screen.
The translation from text to screen should be trivial, easy to do in your head.
Otherwise you need visual feedback to get what you want.
Bevy, even with hot reloading or [`bevy-inspector-egui`]
will always have extremely slow visual feedback.

Flexbox has too many parameters and depends on implicit properties of UI elements,
it is not possible to emulate it in your head.

cuicui's layout in contrast to Flexbox is easy to fit in your head.
In fact, I will forecefully push cuicui's layout algorithm in your head
in two short bullet points.

- A node can be a `Node::Container` and distribute its children
  along a `Direction` either by evenly spacing them (`Distribution::FillParent`)
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

**Q**: How do I center a node?
<br>**A**: Add an empty node at the start and end of the container, and use `fill_parent`

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