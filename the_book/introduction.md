# The `cuicui` framework

`cuicui` is a collection of crates to work with bevy scenes and making UIs in bevy.

This book contains a user-focused guide on how to use the `cuicui` crates and
more reference-level documentation on what the capabilities of each crate are.

## Semantic versioning

All crates in the `cuicui` framework follow a synchronous release model, similar
to bevy's. However, the release cadence is independent from bevy's.

#### Version matrix

| bevy | latest supporting version |
|------|-------|
| 0.11 | 0.9.0 |
| 0.10 | 0.3.0 |

## Stability

`cuicui` is a playground for experiments, so use at your own risk,
it is likely that a lot of things are going to break each release.

## `cuicui` crates

The crates included in `cuicui` are:

- [`cuicui_dsl`]: The `dsl!` macro and `DslBundle`.
- [`cuicui_chirp`]: A parser for files that follow the `dsl!`
  syntax. It creates a scene identical to what the same text passed to the
  `dsl!` macro would produce.
  \
  It also includes a bevy plugin to load `.chirp` files defined in this format.
- [`cuicui_layout`]: The base algorithm and components, does not make any assumption
  about how it is used, beside the requirement that layout nodes be bevy `Entitiy` and
  uses `bevy_hierarchy`.
- [`cuicui_layout_bevy_ui`]: Integration with `bevy_ui`, including extension to `UiDsl`
  for `UiImage`, `Text`, background images and background colors.
- [`cuicui_layout_bevy_sprite`]: `bevy_sprite` integration, supports
  `Mesh2dHandle`, `Sprite` and `Text2d`. This isn't as good as the `bevy_ui`-based integration
  when it comes to content-driven sizes, but otherwise should work very much like the `bevy_ui`
  integration.

## Supporting development

This crate is a single person effort. I don't get paid for it, and is generally
unsustainable. Please consider donating to make `cuicui` sustainable.

<https://github.com/sponsors/nicopap>

[`cuicui_dsl`]: dsl
[`cuicui_chirp`]: chirp
[`cuicui_layout`]: layout
[`cuicui_layout_bevy_ui`]: choosing_a_cuicui_crate.html
[`cuicui_layout_bevy_sprite`]: choosing_a_cuicui_crate.html
