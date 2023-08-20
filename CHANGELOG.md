# 0.9.0

## All DSL implementations (`LayoutDsl`, `UiDsl`, `SpriteDsl`)

**MAJOR**: The order of application of nested DSLs is now `Parent<Child>`
where `<Parent as DslBundle>::insert` runs before `<Child as DslBundle>::insert`.

The order matters when two different DSLs try to insert the same component.
The last DSL to be applied "wins". This is especially visible with the `LayoutDsl::ui`
method. Since it allows you to insert arbitrary component, it might conflict
with other DSLs.

Which order makes the most sense is arbitrary. I swapped it because in rust
syntax, the "outer" DSL is placed to the left of the "inner" DSL, so that
DSL insertion order happens left to right.

## `cuicui_layout`

* The `LayoutDsl::spawn_ui` method is now `LayoutDsl::ui`.

## `cuicui_chirp`

New crate! Define a custom file format, `.chirp`, to load from file `dsl!`s.
The syntax is very similar to that of `dsl!` with the exception of `code`.

* Define `ParseDsl` for deserialization of `.chirp` files.
* Using the `parse_dsl_impl` macro, you can convert a `DslBundle` impl block
  into a `ParseDsl` specification.

## `cuicui_dsl`

* Removed the `IntoEntityCommands` impl on `EntityCommands`
* `DslBundle::node` accepts now a `&mut EntityCommands`.
* `dsl!`: deprecated the field access syntax.
* **MAJOR**: Changed the meaing of leaf nodes!!!

Leaf nodes used to expand to:

```rust
// from
spawn_ui("Some text", "Name", width px(30));
// to
let mut x = <Dsl>::default();
let mut leaf_cmd = cmds.to_cmds();
x.named("Name");
x.width(px(30));
x.insert(&mut leaf_cmd);
x.spawn_ui("Some text", &mut leaf_cmd);
```

It now expands to:

```diff
 let mut x = <Dsl>::default();
 let mut leaf_cmd = cmds.to_cmds();
+x.spawn_ui();
+x.named("Some text");
 x.named("Name");
 x.width(px(30));
 x.insert(&mut leaf_cmd);
-x.spawn_ui("Some text", &mut leaf_cmd);
```

This means the signature of `spawn_ui` is expected to be:

```diff
- fn spawn_ui(&mut self, text: &str, cmds: &mut EntityCommands) -> Entity;
+ fn spawn_ui(&mut self);
```

### How to migrate leaf nodes?

1. Instead of inserting data in the `spawn_ui` method, update `self`  with
   the data that will be later read by the `DslBundle::insert` method and
   insert accordingly.
2. If you were using a non-object-safe generic parameter, you can use a
   `Option<Box<dyn FnOnce(&mut EntityCommands)>>` to store in `self` data
   you were directly spawning, then later call this function in `insert`.
3. Replace `spawn_ui("foo", bar, baz)` by `spawn(spawn_ui "foo", bar, baz)` in
   order to preserve the ability to pass an argument to the method.

You can check the diff of the commit where this change was applied for
inspiration.

Commit: [9932df5adb05f397aba570e7e11290446262d4b6]

[9932df5adb05f397aba570e7e11290446262d4b6]: https://github.com/nicopap/cuicui_layout/commit/9932df5adb05f397aba570e7e11290446262d4b6#diff-eebbb45eb1330ed943fde105912f9c71cb8af1f7b1dc49c832fd6f5b9204fe01

Sorry for the inconvinience. The goal is to make the library easier to understand
and use! The reasoning for this change is recorded in
`./design_docs/migrate_leaf_nodes.md`

# `cuicui_dsl` 0.8.1

* 53d6555 Improve `dsl!` error messages
  * Now, instead of the classic rust `macro_rules!` error messages, we emit
    `compile_error!` messages with context and links to documentation. This
    should make it much easier to use

# 0.8.0

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

# 0.7.0

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

# 0.6.0

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
