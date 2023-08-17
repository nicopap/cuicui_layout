# The `cuicui_dsl` file format

This is a [`cuicui_dsl`] companion crate.

It provides a trait to enable defining `cuicui_dsl` UI trees outside of rust,
in an independent file format.

Currently it is based on the [KDL](https://kdl.dev/) config format, but may
in the future change to look exactly like the `dsl!` macro (or reversly, the
`dsl!` macro may change to look exactly like KDL)

## Features

* **`derive`** (default): Define the `parse_dsl_impl` macro. If you are not using
  the proc macro and defining `ParseDsl` implementations manually, you can
  disable this feature for faster compile times.