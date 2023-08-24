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