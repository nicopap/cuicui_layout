# The `cuicui_dsl` file format

This is a [`cuicui_dsl`] companion crate.

It provides a trait to enable defining `cuicui_dsl` UI trees outside of rust,
in an independent file format.

It is a custom file format. The parser is written using `winnow` and directly
interprets the bits.

It provides a bevy `Plugin` to load `.chirp` files with the `AssetServer` as
scenes.

## Features

* **`macros`** (default): Define the `parse_dsl_impl` macro. If you are not using
  the proc macro and defining `ParseDsl` implementations manually, you can
  disable this feature for faster compile times.
