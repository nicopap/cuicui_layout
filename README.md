# Cuicui Layout

A dumb layout algorithm you can rely on, built for and with bevy.

## Running examples

Use the `cargo run --bin` command to list possible examples, and run them.

We do this because it allows us to have different dependencies between examples.

## Stability

This crate is in expansion, use at your own risk, it is extremely likely that
a lot of things are going to break a lot.

## Using `cuicui_layout`

1. Chose which crate you want to use:
  - Interested in a ready-made UI library on top of `bevy_ui`? [`cuicui_layout_bevy_ui`] is for you!
  - Want more flexibility? Using `cuicui_layout` on top of `bevy_sprite` will let you
    integrate your UI with a lot of 3rd party crates that only work with sprites!
    [`cuicui_layout_bevy_sprite`] is for you!
  - Using a custom renderer or want your UI to be part of the 3D environment?
    Build on top of [`cuicui_layout`] itself then!
1. Add the chosen crate as a dependency to your crate.
2. Use either the [`CommandsLayoutExt`] trait or [`layout!`] macro to build
   a UI (text representation coming soon). The macro is just a thin wrapper
   around the trait, and the trait itself is very easy to use, so your choice.
3. That's it! You are now using `cuicui_layout`, congratulations!
   Make sure to check the [`CommandsLayoutExt`]
   docs to learn the current capabilities of `cuicui_layout`.

## `cuicui_layout` crates

This repository contains several crates:

- `cuicui_layout`: The base algorithm and components, does not make any assumption
  about how it is used, beside the requirement that layout nodes be bevy `Entitiy` and
  uses `bevy_hierarchy` and `bevy_transform`.
- `cuicui_layout_bevy_ui`: Integration with `bevy_ui` and the nice layout DSL.
- `cuicui_layout_bevy_sprite`: More bare-bone `bevy_sprite` integration.

(maybe `cuicui_layout_spec` in the future)

## Why not Flexbox

You are writing text to get 2d visual results on screen.
The translation from text to screen should be trivial, easy to do in your head.
Otherwise you need visual feedback to get what you want.
Bevy, even with hot reloading or [`bevy-inspector-egui`]
will always have extremely slow visual feedback.

Flexbox has too many parameters and depends on implicit properties of UI elements,
it is not possible to emulate it in your head.

cuicui's layout in contrast to Flexbox is easy to fit in your head.
In fact, I will forecefully push cuicui's layout algorithm in your head
in two short bullet points.

- A node can be a `Node::Container` and distribute its children
  along a `Direction` either by evenly spacing them (`Distribution::FillParent`)
  or putting them directly one after another (`Distribution::Start`).
- A `Container`'s size can be expressed as a static value, a fraction
  of the size of what contains it, or a multiple of what it contains.
- The content of a `Container` can be `Alignment` to the start, end or center
  of its parent (by default it's centered).

That's it. There are some edge cases, but cuicui will ~~yell at you~~
tell you nicely when you hit them and tell you how to handle them properly.

### Flexbox FAQ

**Q**: Where is `padding`?
<br>**A**: `padding` is equivalent to `margin` in cuicui_layout. `margin` and `border`
doesn't make conceptual sense.

**Q**: How do I center a node?
<br>**A**: Add an empty node at the start and end of the container, and use `fill_parent`

**Q**: What is the equivalent of `flex_direction`?
<br>**A**: use `row` and `column`

**Q**: What are the equivalents of `column-reverse` and `row-reverse`?
<br>**A**: None. Use `Alignment::End` and swap your elements! Note that the `*-reverse`
flows in flexbox are very useful for internationalization, as cultures that are not
western have different understanding of where is the start and end of things. However,
when making a game, it is not enough to just swap the elements! Artistic control is
paramount and internationalization needs to be taken as a whole in the context of the UI.

**Q**: What is the equivalent of `flex_wrap`?
<br>**A**: None, do you really need it?

**Q**: What is the equivalent of `align_item`, `align_self`, `align_content`, `justify_content`?
<br>**A**: After 5 years of working with CSS, I still have no clue which one does what,
and whether they really do anything, so I wont' adventure an asnwer.

**Q**: What is the equivalent of `flex_grow`, `flex_shrink`, `flex_basis`, `gap`?
<br>**A**: Do you even know what they do?

**Q**: Why can't child container overflow their parents?
<br>**A**: It's likely you didn't expect this, so we report it as an error.

## Why cuicui layout

On top of the very friendly layout algorithm,
cuicui runs on `bevy_ecs` and therefore can ~~abuse~~ use it as a backing storage.

Layouts are generally backed by a tree,
[`taffy`]'s implementation of Flexbox internally uses a [`slotmap`].
cuicui uses the ECS, which is basically a faster slotmap.

Also, cuicui's layouting system relinquishes control to give more power to users.
Meaning that you can tell cuicui to not manage UI entities `Transform`
and instead chose yourself to build the UI based on what info cuicui gives you.

### Limitations

cuicui layout returns postion as offset from parent, which may not be useful
if you do not use bevy's transform hierarchy. This also locks you into using
bevy hierarchy for your Ui.

## Understanding this repository

This repository at <https://github.com/nicopap/cuicui_layout> contains several
packages. One root package and further integration packages:

- The root package, `cuicui_layout` defines a layouting algorithm that works
  on its own defined components. It doesn't even hard-code to update `Transform`.
- `ui` contains integration with the bevy-native UI framework, `bevy_ui`
- `sprite` on the other hand, integrates with bevy's 2D renderer, `bevy_sprite`.

 
## TODO

Before first release:

- [ ] Integrate Change detection
- [X] Basic algorithm
- [X] In depth documentation explaining the algorithm
- [X] Meaningfull error messages when algorithm hits circular constraints
- [X] Ergonomic macro to define a UI tree
- [ ] Margin as `Container` property.
- [X] Alignment & Distribution

After first release:

- [ ] bevy_ui: background image and background color of row/column nodes.
- [ ] layout error: wonky (ie: any `?` in `layout` function)
- [ ] layout error: give size of largest node when overspill
- [ ] layout error: compute relative size and raise error based on this when
      several sibling have a `Rule::Parent`.
- [ ] Consider not making macro dependent on bevy_ui
- [ ] Better {ui,sprite}/debug module (using gizmos)
- [ ] Write a tool to make and export layouts.
- [ ] Separate the algo into its own crate independent of bevy
- [ ] Typed constructor
- [ ] (questionable) Accumulate errors instead of early exit.
