# [`cuicui_layout`] integration for [`bevy_ui`]

[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)
[![Documentation](https://docs.rs/cuicui_layout_bevy_ui/badge.svg)](https://docs.rs/cuicui_layout_bevy_ui/)

Provides the [`UiDsl`] trait, extending the [`cuicui_layout`]'s `LayoutDsl`
with the following features:

- `Style` border color and width (but only uniform)
- UI node background color
- UI node background image
- Direct spawning of text and images as argument to [`ui`].

Note that this is in addition to the methods `LayoutDsl` already supports.

Additionally, `cuicui_layout_bevy_ui` provides a [`ContentSized`] implementation
that supports `Text` and `UiImage` terminal nodes
which size is determined by their content.

Check the following crates for details:

- [`cuicui_layout`]: the layouting algorithm
- [`cuicui_dsl`]: the `dsl!` macro and what [`UiDsl`] entails.

[`cuicui_layout`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/
[`bevy_ui`]: https://docs.rs/bevy_ui/latest/bevy_ui/
[`UiDsl`]: https://docs.rs/cuicui_layout_bevy_ui/latest/cuicui_layout_bevy_ui/struct.UiDsl.html
[`ui`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/dsl/struct.LayoutDsl.html#method.ui
[`ContentSized`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/dsl/struct.ContentSized.html
[`cuicui_dsl`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/