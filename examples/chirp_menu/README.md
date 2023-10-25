# Define a menu using a chirp file

[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)

We will now use `cuicui_chirp` to define the UI. To do so, we need to add it
as a dependency first:

```toml
[dependencies]
# previous dependencies
# ...
# New dependency:
cuicui_chirp = "0.10.2"
```

The app setup requires adding `cuicui_chirp::loader::Plugin::new::<UiDsl>()`,
we also setup hot reloading by setting the asset plugin.

```rust,no_run,noplayground
{{#include main.rs:app}}
```

> **Documentation**
>
> Methods available in `chirp` files are the methods available in the choosen
> DSL type (in this case, it would be the `UiDsl` methods). Check the documentation
> page for the corresponding type you are using as DSL. All methods that accept
> an `&mut self` are candidate.

The `setup` system, where we previously spawned the whole scene, is now
completely trivial, we just spawn a single entity with a `ChirpBundle`.

```rust,no_run,noplayground
{{#include main.rs:setup}}
```

`chirp_menu.chirp` is located the `assets/` folder.

This chapter assumes you've read [the previous chapter][dsl-chapter]. We will
use it as a base for this.

So where to start? Well, let's copy/past the code from the simple menu example
into `chirp_menu.chirp` and see what happens:

```rust,no_run,noplayground
Root(screen_root row distrib_start main_margin(50.) image(&bg)) {
    Column(column rules(px(100), pct(100)) main_margin(10.) image(&board)) {
        TitleCard(width(pct(100)) image(&title_card))
        TitleCard2(width(pct(50)) image(&title_card))
        Entity(image(&button_bg) width(pct(80)) text("CONTINUE"))
        Entity(image(&button_bg) width(pct(80)) text("NEW GAME"))
        Entity(image(&button_bg) width(pct(80)) text("LOAD GAME"))
        Entity(image(&button_bg) width(pct(80)) text("SETTINGS"))
        Entity(image(&button_bg) width(pct(80)) text("ADDITIONAL CONTENT"))
        Entity(image(&button_bg) width(pct(80)) text("CREDITS"))
        Entity(image(&button_bg) width(pct(80)) text("QUIT GAME"))
    }
}
```

Of course this doesn't work! But here the error format is different. The game
compiles, `cuicui_chirp` tries to load the file and displays errors it encountered
instead of spawning a scene:

```text
Error:   × Failed to load 'Handle<bevy_render::texture::image::Image>' from file '&button_bg':
  │ No such file or directory (os error 2)
    ╭─[chirp_menu.chirp:10:1]
 10 │         Entity(image(&button_bg) width(pct(80)) text("ADDITIONAL CONTENT"))
 11 │         Entity(image(&button_bg) width(pct(80)) text("CREDITS"))
 12 │         Entity(image(&button_bg) width(pct(80)) text("QUIT GAME"))
    ·                      ──────────
 13 │     }
 14 │ }
    ╰────
  help: The error comes from the ParseDsl implementation.
Error:   × Rule format was not recognized: 'pct(80)', rules end with '%', '*' or 'px'.
  │ Examples: '53%', '0.35*' and '1024px'
    ╭─[chirp_menu.chirp:10:1]
 10 │         Entity(image(&button_bg) width(pct(80)) text("ADDITIONAL CONTENT"))
 11 │         Entity(image(&button_bg) width(pct(80)) text("CREDITS"))
 12 │         Entity(image(&button_bg) width(pct(80)) text("QUIT GAME"))
    ·                                        ───────
 13 │     }
 14 │ }
    ╰────
  help: The error comes from the ParseDsl implementation.
```

The part of the error message we are the most interested in is the bit of text after `Error`:

> × Failed to load 'Handle<bevy_render::texture::image::Image>' from file '&button_bg':<br>
> │ No such file or directory (os error 2)

and

> × Rule format was not recognized: 'pct(80)', rules end with '%', '\*' or 'px'.<br>
> │ Examples: '53%', '0.35\*' and '1024px'


Don't close the window! Chirp files are hot-reloadable, you can edit the file
and see the effect live.

We have two kind of errors here:

1. Argument to the `image` method.
2. Argument to `width` and `rule`.

For (1), **methods that accept a `Handle<T>` in rust accept a string argument
in chirp files**. For (2), chirp files use the
`FromStr` implementation on `Rule` to parse them, again, as the error message
states.

So let's replace the variables from the DSL example with the file path and
change the syntax on rules:

```rust,no_run,noplayground
Root(screen_root row distrib_start main_margin(50) image("background.jpg")) {
    Column(column rules(100px, 100pct) main_margin(10) image("board.png")) {
        TitleCard(width(100pct) image("logo.png"))
        TitleCard2(width(50pct) image("logo.png"))
        Entity(image("button.png") width(80%) text("CONTINUE"))
        Entity(image("button.png") width(80%) text("NEW GAME"))
        Entity(image("button.png") width(80%) text("LOAD GAME"))
        Entity(image("button.png") width(80%) text("SETTINGS"))
        Entity(image("button.png") width(80%) text("ADDITIONAL CONTENT"))
        Entity(image("button.png") width(80%) text("CREDITS"))
        Entity(image("button.png") width(80%) text("QUIT GAME"))
    }
}
```

Save the file and …

New set of errors, but not as many.
We forgot to convert `pct` to `%` in some places. Let's fix this and save again.

![The scene from simple menu, now loaded](../../chirp_menu_gallery/first_attempt.jpg)

<details><summary><b>Rules syntax by context</b></summary>

So how to write `cuicui_layout` rules? Here is a table:

Note that `pct`, `child` and `px` are [**rust functions**][dsl-functions]
and must be imported.

| [`Rule`]     | in `dsl!` | in chirp |
|--------------|-----------|----------|
| [`Children`] | [`child`] | `2*`     |
| [`Parent`]   | [`pct`]   | `95%`    |
| [`Fixed`]    | [`px`]    | `120px`  |

</details>

### Templates

This is already good. And it was much faster than before! Didn't even need to
close and re-open the game once!

But, as before, we'd like to make this shorter. To do this, we'll extract the button
entity into a **template definition**. In chirp, you define templates at the
beginning of the file with the `fn` keyword, and you use them like you would use
a rust macro:

```rust,no_run,noplayground
// Define a template
fn button() {
    Button(image("button.png") width(80%) text("Button"))
}
Root(screen_root row distrib_start main_margin(50) image("background.jpg")) {
    Column(column rules(150px, 100%) main_margin(10) image("board.png")) {
        TitleCard(width(100%) image("logo.png"))
        TitleCard2(width(50%) image("logo.png"))
        // Call it like a rust macro
        button!()
        button!()
        button!()
        button!()
        button!()
        button!()
        button!()
    }
}
```

Again, all you need to do is hit the save shortcut in your text editor, and
the changes show up directly on screen. (Or errors in the terminal, if any)

![All buttons now have the "Button" text](../../chirp_menu_gallery/button_button.jpg)

### Template arguments

Well, we still want to have different names per button. Miracle, templates support
**parameters**. They are like argument to rust functions:

```rust,no_run,noplayground
{{#include ../../assets/chirp_menu.chirp:template_fn}}
```

And when calling the template, we pass an argument:

```rust,no_run,noplayground
{{#include ../../assets/chirp_menu.chirp:template_call}}
```

#### Template parameter substitution rules

Currently, it is not possible to use template parameters everywhere.
See [limitations][parameter-substitution-rules].

### Template extras

Now we want each button to have a different color. There are seven of them, like
the seven dwarfes, ~~the seven fingers of the hand~~, and seven colors of the rainbow!

We could add a second parameter to our template, but instead, we'll use a method extra:

```rust,no_run,noplayground
        button!("CONTINUE")(bg(red))
        button!("NEW GAME")(bg(orange))
        button!("LOAD GAME")(bg(yellow))
        button!("SETTINGS")(bg(green))
        button!("ADDITIONAL CONTENT")(bg(cyan))
        button!("CREDITS")(bg(blue))
        button!("QUIT GAME")(bg(violet))
```

![Now the buttons's outline are varycolored](../../chirp_menu_gallery/rainbow.jpg)

A bit ugly. We should make `"button.png"` white so that it mixes with the rainbow
colors correctly.

In the chirp file, what happens here is that we are adding the `bg(color)` method
to the entity spawned by `button!`. In effect `button!("CREDITS")(bg(blue))`,
if we expand the template, becomes:

```rust,no_run,noplayground
//                                 bg(blue) method is added to the end vvvvvvvv
Entity(named("CREDITS") image("button.png") width(80%) text("CREDITS") bg(blue))
```

Template extras also work with children nodes, within `{}`.

And that's pretty much it when it comes to `cuicui_chirp`. Next, we will
add a bit of interactivity.


[`Rule`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/enum.Rule.html
[`Children`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/enum.Rule.html#variant.Children
[`Parent`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/enum.Rule.html#variant.Parent
[`Fixed`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/enum.Rule.html#variant.Fixed
[`LayoutDsl`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/dsl/struct.LayoutDsl.html
[`UiDsl`]: https://docs.rs/cuicui_layout_bevy_ui/0.10.2/cuicui_layout_bevy_ui/dsl/struct.UiDsl.html
[dsl-chapter]: ../simple_menu
[parameter-substitution-rules]: https://docs.rs/cuicui_chirp/0.10.2/cuicui_chirp/index.html#parameter-substitution
[dsl-functions]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/dsl/index.html#functions
[`child`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/dsl/fn.child.html
[`pct`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/dsl/fn.pct.html
[`px`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/dsl/fn.px.html
