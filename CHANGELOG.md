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
