# `cuicui_layout` Content sized nodes

Leaf nodes (nodes that do not contain other nodes) may be "content-sized".

For example, in `cuicui_layout_bevy_ui`, a leaf node containing an image may
keep the same aspect ratio as the image.

## How to define content-sized elements?

First off, if you are already using `cuicui_layout_bevy_ui` or
`cuicui_layout_bevy_sprite`, **you don't need to do anything**, those plugins
already take care of elements that should depend on the size of their content.

If you need to implement content-sized elements for your own UI stuff, you will
need to:

1. Define a [`SystemParam`] (we will refer to it as `MyContentSize`)
2. Implement [`ComputeContentParam`] for `MyContentSize`
    - In [`ComputeContentParam::Components`], tell which components are used to
      tell content size. Use [`AnyOf`] if several.
    - In [`ComputeContentParam::condition`], tell when the size update system should run
3. Implement `ComputeContentSize` for `MyContentSize`. [`ComputeContentSize::compute_content`]
     is ran for each leaf node [`Entity`] with the provided components.
    - The sizes infered by the layouting algorithm is passed as the `set_size`
      parameter.
    - The return value is the sizes as they should be, based on the passed `components`
    - Note that the non-content-sized axis will always keep the pre-set size, regardless
      of the return value.
4. Register `MyContentSize` as a content sized element computation using
   [`app.add_content_sized::<MyContentSize>()`][`AppContentSizeExt::add_content_sized`].

And that's it!

The two distinct traits are required due to a limitation in the rust type system.
Trying to merge the two traits came close to unleashing Cthulhu into the world.
Do not ask me to merge them, do not open an issue for merging them, this way
lies madness.

## Example

The best examples are the `content_sized.rs` modules in `cuicui_layout_bevy_ui`
and `cuicui_layout_bevy_sprite`.

Please take a look at them to get an idea of the kind of code you need to write.

[`AnyOf`]: https://docs.rs/bevy/0.11/bevy/ecs/prelude/struct.AnyOf.html
[`AppContentSizeExt::add_content_sized`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/content_sized/trait.AppContentSizeExt.html#tymethod.add_content_sized
[`ComputeContentParam`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/content_sized/trait.ComputeContentParam.html
[`ComputeContentParam::Components`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/content_sized/trait.ComputeContentParam.html#associatedtype.Components
[`ComputeContentParam::condition`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/content_sized/trait.ComputeContentParam.html#tymethod.condition
[`ComputeContentSize::compute_content`]: https://docs.rs/cuicui_layout/0.10.1/cuicui_layout/content_sized/trait.ComputeContentSize.html#tymethod.compute_content
[`Entity`]: https://docs.rs/bevy/0.11/bevy/ecs/prelude/struct.Entity.html
[`SystemParam`]: https://docs.rs/bevy/0.11/bevy/ecs/system/trait.SystemParam.html
