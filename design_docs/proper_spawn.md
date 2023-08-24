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
