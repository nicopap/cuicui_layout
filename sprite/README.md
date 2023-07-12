# [`cuicui_layout`] integration for [`bevy_sprite`]

Provide the [`SpriteDsl`] trait, extending the [`cuicui_layout`]'s `LayoutDsl`
with the following features:

- Sprite background color
- Sprite background image
- Direct spawning of text and images as argument to [`spawn_ui`].

Note that this is in addition to the methods `LayoutDsl` already supports.

Additionally, `cuicui_layout_bevy_sprite` provides a [`ContentSized`]
implementation that supports `Text2dBounds` and `Handle<Image>` terminal nodes
which size is determined by their content.

Check the following crates for details:

- [`cuicui_layout`]: the layouting algorithm
- [`cuicui_dsl`]: the `dsl!` macro and what [`SpriteDsl`] entails.

[`cuicui_layout`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/
[`bevy_sprite`]: https://docs.rs/bevy_sprite/latest/bevy_sprite/
[`SpriteDsl`]: https://docs.rs/cuicui_layout_bevy_sprite/latest/cuicui_layout_bevy_sprite/struct.SpriteDsl.html
[`spawn_ui`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/dsl/struct.LayoutDsl.html#method.spawn_ui
[`ContentSized`]: https://docs.rs/cuicui_layout/latest/cuicui_layout/dsl/struct.ContentSized.html
[`cuicui_dsl`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/