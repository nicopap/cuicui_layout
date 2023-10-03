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

## Using `cuicui_layout`

The [Usage section of the book][book-usage] is a good starting point.

### MOAR DOCS!!

- [The cuicui Book]
- [The docs.rs API docs]

### For the lazy

Please read the [Usage section of the book][book-usage]. Skip to the code
if you don't care for explanations.

### Change log

See the [./CHANGELOG.md] file.

### Version matrix

| bevy | latest supporting version |
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

[book-usage]: https://nicopap.github.io/cuicui_layout/usage.html