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
