# Compute size from image/text

**Problem**: We want to integrate content-sized nodes into `LeafRule`,
but there is a few design issues at hand:

- We want to be able to compute width based on fixed or parent-based height
- We want computed height to be used for child-based container height
- We don't want to manage content-sized in `cuicui_layout`, but only in
  integration crates.

For text, height may depend on width. For images (we want to keep always
aspect ratio) content-sized on axis (a) will always depends on axis (b) if
it is set.


## What we can use

- width and height are fully independent in `cuicui_layout`
- Maybe use `ContentSized` as a component to communicate to layout algo
  proper size? (This is what bevy does, even if a bit awkward)
  - For images this is easy: store the aspect ratio & native pixel size
  - For text, we need access to text, computed size and `Res<Asset<Font>>` at
    the same time

`bevy_ui` solves this by pushing those into a closure stored in a `Box<dyn>`
inside a component.

It's very yickes.

Somehow we need to communicate the computed sized to a system, then run that
system, and finally compute again the size.

Consider that we are only doing this for terminal nodes

If non-content-sized axis is parent dependent,
then look up parent, compute size, then set the non-content-sized size.
It is correct.

Then, run the content-size system, setting the other axis.

Then, run the whole algorithm again.

### How the API should look like

Ideally, we remove the user the embarasment of having to access the `Node::Box`
(or `Node::Axis`), then check which axis is unset, read the other one
(reading the parent's size if necessary) and finally set the content-sized axis's
fixed value.

We could also directly write to `PosRect`, content-sized would be a unit variant
`LeafRule::ContentSized`.

- We would "simply" get the size from `PosRect`. The algo works with this,
  the change would only be in `Layout::node`

But right now it's easier to update the `Fixed.0`, so I'll do this and add
the refactor to the bug tracker.

#### Library `WorldQuery`

We expose a `WorldQuery` that does the work of computing the size, this way
it can be embedded in a query in a system defined by the user, the user
would add the system to an exposed set.

#### User `SystemParam`

Alternative is to provide a generic system generic over a `SystemParam`.
The `SystemParam` in question must implement a trait to compute
individual item sizes.

I'll opt for this, since I know it's possible to implement and I'm already
familiar with the pattern