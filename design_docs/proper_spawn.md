# Proper handling of scenes

## Problem

The `Handle<Scene>` setup for bevy doesn't work with `.chirp` files because it
requires a "parent" `Entity`. But the correct set of components to give to
the parent is unclear due to the genericity of the library.

I've a few options at hand:

- Add a method to `DslBundle` to insert a "default set of components" for a
  root node. Would still ned a wrapper around `SceneBundle` to add the relevant
  component somehow.
- Do not rely on `Handle<Scene>`, define your own handle type that interally
  uses `Scene` (since `bevy_scene` let you spawn scenes at arbitrary locations,
  it is doable)

My hunch is we are better off with a custom loader. In addition, it will allow
me to have more control over the spawned entities and inject hot-reloading &
serialization into the mix.

## How `bevy_scene` does it

Has two systems:

- `scene_spawner`: Reads `Handle<Scene>`, `Handle<DynamicScene>` and queues scene
  spawning in `ResMut<SceneSpawner>`.
- `scene_spawner_system`: exclusive world access; Calls a series of functions
  that for each queued scene try to spawn them, if fails, re-queue them for
  next frame. (second `for` loop in `SceneSpawner::spawn_queued_scenes`)

### And `bevy-scene-hook::reload`?

Checks if instance is ready through `SceneSpawner::instance_is_ready`, adds
logic on top.

When instance is ready calls `SceneSpawner::iter_instance_entities`, runs hook
for each entity.

`SceneSpawner::iter_instance_entities` uses the `InstanceId` to get the interally
kept `EntityMap`.

## How to spawn the scene?

So `SceneSpawner` always take a `Handle<Scene>`. Ideally, we trust it to do
things correctly. `Chirp` would then just be a `struct Chirp(Handle<Scene>)`,
not even `struct Chirp(Scene)`?

Then in `spawn::chirp` we would just call `SceneSpawner`, store `InstanceId`,
call `instance_is_ready` repetitively in a system (called `chirp_hook`)
in `PostUpdate` after `scene_spawner_system`.

`chirp_hook`:
- Look at entities with `Handle<Chirp>`
- Get the `Chirp` for the handle.
- `Chirp` contains a field with the `InstanceId`, checks in `SceneSpawner` if
  the `Chirp`'s `Handle<Scene>` "is ready"
- If it is ready, we run the hook for the entities in the scene with
  `iter_instance_entities`. Note that if the seed was spawned as a child, we
  need to set the parent of all the root entities (and ideally, that, before the
  transform prop system)
- Need to store the `InstanceId` outside of `Chirp`, because `Chirp` may be
  duplicated per instance.

```rust
app.add_systems(PostUpdate,
  (chirp_hook.after(scene_spawner_system), apply_deferred)
    .chain()
    .before(TransformSystem),
);
```

Open questions:

- Ideally we control scene reloads through a component. We don't have a single
  entity to associate this component with. So we need some ad-hoc registry
  in a `Res`.

# Current architecture

Description of flow is available in `chirp/src/loader/mod.rs`. But our approach
is currently broken:

`ChirpInstances` holding `HashMap<HandleId, ChirpInstance>` doesn't work for when
we have multiple instances of the same scene. Ideally we want to store instance
info per-instance (duh…). To do so we will:

Use a `HashMap<Seed, ChirpInstance>`: where `type Seed = Entity;` The `Entity`
used to spawn a specific instance.

User always has acccess to the seed since they have control over the.

This might become more complicated as scenes themselves may contain other
scenes. But that's for future considerations.

User-controlled stuff:

- Add an `Entity` with `Handle<Chirp>`
- Manually remove/re-queue loading of a specific instance through `ChirpInstances`

We want:

- When getting `AssetEvent`s, to reload/delete entity from **all scenes** which `Handle<Chirp>` changed.
- When finding a `Handle<Chirp>`, to load the chirp's scene and add it to `ChirpInstances`.
- When `Scene`'s instance is ready, to do our hooking stuff

# New architecture

It was supposed to simplify things:

1. Load scene
2. Read all components on the root of the scene, store it into a closure in `InsertRoot`.
3. Remove the parent component on children of root and despawn root in scene world.
4. Add the root components to the seed entity and use SceneSpawner to spawn the
   rest of the scene as child of the seed entity
5. wait until scene is spawned (as child of the seed entity) and mark it as loaded

(2) is very complex and error-prone.

The advantage is that scenes are added in-place:

- we can re-use the spawned entity id as a reference for the scene instance.
- also fixes any issue with other entites referencing the thing we just spawned
- the scene keeps the exact seed location in the parent's children list.

## Current architecture loses info on root entities.

1. Can't access the scene entity map, so ReflectEntityMap components on the root
   entity are discarded
2. Handles that only exist on the root entity are dropped through `.clone_value()`.

I'm getting a lot of errors I'm not sure where they are coming from.

# New New architecture

copy/paste bevy scene (well we already did for `InsertRoot`)

Fun thing is loaded chirp implies the `Handle<Scene>` inside is also loaded so
we are good on that front.

But now we are down to requiring exclusive world access for scene spawning, so
how to manage that?

1. Store `(Entity, Handle<Scene>)` in a `Vec`
2. Extract the `Scene` from the `Assets<Scene>`
3. spawn scene using `insert_on`
4. Re-add the scene to the world

## Preserving root components

We setup things properly, it kinda works. But there is still a major limitation.

**Problem**: `EntityMap` rewrites the pre-existing components on the root entity
with bogus new entities.

**Problem**: When respawning, we clear the pre-existing components on root
entity.

**Problem**: We still get the lots of warning from having parent without visibility.
Is it coming from the root?

I'm extremely surprised that the first draft turned out to be so robust, and I've
been spending 3 days right now trying to refine an alternative approach into
something useable.

### Solution

1. Track which components the target root has
2. when spawning scene:
  1. Remove from target root all those components
  2. Remove from that list all components that are overwritten by the source scene root
  3. Apply `EntityMap`.
  4. Insert back the target root components left on the list.
3. Store in the `ChirpInstace` that list
4. When respawning:
  1. Iterate over `Archetype`'s component_id, removing the ones from the instance list
  2. Use the innexisting `remove_by_ids` method to remove those TODO(BUG)¹

¹bug: Say we insert a `NodeBundle`, now our root entity doesn't have a node bundle
anymore, so we may be getting the "non-UI child of UI" panick, which sucks.

We might want to track separately overwritten component and discard that information
at the end of the scene spawning routine.

### Innexisting `remove_by_ids` method

1. Should return a `MovedErasedBundle` which has a private `consumed` field
   set to `false`
2. `EntityMut` should have a method that re-inserts the bundle, consuming the
   `MovedErasedBundle`, setting the `consumed` to `true` before dropping.

```rust
struct MovedErasedBundle {
  // NOTE: care must be taken to make sure the data of each individual component is aligned.
  data: Box<[u8]>,
  // (offset_in_data, id)
  components: Box<[(usize, ComponentId)]>,
  consumed: bool,
}
```

nah, tried and it's too complex.

Let's try something different.
