# Using change detection to minimize layout recomputation

**Problem**: currently, each frame, the whole hierarchy of layout is traversed.

Scandalous! And then we pretend to be green.

But what would look like an algorithm that avoids re-computing unchanged values?

The difficulty is **some node depend on size of children, some of their parent
some both (different for each axis)**.

We also might depend on **deeply nested** element value change (say container
is `Rule::Children` and contains a `Rule::Children` which itself etc...)

How can we effectively prune and/or avoid branches without updates while not
navigating to them? It seems contradictory.

## The 3 phases algorithm


Currently, we have a two-phase algorithm within a single system:

1. Deeply traverse the hierarchy, setting the size of each traversed node
2. Once all children size's is known (since we traverse hierarchy from root, we
   always know the parent's size) we compute the offsets of each children
   (we need each child's size to compute offsets). We return our own size.

More precisely, we compute the offset at each node, before returning, this
avoids two full traversals, but still has two phases.

Things that can change:

- Number of children of a `Node`
- saddly, parent of a `Node`
- the node rules themselves, which direction a node depends on

Things we compute:

- `PosRect.pos` and `PosRect.size`.

A `Node`'s size may depend on:

- Parent width
- Parent height
- Children width
- Children height

A `Node`'s size that depends on XXX will need to be recomputed when:

- Parent width: parent changed, parent's width changed.
- Parent height: parent changed, parent's height changed.
- Children width: added/removed child, one of the children's width changed.
- Children height: added/removed child, one of the children's height changed.

On the `Children` case, it's important to note that **on the cross axis**,
the parent's size change only if the **max size** child changed.

Or when it changed what it depends on.

Consider marker components with `storage = "SparseSet"`:

```rust
struct ReadsParent__Height;
struct ReadsParent__Width;
struct ReadsChildrenHeight;
struct ReadsChildrenWidth;
```

We can setup/update which marker components a `Node` has on `Changed<Node>`
guarenteed!

Maybe this sucks? Should we rather mark entities which when updated will cause
their dependents to be updated?

```rust
struct UpdatesParent__Height;
struct UpdatesParent__Width_;
struct UpdatesChildrenHeight;
struct UpdatesChildrenWidth_;
```

**children dependent**: if any of the child's size change, then its size will change.
But need to know in aggregate the size of all children to update own size.
(unless we keep track of the previous size)

previous-size tracking would enable:

- Update self, if parent is dependent on self, check old size, if
  size change is on cross axis, then check whether we were the max or became
  the max (if container is not AlignStart, this requires updating all
  children's position as well) if on main axis, add (new - old) to parent size, update
  position of siblings following self.
- Update self, if child is dependent on self, just update it
  Since it would be honerous to keep track of which child is dependent on self
  we traverse all children, check if they are dependent on self and update them
  only then(a potential solution would be to use a u64 bitset)

`Rule::Parent` is annoying to deal with: How do I efficiently store which of
the children is dependent on self?

### What about position dependencies

Position depends on parent's alignment and distribution, but also the size
of siblings.

- Alignment::{End,Center}: changes based on cross axis of parent's size
  and parent's margin
- Alignment::Start: is always 0
  and parent's margin
- Distribution::{Start}: changes based on main axis previous sibling's size
  and parent's margin
- Distribution::End: changes based on main axis next sibling's size,
  and parent's size
  and parent's margin
- Distribution::FillMain: changes based on main axis all sibling's size,
  and parent's size
  and parent's margin

### Hmmduff

List of tricks:

- Keep track of old size for entities children of a `ReadsChildren{W,H}`
  Allows to not have to go downward (unless self becomes not max)
- We know that a direct child of X that is `ReadsChildren{W,H}` cannot be
  `ReadsParent{W,H}` on the same axis. ie: child of X either depends on size
  of its own children or nothing.
  This means that a single pass propagation from bottom to top (and top to
  bottom for the entites with a `ReadsParent` dependency)

Ideally, the algo should be fully independent of where in the hierarchy we start,
so that we can parallelize it/reduce complexity.

Ideally, it's a single pass deal too.

But I don't think it's possible, so let's design an iterative algorithm

**Setup markers**:
- Read all `Changed<Node>`, `Changed<Parent>` and `Changed<Children>`
- Set accordingly the `Updates{Parent,Chi}{W,H}` of their parent/children
- Set accordingly the `Reads{Parent,Chi}{W,H}` of the changed entity.

**Setup change markers**:
- Mark all entities with one of the `Updates{Parent,Chi}{W,H}` components
  **and** `Changed<Node>` with the `NeedsRecompute{W,H}` component

The tricky bit here is to order the recomputation such as change propagate from
Updates -> Reads, and the recomputed value is never overwritten.

**Recompute leaf nodes**:
- `NeedsRecomputeW` **and** `UpdatesParentW` **and not** `ReadsChildrenW`

```
row("root", width child 1.0) {
  spawn_ui(image0, "leaf0");
  column("c1", width child 1.0) {
    spawn_ui(image1, "leaf1");
    row("c2", width child 1.0) {
      spawn_ui(image2, "leaf2");
    }
    spawn_ui(image3, "leaf3");
  }
}
```

- "leaf0" updates its width: Since it's `UpdatesParentW`, we update the parent's
  width "c1".
- Since we updated "c1" and it's `UpdatesParentW`
