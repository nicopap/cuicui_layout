# Scaling up with a custom DSL

[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)

[`cuicui_dsl`] and [`cuicui_chirp`] are parametrized over the [`DslBundle`] and
[`ParseDsl`] traits respectively.

You can directly use one of the DSLs exported by an external crate such as
[`UiDsl`], [`LayoutDsl`], [`NavigationDsl`] or [`SpriteDsl`], but we recommend
that you define your own DSL on top of them.

This is how [the chirpunk example] works. We re-use pre-existing DSLs, but add
our own layer on top, to create a unique vocabulary that applies to the specific
game we build.

So let's make a game.

## Better Factorio

What better genre than factory-building to illustrate a game UI library?
Let's mix in some grand strategy for good measure. We'll make
Factorio×CrusaderKings fusion.

Our goal is to make:

- A menu with several tabs, tabs are:
- A "game" menu with buttons that represent game menu buttons, clicking on them prints a message
- A "production" menu (with static `png` as graph for now)
- A "diplomacy" menu where we can start and end wars, marry someone, launch missiles

We will be able to swap menu by clicking buttons, and most game-specific actions
will result in a message being logged into the terminal.

For more complex interaction patterns, read [the next chapter].

## A menu with tabs

So let's make a menu.

First, we write the whole menu. To pretend it is a game menu, we draw a
background and then have the menu occupy a subset of the screen. We do that
by setting a 25 pixels margin on the `Root` entity.

The menu with tabs is a column menu, the first row is the tab line, the second
the content:

```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:root_methods}}
```

Then, we fill up the rest.

The tab line:

```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:tabs}}
```

(more on the `tab!` template later)

```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:menu}}
```

The content uses the overlapping layout distribution mode with `layout(">oCaC")`
(the `o` is the important bit, it stands for "overlapping"). This allows
each child of `Menu` to occupy the same space.

We now just have `tab`, `game_menu`, `production_menu`, and `diplomacy_menu` to
define. (more on those `(hidden)` later)

![Several nodes on the same space](../../custom_dsl_gallery/all_overlapping.png)

## Hide the menus

We used _template extras_ here to mark two of the `Menu` children as "hidden".

```rust,no_run,noplayground
game_menu!()
production_menu!()(hidden)
diplomacy_menu!()(hidden)
```

Indeed, we don't want all three menus to be visible at the same time. To this
end, we spawn the production and diplomacy menus with the `Visibility` component
set to `Hidden`.

But here is the hang up! Neither `LayoutDsl` or `UiDsl` have a `hidden` method,
how are we to set the `Visibility` component?

Answer: **We write our own DSL.**

Let's start by creating a new module:

```rust,no_run,noplayground
mod dsl;
```

Then define a `BetterFactorioDsl`:

```rust,no_run,noplayground
// `DslBundle` requires `Default`
#[derive(Default)]
pub struct BetterFactorioDsl {
    inner: UiDsl,
    is_hidden: bool,
}
```

Let's add the chirp loader for `BetterFactorioDsl` in our `add_plugins`:

```rust,no_run,noplayground
{{#include src/main.rs:add_plugin}}
```

We need to implement [`ParseDsl`] and [`DslBundle`] on `BetterFactorioDsl`
for this to compile.
The [`parse_dsl_impl`] macro is how we implement `ParseDsl`.
We use the [`delegate`] meta-attribute, so that we can re-use the `UiDsl` and
`LayoutDsl` methods in our chirp file:

```rust,no_run,noplayground
#[parse_dsl_impl(delegate = inner)]
impl BetterFactorioDsl {}

impl DslBundle for BetterFactorioDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        self.inner.insert(cmds)
    }
}
```

It compiles now, but we are missing the `hidden` method:

```
Error:   × No 'hidden' method
    ╭─[better_factorio/menu.chirp:65:1]
 65 │             game_menu!()
 66 │             production_menu!()(hidden)
 67 │             diplomacy_menu!()(hidden)
    ·                               ^^^^^^
 68 │         }
    ╰────
  help: custom_dsl::dsl::BetterFactorioDsl doesn't contain a method with this name.
```

Let's add it then! We already have a `is_hidden` field, we just have to
define a `hidden` method to set it:

```rust,no_run,noplayground
{{#include src/dsl.rs:hidden_method}}
```

Now we can read the `is_hidden` field in the **`DslBundle::insert`** implementation:

```rust,no_run,noplayground
{{#include src/dsl.rs:add_hidden}}
```

Make sure to add it _after_ the `inner.insert(cmds)`. `Visibility` is part of the
`NodeBundle` that `UiDsl` adds to the entity. Adding it _after_ the `inner` ensures
that we overwrite the `Visibility` component with the wanted value.

## Game menu & interaction

Ok, so I included the game menu in all those screenshots, but didn't define it
yet. Sorry for the confusion.

The game menu was a template call, `game_menu!()`. Then, let's define it.

```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:game_menu}}
```

`print_button!` is yet another template. It stands for a button we can click,
when clicking it, a message is displayed in the console.

```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:print_button}}
```

Notice that we called the `print_text(button_text)` and `highlight` methods in
`print_button!`.

- `highlight` should add a component that changes the color of the button when
  it's being hovered
- `print_text` does print in the console the button text content.

Let's add them to our `BetterFactorioDsl`.

### Using `bevy_mod_picking`

We will use the [`bevy_mod_picking`] components for this.

We should use the [`On`] component from `bevy_mod_picking`. One issue with `On`
is that it is not [`Reflect`], therefore, we cannot use it in our chirp file;
`cuicui_chirp` creates a scene then adds it to the bevy world, and this requires
all components from the scene to be `Reflect`.

> `cuicui_dsl` doesn't have this issue.

We can get around this limitation by creating a "mirror" component. Mirror
components are `Reflect` proxies that are synchronized with actual components.

We already define a `MirrorPlugin` in the [`cuicui_examples`] shared code.

> I plan on extracting this into a standalone crate, `cuicui_mirror`.

All we have left to do is to define a mirror component,
implement `From<&ReflectOnClick>` for `On<Pointer<Click>>`, and add `MirrorPlugin`.

```rust,no_run,noplayground
{{#include src/reflect_on_click.rs}}
```

```rust,no_run,noplayground
{{#include src/main.rs:mirror_plugin}}
```
Now let's amend `BetterFactorioDsl` to add the `highlight` and `print_text` methods:

```diff,no_run,noplayground
#[derive(Default)]
pub struct BetterFactorioDsl {
    inner: UiDsl,
    is_hidden: bool,
+   is_highlight: bool,
+   text_to_print: Option<Box<str>>,
}
```

In the `#[parse_dsl_impl] impl BetterFactorioDsl` block:

```rust,no_run,noplayground
{{#include src/dsl.rs:game_menu_methods}}
```

In `DslBundle::insert for BetterFactorioDsl`:

```rust,no_run,noplayground
{{#include src/dsl.rs:game_menu_inserts}}
```

I'll skip over `Highlight` here, you can [see the implementation for yourself][highlight.rs].
Similarly to `ReflectOnClick`, we use `bevy_mod_picking` to react to events, but
define it as a standalone `Reflect` component to be able to use it in a scene.

![A mouse cursor moving over buttons that change color when they are hovered](../../custom_dsl_gallery/hover.gif)

(not shown: the logs in the terminal)

## The tab line

Now that only a single menu shows up at a time, we should have the ability to
switch between them.

We should be able to click on a tab to swap between menu.

Let's define our `tab!` template.

- We'll use a darker tone color for unselected tabs, `#6b4d22`.
- We'll give some space between each tab, to reproduce real world tabs
  ([skeuomorphic design] 🤓).

```rust,no_run,noplayground
fn tab(menu_name) {
    Entity(row rules(1.1*, 100%) named(menu_name)) {
        TabSurface(highlight row rules(1.1*, 100%) bg(#6b4d22)) {
            TabText(text(menu_name))
        }
    }
}
// ...
Tabs(row rules(100%, 12.5%) distrib_start) {
    tab!("Game Menu")
    tab!("Production Menu")
    tab!("Diplomacy Menu")
}
```

![Our menu with the tabs, but all deselected](../../custom_dsl_gallery/all_deselected_tabs.png)

It's nice and all, but we need some interactivity. We also need the selected tab
to have the same color as the game menu background.

We can't use _template extras_ in this case, because the root node of the `tab`
template is transparent, we need to pass the color as an additional argument

```diff
-fn tab(menu_name) {
+fn tab(menu_name, initial_color) {
    Entity(row rules(1.1*, 100%) named(menu_name)) {
-       TabSurface(highlight row rules(1.1*, 100%) bg(#6b4d22)) {
+       TabSurface(highlight row rules(1.1*, 100%) bg(initial_color)) {
            TabText(text(menu_name))
        }
Tabs(row rules(100%, 12.5%) distrib_start) {
-   tab!("Game Menu")
-   tab!("Production Menu")
-   tab!("Diplomacy Menu")
+   tab!("Game Menu", burlywood)
+   tab!("Production Menu", #6b4d22)
+   tab!("Diplomacy Menu", #6b4d22)
```

### Switching between tabs

Unlike the `print_text` buttons, here, we need to change which menu is visible
when the tab is pressed. So let's create a new method: `switch_tab` and add it
to our template:

```diff
    Entity(row rules(1.1*, 100%) named(menu_name)) {
-       TabSurface(highlight row rules(1.1*, 100%) bg(#6b4d22)) {
+       TabSurface(highlight switch_tab(menu_index) row rules(1.1*, 100%) bg(initial_color)) {
            TabText(text(menu_name))
        }
```

Similarly to [`Highlight`][highlight.rs], I won't expand on `switch_tab`. The
idea is to:

1. Have a `TabButton(u8)` component. Whenever an entity with this component is
   clicked, send a `SwitchTab` event with the `u8`.
2. Mark the parent of all the three menus with a component, `Tabs`.
3. Whenever we receive a `SwitchTab(u8)` component, query for the `Tabs` entity and
   its children, set `Visibility` of all the children but the `u8` to `Visibility::Hidden`.

```rust,no_run,noplayground
{{#include ../cuicui_examples/src/switch.rs:system}}
```

[See the implementation for details][switch.rs].

We need to use `bevy_mod_picking` for this as well, and this requires using a
mirror component. Conveniently, we already did it in [a previous section](#using-bevy_mod_picking).

```diff,no_run,noplayground
#[derive(Default)]
pub struct BetterFactorioDsl {
    inner: UiDsl,
    is_hidden: bool,
    is_highlight: bool,
    text_to_print: Option<Box<str>>,
+   switch_tab: Option<u8>,
}
```

In the `#[parse_dsl_impl] impl BetterFactorioDsl` block:

```rust,no_run,noplayground
{{#include src/dsl.rs:switch_tab_method}}
```

In `DslBundle::insert for BetterFactorioDsl`:

```rust,no_run,noplayground
{{#include src/dsl.rs:switch_tab_insert}}
```

Finally, we need to pass the menu index as parameter to the template:

```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:tab}}
```

```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:tabs}}
```


![Navigating between menus with tabs](../../custom_dsl_gallery/switch_tab.gif)

## Diplomacy and Production

The diplomacy menu is very similar to the game menu, I won't go over it, just
get a look at the code:


```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:diplomacy}}
```

The production menu is more interesting. Similarly to the root menu, we want
several panels (production types: electricity, water, pollution)
we can switch between, and buttons to select the panel.

We will use the same [switch][switch.rs] implementation that we used for tabs.
This time, we will name our method `switch_graph`. I won't go over the rust
implementation, as it's pretty much a copy/paste of the tabs switching code.

```rust,no_run,noplayground
{{#include ../../assets/better_factorio/menu.chirp:production}}
```


[`bevy_mod_picking`]: https://crates.io/crates/bevy_mod_picking
[`cuicui_chirp`]: https://docs.rs/cuicui_chirp/0.12.0/cuicui_chirp/
[`cuicui_dsl`]: https://docs.rs/cuicui_dsl/0.12.0/cuicui_dsl/
[`cuicui_examples`]: https://github.com/nicopap/cuicui_layout/tree/main/examples/cuicui_examples
[`delegate`]: https://docs.rs/cuicui_chirp/0.12.0/cuicui_chirp/parse_dsl_impl/fn.delegate.html
[`DslBundle`]: https://docs.rs/cuicui_layout/0.12.0/cuicui_layout/trait.DslBundle.html
[highlight.rs]: https://github.com/nicopap/cuicui_layout/blob/main/examples/cuicui_examples/src/highlight.rs
[`LayoutDsl`]: https://docs.rs/cuicui_layout/0.12.0/cuicui_layout/dsl/struct.LayoutDsl.html
[`NavigationDsl`]: https://docs.rs/bevy-ui-navigation/0.33.0/bevy_ui_navigation/index.html
[`On`]: https://docs.rs/bevy_mod_picking/0.17.0/bevy_mod_picking/prelude/struct.On.html
[`ParseDsl`]: https://docs.rs/cuicui_chirp/0.12.0/cuicui_chirp/parse_dsl/trait.ParseDsl.html
[`parse_dsl_impl`]: https://docs.rs/cuicui_chirp/0.12.0/cuicui_chirp/parse_dsl_impl/index.html
[`Reflect`]: https://docs.rs/bevy/0.12/bevy/reflect/trait.Reflect.html
[skeuomorphic design]: https://en.wikipedia.org/wiki/Skeuomorph
[`SpriteDsl`]: https://docs.rs/cuicui_layout_bevy_sprite/0.12.0/cuicui_layout_bevy_sprite/struct.SpriteDsl.html
[switch.rs]: https://github.com/nicopap/cuicui_layout/blob/main/examples/cuicui_examples/src/switch.rs
[the chirpunk example]: https://github.com/nicopap/cuicui_layout/tree/main/examples/chirpunk
[the next chapter]: ../../reactivity.html
[`UiDsl`]: https://docs.rs/cuicui_layout_bevy_ui/0.12.0/cuicui_layout_bevy_ui/dsl/struct.UiDsl.html
