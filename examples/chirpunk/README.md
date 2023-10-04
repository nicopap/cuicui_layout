# Chirpunk

[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)

The cuicui cyberpunk example.

Inspired by [`bevy-lunex`]'s [cyberpunk example].

This demonstrates how to create and handle a more complex menu. It has:

- Several screens (title screen, graphics settings, audio settings, bonus tabs)
- Buttons with juicy hover animations
- Nice screen effects such as background animation and bloom
- Gamepad handling with [`bevy-ui-navigation`]
- Minimal data binding based on events.


## Building

Due to the nature of the assets used, I prefer not including them in this
repository. To get them, run the `make checkout-cyberpunk` make recipe.

The idea is to clone the bevy-lunex cyberpunk example and re-use its `asset`
directory.

If `make` is not an option for you, the following shell commands _should work_,
although it has only been proven to work on my machine™.

```sh
# First Make sure that your working directory (PWD) is the cuicui_layout workspace root.
if not test -e CHANGELOG.md ; then
  echo "your working directory (PWD) should be the cuicui_layout workspace root."
  exit 1
fi

# Create a repository for bevy-lunex-cyberpunk, without copying the files
git clone --no-checkout --depth=1 --filter=tree:0 \
    https://github.com/IDEDARY/bevy-lunex-cyberpunk.git \
    examples/chirpunk/lunex-cyberpunk-assets
# Go into bevy-lunex-cyberpunk repo and tell it to only download the 'assets' directory
cd examples/chirpunk/lunex-cyberpunk-assets
git sparse-checkout set --no-cone assets
# Check out the bevy-lunex-cyberpunk files
git checkout

# Setup the 'assets' directory in ./examples/chirpunk
cd ..
../../scripts/x_platform_ln.sh lunex-cyberpunk-assets/assets assets
cd lunex-cyberpunk-assets/assets
# add 'menus' files to the assets directory
../../../../scripts/x_platform_ln.sh ../../menus menus
cd ../../../..
```

## Running

> `cargo run --bin chirpunk --features cuicui_layout/debug`
>
> (the feature flag is optional)

Use `--no-default-features` to not spawn the `bevy-inspector-egui` world inspector.
And use `--features advanced_logging` to log more stuff.

## Limitations

- Uses `bevy_ui` (through `cuicui_layout_bevy_ui`):
  - Bloom doesn't work on UI
  - **Requires a patches version of bevy for hot reloading** to work (see the
    repository's workspace `Cargo.toml`)
- Missing cuicui features:
  - A "all overlapping" `Distribution` mode, to replace some of the `MenuSwatch`
    functionality and the deep nesting on the main menu background.
  - A "templating" feature, replacing `dsl::element`
  - Single root attribute, so to avoid some nesting required for `bevy_ui` to
    not panic, and generally better scene management.
- General 3rd party crate ideas:
  - Extract and generalize the `style.rs` module, which is really cool.
- When using mouse input, the current tab in the settings menu is not highlighted.
- The "BACK" button is part of the tabs menu


The end-goal is to use `cuicui_layout_bevy_sprite` instead of
`cuicui_layout_bevy_ui` as "rendering backend". As we don't need `bevy_ui` for
this specific example (the only benefit of `bevy_ui` over `bevy_sprite` is
layouting, click management and borders, of which we use none), and bevy' sprite
renderer is more flexible.

But I wanted to start with a working example, and `cuicui_layout_bevy_sprite`
still is missing some basic features to make it useable.

In the future, we will add a new bin target, that re-uses most of the code but
uses the `dsl!` macro instead of chirp files.


## Architecture

Since this is a complex example, it needs a bit of a "map" so that you can
orient yourself and find the landmarks that is most relevant to you.

We have five modules:

- `dsl`: **The most important module**. A wrapper around `UiDsl` to add to the
  DSL a vocabulary specific to our own UI, such as "main_menu_item" or "tab_button".
  The "method names" you see used in the `.chirp` files and `dsl!` macros are
  methods on `BevypunkDsl`, and `UiDsl`, and `LayoutDsl` and `BaseDsl`.
  \
  Those are methods you can call using regular rust method syntax!
- `animate`: Animation components, used for the shift-on-hover & background
  police car strobe lights in the title screen
- `colormix`: defines `color_lerp` to blend bevy `Color`s in HSLuv space, used
  in `animate`
- `ui_offset`: Simple plugin to apply object movement AFTER `bevy_ui`'s layouting
  system. Used in `animate` for the shift-on-hover effect.
- `style`: A styling module. It's a way to change styling variable at runtime
  throuhg the `style::Bevypunk` resource. This could also be loaded as a resource
  or modified through `bevy-inspector-egui`.

The `.chirp` files defining the menus are in the `menus` directory.

[`bevy-lunex`]: https://github.com/bytestring-net/bevy-lunex
[cyberpunk example]: https://github.com/IDEDARY/bevy-lunex-cyberpunk
[`bevy-ui-navigation`]: https://lib.rs/crates/bevy-ui-navigation