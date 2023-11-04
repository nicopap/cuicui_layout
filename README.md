[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![Latest version](https://img.shields.io/crates/v/cuicui_layout.svg)](https://crates.io/crates/cuicui_layout)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](./LICENSE)
[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)

# The Cuicui Framework

The `cuicui` framework is a collection of rust crates made for bevy.

- [`cuicui_layout`]: A dumb layout algorithm you can rely on, built for and with bevy.
- [`cuicui_dsl`] and [`cuicui_chirp`]: Two enjoyable ways to spawn scenes in bevy.

<details><summary><b>The Chirpunk demo</b></summary>

For some reasons, the Cyberpunk main menu has become the 7GUI of bevy, so here
is the Cyberpunk main menu using `cuicui_layout_bevy_ui`.

https://user-images.githubusercontent.com/26321040/272480834-e964565b-44bb-4363-8955-19515624d71a.mp4

| **❗ Hot reloading disclaimer ❗** |
|------------------------------------|

Chirp hot reloading with `bevy_ui` components (ie: using `cuicui_layout_bevy_ui`)
is broken due to <https://github.com/bevyengine/bevy/pull/9621>.
You may want to work on a local patched version of bevy.
A workaround will probably be provided in cuicui 0.10.

The code for the video demo can be read in [the chirpunk example].

</details>

## Hello World

A tinny example to get you an idea of what you can do with `cuicui`.

Shows a blue box with cyan outline in the center of the screen:

```rust,no_run
use bevy::prelude::*;
use cuicui_layout::{dsl, LayoutRootCamera};
use cuicui_layout_bevy_ui::UiDsl as Dsl;

fn main() {
    // Do not forget to add cuicui_layout_bevy_{ui,sprite}::Plugin
    App::new()
        .add_plugins((DefaultPlugins, cuicui_layout_bevy_ui::Plugin))
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands) {
    // Use LayoutRootCamera to mark a camera as the screen boundaries.
    commands.spawn((Camera2dBundle::default(), LayoutRootCamera));

    dsl! { &mut commands.spawn_empty(),
        // Use screen_root to follow the screen's boundaries
        Entity(row screen_root) {
            // Stuff is centered by default.
            Entity(row margin(9.) border(5, Color::CYAN) bg(Color::NAVY)) {
                Entity(ui("Hello world!"))
            }
        }
    };
}
```

## Running examples

Use the `cargo run --bin` command to list possible examples, and run them.

We do this because it allows us to have different dependencies between examples.

## Using `cuicui_layout`

The [Usage section of the book][book-usage] is a good starting point.

### MOAR DOCS!!

- [The cuicui Book]

### For the lazy

Please read the [Usage section of the book][book-usage]. Skip to the code
if you don't care for explanations.

### Change log

See the [./CHANGELOG.md](./CHANGELOG.md) file.

### Version matrix

| bevy | latest supporting version |
|------|-------|
| 0.12 | 0.10.2 |
| 0.11 | 0.10.2 |
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

[book-usage]: https://cuicui.nicopap.ch/usage.html
[The cuicui Book]: https://cuicui.nicopap.ch/introduction.html
[the chirpunk example]: https://github.com/nicopap/cuicui_layout/tree/main/examples/chirpunk
[`cuicui_layout`]: https://cuicui.nicopap.ch/layout/index.html
[`cuicui_chirp`]: https://cuicui.nicopap.ch/chirp/index.html
[`cuicui_dsl`]: https://cuicui.nicopap.ch/dsl/index.html
