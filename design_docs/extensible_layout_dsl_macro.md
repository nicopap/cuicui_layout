# Extensible Layout declaration

**Problem**: I want users to be able to add **arbitrary methods** to
`LayoutCommandsExt`.

This would allow seemlessly integrating 3rd party and end-user components to the
layout DSL. Consider this, for `bevy-ui-navigation`:

```diff
  layout! {
      &mut cmds,
      row(screen_root, "root", main_margin 100, align_start) {
-         column("menu", width px 300, fill_main_axis) {
+         column("menu", menu, width px 300, fill_main_axis) {
              spawn_ui(title_card, "Title card", height px 100, width %100);
              code(let cmds) {
                  for n in &menu_buttons {
                      let name = format!("{n} button");
                      layout!(
                          cmds,
-                         spawn_ui(text!(font, *n), named name, height px 30);
+                         spawn_ui(text!(font, *n), focusable, named name, height px 30);
                      );
                  }
              }
          }
      }
  }
```

The following should be possible:

- Call the extension methods on `EntityCommands`, `&mut Commands` `&mut ChildBuilder`
- Should only require importing the user's extension trait.
- Allow user defaults

## 1. Do nothing, let users add their own extension methods

This approach is invalid.

User needs to control what `LayoutCommands` spawns (maybe also inspect it), the
extension method has no power over the code implemented by `cuicui_layout`, we
need a form of inversion of control.

## 2. Make `LayoutCommands` wrap a `T`

We need users to be able to "intercept" spawn/insert commands and add their own
bundles on top of it, based on the `T` state.

For example, when spawing `FlowBundle` and `RootBundle` in `cuicui_layout_bevy_ui`,
we need to add `bevy_ui` related components.

Then user can do something like

```rust
trait MyExtraMethods {
  fn menu(self) -> Lc<MenuCommands>;
}
impl<T: LayoutCommandsExt<MenuCommands>> MyExtraMethods for T {
  // ...
}
```

## 3. The `spawn_ui` question

We also want to be able to spawn arbitrary `UiBundle`. And ideally, use arbitrary
types for them! (such as `&str`).

Problem is. Two different users of `cuicui_layout` might have two different
target bundle for `&str`. For example `cuicui_layout_bevy_ui` and `cuicui_layout_bevy_sprite`.

My solution was to add a `Marker` generic type parameter to `IntoUiBundle`.
And have `spawn_ui` be generic over `Marker` as follow:

```rust
fn spawn_ui<M>(mut self, bundle: impl IntoUiBundle<M>);
```

Now, the implementor can use an arbitrary type unique to their crate when
implementing `IntoUiBundle`.

Since rust knows which traits a type implements, and unless the users depends
on several "implementation" crates for `cuicui_layout`
(**TODO**: probably likely to happen to at least 5% of users!),
it will chose automatically `Marker`, without further specificaction required
by the end user.
