# The `chirp` file format

The `.chirp` file format is a **general bevy scene text file format**. It was
designed to look as close as possible to the [`dsl!`] macro from `cuicui_dsl`.

It just so happen that all `cuicui` crates are compatible with `.chirp` files.

It is a custom file format. The parser is written using `winnow` and directly
interprets the bits.

It provides a bevy `Plugin` to load `.chirp` files with the `AssetServer` as
scenes.

### Usage

First, write the chirp file:

```ron
// file: <scene.chirp>
// Use screen_root to follow the screen's boundaries
row(screen_root) {
    row(margin 9, border(5, cyan), bg navy) {
        spawn(text "Hello world!");
    }
}
```
Second, add the plugin and load the chirp file:
```rust,no_run
use bevy::prelude::*;
use cuicui_dsl::BaseDsl;
use cuicui_chirp::Chirp;

fn main() {
    // Do not forget to add cuicui_layout_bevy_{ui,sprite}::Plugin
    // and cuicui_chirp::loader::Plugin with the wanted DSL as type parameter
    App::new().add_plugins((
        DefaultPlugins,
        cuicui_chirp::loader::Plugin::new::<BaseDsl>(),
    ))
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // commands.spawn((Camera2dBundle::default(), LayoutRootCamera));
    // Spawn the chirp scene as is. Yeah that's it.
    commands.spawn(assets.load::<Chirp, _>("scene.chirp"));
}
```

See the `cuicui_layout` repository root README for more details.

### Differences with the `dsl!` macro

- It can be hot reloaded, and scenes are cleaned up and reloaded properly, unlike rust code.
- It supports an additional feature: [`ReflectDsl`]. It implements `ParseDsl`
  based on a `Reflect + Bundle` type. No such thing is possible with the `dsl!`
  macro.
- `.chirp` files support arbitrary syntax. For example, we use [`css-color`]
  to parse colors.
- As a separate file format, chirp files are not written within rust code
- It can't directly inline rust code as in the `dsl!` macro
- It requires implementing an additional trait on your `DslBundle`. The trait
  in question is `ParseDsl`. You may use `parse_dsl_impl` to make this easy.
- It requires loading assets synchronously, using `FileAssetIo`.

Both crates are not mutually exclusive.
You can prototype with a `.chirp` file, then copy the chirp content
into a `dsl!` macro. This is how the `chirpunk` example was built.

## Features

* **`macros`** (default): Define the `parse_dsl_impl` macro. If you are not using
  the proc macro and defining `ParseDsl` implementations manually, you can
  disable this feature for faster compile times.
* **`load_font`** and **`load_image`** (default): Enable loading `Font` and `Image`
  assets from `.chirp` files. Disable those features if you are not using `Font`s
  or `Image`s
* **`fancy_errors`** (default): Show position in source code when failing to parse
  `.chirp` files.

[`css-color`]: https://lib.rs/crates/css-color
[`ReflectDsl`]: https://docs.rs/cuicui_chirp/latest/cuicui_chirp/reflect/struct.ReflectDsl.html
[`dsl!`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html
