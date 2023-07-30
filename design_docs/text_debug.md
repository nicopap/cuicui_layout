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
