# Showing text on the debug overlay

Requirements:
* Fast: ideally only allocation on spawn
* Precise: Need to be placed correctly according to non-local rules
* maybe/immediate: Should somehow integrate with the immediate mode API.

## How?

1. `Text2dBundle` (sprite) on the debug layer
2. A system similar to `bevy-debug-text-overlay`, but keyed by block entity

## Not so fast

Well, the bevy rendering setup makes it very difficult to avoid creating
and cloning strings. Initial draft will be heavy on allocations.

I think a fast (yet convinient) debug text renderer without allocation is out
of scope for this, even if possible.

## Offseting

Goal is to avoid two text boxes from overlapping.
But how?

My first though "sounds like a spatial optimization datastructure problem"!

But not so fast. Consider this: All text boxes have the same size (because
the character count is set)

What if we split the screen in N regions of fixed size of text boxes. Then
when we add a text box, we set the "pixel" in the region grid to the offset
from the region's center.

==> Too complex, not in scope

What about a KD tree using the `kd-tree` crate?

==> Requires rebuilding every time an object is added. A bit limiting when we
are adding all the text each frame!

What about a naive iteration?
