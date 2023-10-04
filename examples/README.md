## Running examples

[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)

Use the `cargo run --bin` command to list possible examples, and run them.

We do this because it allows us to have different dependencies between examples.

### Specific example docs

#### `hello_world`

The most minimal code to show usage of `cuicui_layout` using `cuicui_dsl`

#### `simple_menu`

A single menu made using `cuicui_dsl`.

#### `chirp_menu`

A single menu supporting hot reloading made using `cuicui_chirp`.

#### `chirpunk`

A clone of the cyberpunk 2077 main menu and settings menu.

Demonstrates full end-to-end usage of `.chirp`, including common patterns for
managining complexity.

This example requires additional steps to work properly.

Check the [example's README](./chirpunk/) for more details.

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

#### `templates`

demonstrates usage of the `cuicui_chirp` templating features. See the file in
`assts/templates.chirp` for details, as most of the interesting code is in the
chirp file itself, not the rust source code.
