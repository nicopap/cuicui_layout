# Cuicui Layout

cuicui defines its own layout algorithm.

### Why not Flexbox

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
  along a `Direction` either by evenly spacing them (`Stretched`)
  or putting them directly one after another (`Compact`).
- A `Container`'s size can be expressed as a static value or a fraction
  of the size of what contains it.

That's it. There are some edge cases, but cuicui will ~~yell at you~~
tell you nicely when you hit them and tell you how to handle them properly.

### Why cuicui layout

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

## TODO

- [X] Basic algorithm
- [X] Typed constructor
- [X] In depth documentation explaining the algorithm
- [X] Meaningfull error messages when algorithm hits circular constraints
- [ ] Ergonomic macro to define a UI tree
- [ ] Alignments
  - [ ] Cross axis alignment (aka alignment): for Horizontal, should items be aligned to the:
    - top
    - bottom
    - middle
  - [ ] Main axis alignment (aka distribution/justification): for Horizontal, should items be:
    - compactly pushed at the start
    - compactly pushed at the end
    - spaced evenly so that the first is at the start and last at the end
- [X] `ChildDefined(how_much_larger_than_child)`
- [ ] API cleanup
- [ ] Define a parametrable plugin to add smoothly the layout systems to app
- [ ] Integrate Change detection
- [ ] Accumulate errors instead of early exit.
- [ ] Write a tool to make and export layouts.
- [ ] Separate the algo into its own crate independent of bevy
