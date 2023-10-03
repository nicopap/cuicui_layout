# Define a menu using a chirp file

> **Note**
> This section is abbridged and might go in much more details in the future.

The app setup requires adding `cuicui_chirp::loader::Plugin::new::<UiDsl>()`,
we also setup hot reloading by setting the asset plugin.

```rust,no_run,noplayground
{{#include main.rs:app}}
```

The setup, however, is completely trivial, since it is now the loading plugin
that takes care of spawning the UI.

```rust,no_run,noplayground
{{#include main.rs:setup}}
```

Now, we write the `chirp` file in the `assets/` folder.

This is fairly close to a copy/paste of the one in ???, with the distinction
that we use an `fn` template instead of a `code` block to avoid repetition for
the menu buttons.

```ron
{{#include ../../assets/chirp_menu.chirp}}
```

Similarly to the `dsl`-based example, the documentation for which methods are
available is on `docs.rs` (or your local copy with `cargo doc`) for [`UiDsl`]
and [`LayoutDsl`].

Try running the `chirp_menu` example and modifying `chirp_menu.chirp` to see
how it affects the layout!

[`LayoutDsl`]: https://docs.rs/cuicui_layout/0.9.0/cuicui_layout/dsl/struct.LayoutDsl.html
[`UiDsl`]: https://docs.rs/cuicui_layout_bevy_ui/0.9.0/cuicui_layout_bevy_ui/dsl/struct.UiDsl.html
