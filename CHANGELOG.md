# 0.10.0

## Minor breaking changes:

- `cuicui_chirp`: the `wraparg` module is now `parse_dsl::args`
- `cuicui_chirp`: Removed `ChirpInstances`, now that the root entity is preserved,
  you can directly interact with it instead of using a resource. f

## `cuicui_dsl` & `cuicui_chirp`: Major DSL syntax change

The DSL syntax has gotten a lifting!

- All separators got removed.
- The "name literal" method syntax has been removed
- The "single arg" method syntax has been removed
- The deprecated field access syntax has been removed
- The "leaf node" statement syntax is now an alias for entity `Name`, it is
  not a method call anymore. (this replaces the name literal syntax)
- Only a single root entity is allowed per chirp/dsl! declaration

Let's take a look at the old syntax:

```ron
// OLD SYNTAX, not valid anymore
row(screen_root, "root", main_margin 100., align_start, image "images/background.png") {
    spawn(button "Button text 1", color Color::BLUE, width px(40), height pct(100));
    spawn(button "Button text 2", color Color::RED, width px(40), height pct(100));
    column("menu", fill_main_axis, image "images/board.png") {
        spawn("Title card", height px(100), width pct(100));
    }
}
```

This should be translated to:

```ron
// new syntax
// `root` also works, but we now recommend entity names to be uppercased
// Notice that `row` is moved at the very end of the method list.
Root(screen_root main_margin(100.) align_start image("images/background.png") row) {
    spawn(button("Button text 1") color(Color::BLUE) width(px(40)) height(pct(100)))
    // `spawn` still spawns an entity without a name, but I recommend switching to
    // `Entity`, which will also spawn an entity without a name.
    Entity(button("Button text 2") color(Color::RED) width(px(40)) height(pct(100)))
    Menu(fill_main_axis image("images/board.png") column) {
        TitleCard(height(px(100)) width(pct(100)))
        // This is allowed if you want to preserve the space in the entity Name:
        "Title card"(height(px(100)) width(pct(100)))
    }
}
```

Is this an improvement? My subjective opinion is that it's not that much better.
But we got rid of a lot of concepts.

- "name literals" were mixing in with method calls, which could be confusing
  and a bit too magic.
- the unified method call syntax, similar to a rust method call, might help
  understand better what a "method" is (ie: a method on the `DslBundle`)
- The fact _there is only methods_ in method position should also help.
- Removing the quirky method outside of the method list should help understand
  the syntax better.
- Prefixing entity with their name is neat, and it reflects the way
  `bevy-inspector-egui` works.
  Entities can only have a single name, so it also makes sense ot use them
  as the "introduction" modifier
- Have you ever wondered whether you should or not add a `,` or `;`? Well, without
  separators, this isn't a problem anymore. No "automatic `;` insertion" required,
  it's just how the grammar works.
- Spawning a single entity enables a lot of fixes to `cuicui_chirp` and makes
  templating a bit more principled.

Less concepts should make it easier to learn and pick up.

### Root entity change

Files with several root entities won't work anymore:

```ron
// This will fail:
Red(row color(Color::GREEN))
Green(row color(Color::BLUE))
Blue(row color(Color::RED))
```

To fix this, wrap them in a single root entity:

```ron
// Now interprets correctly:
Colors {
    Red(row color(Color::GREEN))
    Green(row color(Color::BLUE))
    Blue(row color(Color::RED))
}
```

## `cuicui_dsl`: Change in behavior of the `code` statements

Now that statements require to be associated to a single entity, `code` statements
are now much less powerful.

Previously, you would write:

```rust
// OLD SYSTEM: does not work anymore
dsl! {
  &mut cmds.spawn_empty(),
  Root(row screen_root main_margin(100.) distrib_start align_start image(&bg)) {
    Menu(rules(px(310), pct(100)) main_margin(40.) image(&board) column) {
      TitleCard(image(&title_card) width(pct(100)))
      MiniTitleCard(ui(title_card) width(pct(50)))
      // This will only create a single button
      code(let cmds) {
        for n in &menu_buttons {
          let name = format!("{n} button");
          dsl!(cmds, Entity(ui(*n) named(name) image(&button) height(px(33))));
        }
      }
    }
  }
};
```

Now, the `cmds` passed to `code` is always an empty `EntityCommands`, you must
do with it, there isn't really way to reproduce some of the old behaviors. Here
is how I solved it for `simple_menu`:

```rust
dsl! {
  &mut cmds.spawn_empty(),
  Root(layout(">dSaS") screen_root main_margin(100.) image(&bg)) {
    Menu(rules(px(310), pct(100)) main_margin(40.) image(&board) column) {
      TitleCard(image(&title_card) width(pct(100)))
      TitleCard2(ui(title_card) width(pct(50)))
      code(let cmds) {
        // Create a "Buttons" container and add the buttons as individual children
        dsl!(cmds, Buttons(column height(child(2.)) width(pct(100))));
        cmds.with_children(|cmds|{
          for n in &menu_buttons {
            let name = format!("{n} button");
            dsl!(
              // You'll notice this works the exact same way as the root invocation.
              &mut cmds.spawn_empty(),
              Entity(ui(*n) named(name) image(&button) height(px(33)))
            );
            }
        });
      }
    }
  }
};
```

## `cuicui_dsl`: Remove the `IntoEntityCommands` trait

Now that dsl declarations MUST spawn a single entity, the `cmds` argument
can only be a `&mut EntityCommands`. The components of the root entity will
be added to the entity of the `EntityCommands`.

## `cuicui_chirp`: Add `fn` statements

It is now possible to define templates in `chirp` files.

Use `fn` to define a template. To use them, call them like you would call a
rust macro (with a trailing `!`):

```ron
fn my_template() {
    Colors {
        Red(row color(Color::GREEN))
        Green(row color(Color::BLUE))
        Blue(row color(Color::RED))
    }
}
fn template_with_arguments(fantasy_color) {
    FantasyColor(row color(fantasy_color))
}
Palette {
    // Use a `fn` template
    my_template!()
    // Use a `fn` template with arguments
    template_with_arguments!(Color::PINK)
}
```

Currently only `fn` defined within the same file are recognized.

## `cuicui_layout`: Add `Distribution::Overlap`

With this distribution mode, all children start at the beginning of
this container. They are not "distributed", they overlap.

## Add `cuicui_layout::debug::Options::show_hidden`

Now `cuicui_layout`'s debug view do not draw outline of invisibile containers by default.
To control this new behavior, toggle the `show_hidden` field. It is `false`
by default. Setting it to `true` will show the outline of hidden containers.

# 0.9.0

## All DSL implementations (`LayoutDsl`, `UiDsl`, `SpriteDsl`)

**CHANGED**: The order of application of nested DSLs is now `Parent<Child>`
where `<Parent as DslBundle>::insert` runs before `<Child as DslBundle>::insert`.

The order matters when two different DSLs try to insert the same component.
The last DSL to be applied "wins". This is especially visible with the `LayoutDsl::ui`
method. Since it allows you to insert arbitrary component, it might conflict
with other DSLs.

Which order makes the most sense is arbitrary. I swapped it because in rust
syntax, the "outer" DSL is placed to the left of the "inner" DSL, so that
DSL insertion order happens left to right.

## `cuicui_layout`

* **CHANGED**: The `LayoutDsl::spawn_ui` method is now `LayoutDsl::ui`.

## `cuicui_chirp`

**ADDED**

New crate! Define a custom file format, `.chirp`, to load from file `dsl!`s.
The syntax is very similar to that of `dsl!` with the exception of `code`.

* Define `ParseDsl` for deserialization of `.chirp` files.
* Using the `parse_dsl_impl` macro, you can convert a `DslBundle` impl block
  into a `ParseDsl` specification.
  * Use the various `parse_dsl_impl` attributes to control how arguments are
    parsed from the text file.
* `IntoUiBundle` now requires `Reflect`! This let us use it with the chirp
  file format.
* `ParseDsl` is implemented for all DSLs exported by cuicui crates.
* `ReflectDsl<B>` implements `ParseDsl` and `DslBundle` for any `B: Reflect + Bundle`
  * Note that unlike other DSLs this only works for `.chirp` files.
* To spawn a `.chirp` scene, you do: `commands.spawn(asset_server.load::<Chirp,_>("scene.chirp"))`
  * All scene "root" entities are added as sibling of the `Entity` with a `Handle<Chirp>`
  * The spawning mechanisms are thought out to be as invisible as possible.
  * Hot reloading works as expected
* The parser & interpreter reports errors in a user-friendly way.

## `cuicui_dsl`

* **CHANGED** Removed the `IntoEntityCommands` impl on `EntityCommands`
* **CHANGED** `DslBundle::node` accepts now a `&mut EntityCommands`.
* **CHANGED** `dsl!`: deprecated the field access syntax.
* **CHANGED MAJOR**: Changed the meaing of leaf nodes!!!

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
