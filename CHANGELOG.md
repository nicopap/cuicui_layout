## `cuicui_dsl` 0.8.1

* 53d6555 Improve `dsl!` error messages
  * Now, instead of the classic rust `macro_rules!` error messages, we emit
    `compile_error!` messages with context and links to documentation. This
    should make it much easier to use

## 0.8.0

### `cuicui_layout_bevy_sprite`

* 331309d Add a Plugin to `cuicui_layout_bevy_sprite`
  * Before, you had to manually add every system, now `cuicui_layout_bevy_sprite`
    exports a plugin to do it for you.

### `cuicui_layout`

* 59ec3fa Split `LeafRule::Fixed` In two (#43)
* d6cceaf Rename PosRect to LayoutRect (#45)
* 3ed5e6f Use a marker component for compute_content_size (#35)
* 653e704 Improve content_sized error handling (#34)
  * This should cause error logs when returning `Nan` from a `ComputeContentSize` impl
  * Also when a content-sized node is orphaned while needing parent size
* bc40e49 Add world space handling
  * This fixes misalignement of the layout debug overlay for bevy_sprite
  * Note that layouting is bottom-to-top in the bevy_sprite implementation, this
    might change in the future
  * You can control whether the debug overlay is screen-space or world-space
    with the `cuicui_layout::debug::Options.screen_space` field
* bc40e49 Handle properly window scaling in debug overlay
  * Before, cuicui_layout's debug overlay assumed a windows scale of 1.5, now
    it is computed from the primary window
  * Might support heterogenous scale (multiple windows) in the future
* 8e454e8 Use a hashset to handle debug layout insets
  * Now the debug layout containers are inset pixel-perfectly so that outer container
    outlines are still visible.
* 2e98bf2 Move update_transforms from cuicui_layout to cuicui_layout_bevy_sprite

## 0.7.0

- Add the `cuicui_layout/debug` feature.
  - Enable it and press `Space` to have a debug overlay showing:
    - Node boundaries
    - Node margins
    - Whether nodes' size on give axis is relative to parent (outward arrows),
      children or content (inward arrows) or fixed (no arrows).
  - Pressing `Space` cycles between debug views, see the log output for details.
  - This is a very basic initial implementation
  - See [debug.md](https://docs.rs/cuicui_layout/latest/cuicui_layout/debug/index.html)
- rename `ui_debug` example to `bevypunk` and `sprite_mesh_debug` to `sprite_debug`.

## 0.6.0

- Clarify the "Using cuicui_layout" section of the README
- Added the following `LayoutDsl` methods:
  - `layout(&str)`
  - Add "combined" methods
    - `rules`: accepts `width` and `height` arguments
    - `margin`: set both cross and main margin to the same value
    - `margins`: accepts `main` and `cross` margin size arguments
    - `border_color`: The old `border` method is now `border`.
    - `border`: Now accepts a pixel width and a color, combining `border_color` and `border_px`
- Improved `LayoutDsl` defaults:
  - Now single child nodes are centered if `FillMain` distribution is used
  - The default `Rule` is now `Children(1.5)` instead of `Children(1.0)`
    it should make it easier to understand what is going on in a very basic
    setup.
