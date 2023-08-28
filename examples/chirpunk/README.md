# Chirpunk

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
although it has only been proved to work on my machine™.

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

This crate has two bin targets. One uses the `dsl!` macro found in `cuicui_dsl`,
the other uses the `AssetLoader` for `.chirp` files found in `cuicui_chirp`.

Both spawn the same menus. Run one of the following commands to try them out:

- `cargo run --bin chirpunk_dsl`
- `cargo run --bin chirpunk_chirp`

Note that **currently only the chirp-based version exists**.


## Architecture

Since this is a complex example, it needs a bit of a "map" so that you can
orient yourself and find the landmarks that is most relevant to you.

We have five modules:

- `animate`: Animation components, used for the shift-on-hover & background
  police car strobe lights in the title screen
- `colormix`: defines `color_lerp` to blend bevy `Color`s in HSLuv space, used
  in `animate`
- `ui_offset`: Simple plugin to apply object movement AFTER `bevy_ui`'s layouting
  system. Used in `animate` for the shift-on-hover effect.
- `style`: A styling module. It's a way to change styling variable at runtime
  throuhg the `style::Bevypunk` resource. This could also be loaded as a resource
  or modified through `bevy-inspector-egui`.
- `dsl`: A wrapper around `UiDsl` to add to the DSL a vocabulary specific to our
  own UI, such as "main_menu_item" or "tab_button".

The `.chirp` files defining the menus are in the `menus` directory.


## Limitations

The end-goal is to use `cuicui_layout_bevy_sprite` instead of
`cuicui_layout_bevy_ui` as "rendering backend". As we don't need `bevy_ui` for
this specific example (the only benefit of `bevy_ui` over `bevy_sprite` is
layouting, click management and borders, of which we use none), and bevy' sprite
renderer is more flexible.

But I wanted to start with a working example, and `cuicui_layout_bevy_sprite`
still is missing some basic features to make it useable.

[`bevy-lunex`]: https://github.com/bytestring-net/bevy-lunex
[cyberpunk example]: https://github.com/IDEDARY/bevy-lunex-cyberpunk
[`bevy-ui-navigation`]: https://lib.rs/crates/bevy-ui-navigation