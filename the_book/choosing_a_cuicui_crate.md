# Choosing which crates to use

Confused by all the cuicui crates? Understandable, there is a lot of them, with
very long names too!

Let's split the task in two. Ask yourself two questions:

- What do I want to layout?
- How do I want to spawn UI scene?

The following sections tell you which crate to use depending on the answer.

## Layouting

All you want is some sort of layouting algorithm that you can keep in your head?
Then use [`cuicui_layout`] without any other crates.

- Interested in a ready-made UI library on top of `bevy_ui`?
  Then, use [`cuicui_layout_bevy_ui`].
- Want more flexibility? Using [`cuicui_layout`] on top of `bevy_sprite` will let you
  integrate your UI with a lot of 3rd party crates that only work with sprites.
  Then use [`cuicui_layout_bevy_sprite`].
- Using `cuicui_layout` with your own custom rendering system? Again, bare `cuicui_layout`
  is what you want.

## Scene syntax

Then you need to decide how to spawn full scenes or UI layouts.

- Using the default bevy syntax, with nested `cmds.insert(…).with_children(…)`?
  No need to add any crate for this :P
- Just want something a bit less verbose that the default syntax? Something
  very lightweight you can use in your rust code? Don't mind recompiling the
  whole game for each UI touchups? Then use [`cuicui_dsl`].
- Want quick iteration times with hot reloading, an advanced scene file format
  with templating, basically an actual scripting language? Ready to
  sacrifice some compile time for this? Then use [`cuicui_chirp`].

The [`cuicui_dsl` page] has a comparison matrix with [`cuicui_chirp`] to give
you a more detailed idea.

Note that the [`cuicui_layout_bevy_ui`] and [`cuicui_layout_bevy_sprite`] crates
have the `chirp` feature enabled by default. If you don't care for it, then
disable it with `default-features = false`.

Furthermore, the integration crates depend on `cuicui_dsl` unconditionally, as
it is a very lightweight dependency.

[`cuicui_chirp`]: chirp
[`cuicui_dsl`]: dsl
[`cuicui_dsl` page]: dsl/index.html#what-is-the-relationship-between-cuicui_dsl-and-cuicui_chirp
[`cuicui_layout`]: layout
[`cuicui_layout_bevy_sprite`]: https://docs.rs/cuicui_layout_bevy_sprite/0.9.0/cuicui_layout_bevy_sprite/index.html
[`cuicui_layout_bevy_ui`]: https://docs.rs/cuicui_layout_bevy_ui/0.9.0/cuicui_layout_bevy_ui/index.html
