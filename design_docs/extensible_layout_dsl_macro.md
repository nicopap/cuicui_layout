# Extensible Layout declaration

As of commit 23cba2fd7 (2023-07-07), we use (2), (3) and (4), but strongly
considering (5).

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

## 2. `impl CommandLike for XCmds` + `impl X for LayoutCommands`

We need users to be able to "intercept" spawn/insert commands and add their own
bundles on top of it, based on the `T` state.

For example, when spawing `FlowBundle` and `RootBundle` in `cuicui_layout_bevy_ui`,
we need to add `bevy_ui` related components.

Then user can do something like

```rust
struct XCmds<'w, 's, 'a> {
  inner: EntityCommands<'w, 's, 'a>,
  menu_type: Menu
}
impl<'w, 's, 'a> CommandLike for XCmds<'w, 's, 'a> {
  //...
}

trait X {
  fn menu(self) -> LayoutCommands<XCmds>;
}
impl X for LayoutCommands<XCmds> {
  fn menu(self) -> Self {
    // ...
  }
}
```

Its very honerous, but we are just making this "possible" until we find a better
way that makes it "simple".

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

## 4. A simpler approach

So, instead of deciding tht command-like types should have all the `LayoutCommands`
methods, let's just add a `layout` method to them (through trait method extension).

Then we have a `LayoutCommands` and we can do whatever we want with it.

We are still `impl CommandLike for X` + `impl MyExtensionTrait for LayoutCommands`
for extensibility, but it's now possible to do, and the API is not absolute bonker
bad.

## 5. Even simpler

Now, I'd like to reverse the `CommandLike`. Instead of using it to "add" components,
I'd like to split the "data accumulation" and "spawn" parts of `LayoutCommands`.

After all, it is doing nothing else than accumulating data and spawning stuff according
to the accumulated data.

It seems that splitting the `LayoutCommands` could help simplify extending it.

Saddly, it wouldn't play well with the method-based DSL, but we can expand a macro
to look like

```rust
#[derive(DerefMut)]
struct Layout<T = ()> {
  #[deref]
  inner: T,
  name: Option<String>,
  root: RootKind,
  // etc.
}
<Layout<Ui>>::new()
  .with(|s| s.align_start())
  .with(|s| s.main_margin(100.0))
  .row(cmds, |cmds| {

})
```

The extension could do:

```rust
#[derive(DerefMut)]
struct Ui<T= ()> {
  #[deref]
  inner: T,
  bg_color: Color,
  bg_image: Handle<Image>,
}
```

This would allow infinite composition.

## 6. Extensible for real.

This is not enough. It also forbids the `-> Self`, since if we call a function
from a `Deref` target, we are basically then locked  to the subset of functions
implemented by the `Deref` target (unless we go the `.with` route, but I worry
it would make macro haters give up the library)