# Cuicui DSL

Making bevy entity hierarchies will never be this easy!

Use the [`dsl!`] with any [`DslBundle`] type to spawn entities using a
concise yet extensible and explicit syntax.

The [`dsl!`] macro transforms an imperative API into a pure declarative syntax,
apt to make a functional programmer sight in satisfaction.

```rust
# use cuicui_dsl::macros::__doc_helpers::*; // ignore this line pls
use cuicui_dsl::dsl;

fn setup(mut cmds: Commands, serv: Res<AssetServer>) {

let bg: Handle<_> = serv.load("background.png");
let board: Handle<_> = serv.load("board.png");

dsl! {
    &mut cmds.spawn_empty(),
    Root(screen_root main_margin(100.) align_start image(&bg) row) {
        Entity(button("Button text 1") color(Color::BLUE) rules(px(40), pct(100)))
        Entity(button("Button text 2") color(Color::RED) rules(px(40), pct(100)))
        Menu(fill_main_axis image(&board) column) {
            TitleCard(rules(pct(100), px(100)))
        }
    }
}
}
```

This would be equivalent to the following:

<details><summary><b>Click to see the macro expansion code</b></summary>

```rust
# use cuicui_dsl::macros::__doc_helpers::*;
fn setup(mut cmds: Commands, serv: Res<AssetServer>) {

let bg = serv.load("background.png");
let board = serv.load("board.png");

let mut x = <Dsl>::default();
x.named("Root");
x.screen_root();
x.main_margin(100.);
x.align_start();
x.image(&bg);
x.row();
x.node(&mut cmds.spawn_empty(), |cmds| {
    let mut x = <Dsl>::default();
    let leaf_cmd = &mut cmds.spawn_empty();
    x.button("Button text 1");
    x.color(Color::BLUE);
    x.rules(px(40), pct(100));
    x.insert(leaf_cmd);

    let mut x = <Dsl>::default();
    let leaf_cmd = &mut cmds.spawn_empty();
    x.button("Button text 2");
    x.color(Color::RED);
    x.rules(px(40), pct(100));
    x.insert(leaf_cmd);

    let mut x = <Dsl>::default();
    let node_cmd = &mut cmds.spawn_empty();
    x.named("Menu");
    x.fill_main_axis();
    x.image(&board);
    x.column();
    x.node(node_cmd, |cmds| {
        let mut x = <Dsl>::default();
        let leaf_cmd = &mut cmds.spawn_empty();
        x.named("TitleCard");
        x.rules(pct(100), px(100));
        x.insert(leaf_cmd);
    });
});
}
```

</details>

Check the [`dsl!`] macro documentation for a very detailed rundown of what it
is possible to do with it.

[`dsl!`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html
[`DslBundle`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/trait.DslBundle.html
