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
    &mut cmds,
    row(screen_root, "root", main_margin 100., align_start, image &bg) {
        spawn(button "Button text 1", color Color::BLUE, width px(40), height pct(100));
        spawn(button "Button text 2", color Color::RED, width px(40), height pct(100));
        column("menu", fill_main_axis, image &board) {
            spawn("Title card", height px(100), width pct(100));
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
x.screen_root();
x.named("root");
x.main_margin(100.);
x.align_start();
x.image(&bg);
x.row();
x.node(&mut cmds.to_cmds(), |cmds| {
    let mut x = <Dsl>::default();
    let mut leaf_cmd = cmds.to_cmds();
    x.button("Button text 1");
    x.color(Color::BLUE);
    x.width(px(40));
    x.height(pct(100));
    x.insert(&mut leaf_cmd);

    let mut x = <Dsl>::default();
    let mut leaf_cmd = cmds.to_cmds();
    x.button("Button text 2");
    x.color(Color::RED);
    x.width(px(40));
    x.height(pct(100));
    x.insert(&mut leaf_cmd);

    let mut x = <Dsl>::default();
    let mut node_cmd = cmds.to_cmds();
    x.named("menu");
    x.fill_main_axis();
    x.image(&board);
    x.column();
    x.node(&mut node_cmd, |cmds| {
        let mut x = <Dsl>::default();
        let mut leaf_cmd = cmds.to_cmds();
        x.named("Title card");
        x.height(px(100));
        x.width(pct(100));
        x.insert(&mut leaf_cmd);
    });
});
}
```

</details>

Check the [`dsl!`] macro documentation for a very detailed rundown of what it
is possible to do with it.

[`dsl!`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html
[`DslBundle`]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/trait.DslBundle.html
