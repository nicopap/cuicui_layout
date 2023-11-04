# `cuicui_layout` debug view

`cuicui_layout` has a "debug" mode. It gives a visual representation of container
and node sizes.

![A screenshot of the cyberpunk menu with container outlines](https://user-images.githubusercontent.com/26321040/272255534-4cb44a1f-09c9-414e-870c-f5ebc3a468f6.jpg)

## Limitations

- While the debug overlay is up, gizmos cannot be used by other plugins
- This is only tested with `cuicui_layout_bevy_ui` and `cuicui_layout_bevy_sprite`
  (I can't implement a debug view for your personal custom UI :P)
- The debug overlay use the bevy [`RenderLayers`] nº16 and camera order 255 to draw gizmos

## How to use the debug view?

Enable the `cuicui_layout/debug` cargo feature.

```sh
cargo run --features cuicui_layout/debug
```

### Debug view mode

There are several view modes, you cycle through them by pressing the space bar:

- **nothing** (default): There is no additional informations displayed
- **outlines**: Displays the outline of each [`Container`] and [`Node`] visible
  on screen, with a different color
- **outlines and rules**: In addition to the outline, display each node's vertical
  and horizontal [`Rule`]s.
  - Arrows pointing outwards left and right mean the horizontal size (width) depends on
    the size of the parent of the node.
  - Arrows pointing inwards left and right mean the horizontal size (width) depends on
    the size of the children of the node.
  - The absence of arrows indicate the node has a fixed size.
  - Arrows going up and down indicate the rules for the vertical size (height).

### Debug view configuration

The debug view can be programmatically manipulated using the [`Options`] [`Resource`].

#### Display invisible layouts

The debug view does not display information about `Container`s with
a `ComputedVisibility` component returning `vis.is_visible() == false`.

Set the [`Options.show_hidden`] field to `true` to display outlines even if the
`ComputedVisibility` is `false`.

#### Change/Remove the cycling key

Maybe your game makes heavy use of the space key (I've heard that some plateformers use
the space key for a common action, would you belive it?) and you don't want to cycle
through the debug views each time space is pressed.

You can set the [`Options.input_map`] value to something else:

```rust
#[cfg(feature = "cuicui_layout/debug")]
fn debug_toggle(mut opts: ResMut<cuicui_layout::debug::Options>) {
  opts.input_map.cycle_debug_flag = KeyCode::X;
}
```

#### Invert Y axis direction

Confusingly, `bevy_ui` has a downward Y axis, while `bevy_sprite` has an upward
Y axis.

You can configure what Y axis direction the debug overlay uses by setting the
[`Options.screen_space`] field.

If you are using `cuicui_layout_bevy_ui`, this should be automatically set to
`true` for you.


[`Container`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/struct.Container.html
[`Node`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/enum.Node.html
[`Options`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/debug/struct.Options.html
[`Options.input_map`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/debug/struct.Options.html#structfield.input_map
[`Options.screen_space`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/debug/struct.Options.html#structfield.screen_space
[`Options.show_hidden`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/debug/struct.Options.html#structfield.show_hidden
[`RenderLayers`]: https://docs.rs/bevy/0.12/bevy/render/view/struct.RenderLayers.html
[`Resource`]: https://docs.rs/bevy/0.12/bevy/ecs/prelude/trait.Resource.html
[`Rule`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/enum.Rule.html
