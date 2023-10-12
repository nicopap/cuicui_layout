# `cuicui_layout`

[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)
[![Documentation](https://docs.rs/cuicui_layout/badge.svg)](https://docs.rs/cuicui_layout/)

`cuicui_layout` is a very primitive layouting algorithm implemented in bevy for bevy.

It is similar to CSS, but without the headache. The philosophy is:

> **You can always predict how it will look like**

`cuicui_layout` is fully independent from other cuicui crates, you can disable
all default feature and have a bare-bone plugin that only adds layouting components
and systems to your bevy game.

However, `cuicui_layout` also integrates with `cuicui_dsl` and `cuicui_chirp`.

See their respective documentation pages for why you'd want to use them as well.

- [`cuicui_dsl`]
- [`cuicui_chirp`]


## When to use `cuicui_layout`?

`cuicui_layout` is always a better choice over Flexbox, the default bevy UI
layouting algorithm. I'm however not claiming that it is better than other
similar non-flexbox layouting algorithm.

Here is some reasons you'd prefer `cuicui_layout` over other layouting
algorithms:

- Friendly algo with less things to keep in your head and good defaults.
- Uses and takes full advantage of the bevy ECS.
- Only controls `LayoutRect`, not `Transform`, you need to add a system that sets
  `Transform` based on `LayoutRect`.
- Fully flexible and extensible, can be used with `bevy_ui`, `bevy_sprite`, your own stuff.
- Helpful and fully detailed error messages when things are incoherent or broken.
  As opposed to FlexBox, which goes "this is fine 🔥🐶🔥" and leaves you to guess
  why things do not turn out as expected.
- This is a single-pass algo, so more efficient than flexbox.
- An extensive debugging overlay.

## How to use `cuicui_layout`?

### Cargo features

- **`debug`**: Enable the debug overlay
- **`reflect`** (default): Enable `bevy_reflect` impls for layout components.
- **`chirp`** (default): Enable [chirp][`cuicui_chirp`] [`ParseDsl`] implementation for [`LayoutDsl`]
- **`dsl`** (default): Define and export `LayoutDsl` [`DslBundle`] impl for the [`dsl!`] macro

### Layouting

`cuicui_layout` exposes the following [`Component`]s to control layouting:

- [`Node`]: A layout node, either a container holding other nodes as bevy
  [`Children`] or a leaf node.
- [`Root`]: The root of a node hierarchy. You may have several, all computations
  start from the root.
- [`ScreenRoot`]: If you add this component to a `Root` entity, it will keep
  the same size as the camera with the [`LayoutRootCamera`] component.

See the [`Rule`] and [`Container`] documentation for detailed explanation.

In short: a `Node` has independent [`Rule`]s on the `x` and `y` axis. When the
node is a [`Container`], it also has additional properties that manages how
children are distributed within the container.

Those additional properties are:

- [`Flow`]: The direction in which the children are distributed
- [`Alignment`]: Where on the cross axis are nodes aligned.
- [`Distribution`]: How to distribute the children of this container.
- `margin`: How much margin to put on main and cross axis

By default, items are aligned at the center of the container, distributed
on the flow direction evenly within the container.

A `Rule` tells the size of the `Node`, it can depend on the size of its children,
the size of its parent or be a fixed value.

There isn't more to it, that's pretty much all of `cuicui_layout`.
If this wasn't clear enough please read the [`Rule`] and [`Container`] documentation.

### Content-sized

It is possible to size leaf nodes based on components present on the same entity.

Use the [`content_sized`] traits to do that.

### Debugging

`cuicui_layout` has an integrated debugger. Enable it with the `cuicui_layout/debug`
cargo feature.

The debugger is an overlay displaying the extent of `Node`s and the direction
of their rules.

## Why not Flexbox

You are writing text to get 2d visual results on screen.
The translation from text to screen should be trivial, easy to do in your head.
Otherwise you need visual feedback to get what you want.
Bevy, even with hot reloading or [`bevy-inspector-egui`]
will always have extremely slow visual feedback.

Flexbox has too many parameters and depends on implicit properties of UI elements,
it is not possible to emulate it in your head.

cuicui's layout, in contrast to Flexbox is easy to fit in your head.
In fact, I will forecefully push cuicui's layout algorithm in your head
in two short bullet points.

- A node can be a `Node::Container` and distribute its children
  along a `Direction` either by evenly spacing them (`Distribution::FillMain`)
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

**Q**: Why not call it `padding` then?
<br>**A**: Look at the dictionary definition of "margin" and "padding".

**Q**: How do I center a node?
<br>**A**: nodes are centered by default, make sure the parent's container size
has the expected size.

**Q**: What is the equivalent of `flex_direction`?
<br>**A**: use `row` and `column`

**Q**: What are the equivalents of `column-reverse` and `row-reverse`?
<br>**A**: None. Use `Alignment::End` and swap your elements! Note that the `*-reverse`
flows in flexbox are very useful for internationalization. However,
when making a game, it is not enough to just swap the elements! Artistic control is
paramount and internationalization needs to be taken as a whole in the context of the UI.

**Q**: What is the equivalent of `flex_wrap`?
<br>**A**: None, do you really need it?

**Q**: What is the equivalent of `align_item`, `align_self`, `align_content`, `justify_content`?
<br>**A**: After 5 years of working with CSS, I still have no clue which one does what,
and whether they really do anything, so I won't adventure an asnwer.

**Q**: What is the equivalent of `flex_grow`, `flex_shrink`, `flex_basis`, `gap`?
<br>**A**: Do you even know what they do?

**Q**: Why can't child container overflow their parents?
<br>**A**: It's likely you didn't expect this, so we report it as an error.

**Q**: How do I make a grid?
<br>**A**: `cuicui_layout` is currently not capable of managing a grid of nodes.
This might be added in the future.

[`Alignment`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/enum.Alignment.html
[`bevy-inspector-egui`]: https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/index.html
[`Children`]: https://docs.rs/bevy/0.11/bevy/hierarchy/struct.Children.html
[`Component`]: https://docs.rs/bevy/0.11/bevy/ecs/component/trait.Component.html
[`Container`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/struct.Container.html
[`cuicui_chirp`]: https://lib.rs/crates/cuicui_chirp
[`cuicui_dsl`]: https://lib.rs/crates/cuicui_dsl
[`Distribution`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/enum.Distribution.html
[`DslBundle`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/trait.DslBundle.html
[`dsl!`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/macro.dsl.html
[`Flow`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/enum.Flow.html
[`LayoutDsl`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/dsl/struct.LayoutDsl.html
[`LayoutRootCamera`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/struct.LayoutRootCamera.html
[`Node`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/enum.Node.html
[`ParseDsl`]: https://docs.rs/cuicui_chirp/0.10.1/cuicui_chirp/parse/trait.ParseDsl.html
[`Root`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/struct.Root.html
[`Rule`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/enum.Rule.html
[`ScreenRoot`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/struct.ScreenRoot.html
[`content_sized`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/content_sized/index.html
