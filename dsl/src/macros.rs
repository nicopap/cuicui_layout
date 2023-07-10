/// Reorganize rust method syntax to play wonderfully with bevy's
/// hierarchy spawning mechanism.
///
/// Basically, this is a way to use `&mut self` methods on an arbitrary type
/// but in a declarative way.
///
/// # Syntax
///
/// `dsl!` accepts as argument:
///
/// 1. (optionally) between `<$ty>`, a [`DslBundle`] type.
///    By default, it will use the identifier `Dsl` in scope.
///    This will be referred as **`Dsl`** in the rest of this documentation.
/// 2. An expression implementing [`IntoEntityCommands`].
/// 3. a series of **dsl statements**.
///
/// ## Dsl statements
///
/// A dsl statement spawns a single entity.
///
/// There are three kinds of dsl statements:
/// - parent node statement
/// - leaf node statement
/// - code statement
///
/// ### Parent node
///
/// The parent node statement has the following syntax:
/// ```text
/// <ident>([dsl method]*) {
///     [dsl statement]*
/// }
/// ```
/// Concretly, it looks like the following:
/// ```ignore
/// row(screen_root, "root", main_margin 100., align_start, image &bg) {
///     button(color Color::BLUE, width 40., height 100.);
///     button(color Color::RED, width 40., height 100.);
///     column("menu", width px 310, main_margin 40., fill_main_axis, image &board) {
///         spawn(title_card, "Title card", height px 100, width %100);
///     }
/// }
/// ```
///
/// The part between parenthesis (`()`) is a list of [dsl methods](#dsl-methods).
/// They are applied to the `Dsl` [`DslBundle`] each one after the other.
/// Then, the `<ident>` (here `row`) dsl method is applied.
/// And finally, an entity is spawn with the so-constructed bundle,
/// following, the dsl statements within braces (`{}`) are spawned
/// as children of the parent node entity.
///
/// For the visually-minded, this is how the previous code would look like without
/// the macro:
///
/// ```ignore
/// let mut x = <Dsl>::default();
/// x.screen_root();
/// x.named("root");
/// x.main_margin(100.);
/// x.align_start();
/// x.image(&bg);
/// x.row();
/// x.insert(cmds.spawn_empty()).with_children(|cmds| {
///     // Same goes with the children:
///     // button(color Color::BLUE, width 40., height 100.);
///     // button(color Color::RED, width 40., height 100.);
///     // column("menu", width px 310, main_margin 40., fill_main_axis, image &board) {
///     //     spawn(title_card, "Title card", height px 100, width %100);
///     // }
/// });
/// ```
///
/// ### Leaf node
///
/// Leaf node statements look exactly like parent nodes, but instead of having
/// a list of children statement between braces, it ends with a semicolon:
///
/// ```text
/// <ident>(<expr>, [dsl methods]*);
/// ```
/// Concretly:
/// ```ignore
/// button_img(sprite, color Color::BLUE, width 40., height 100.);
/// ```
///
/// The methods are called similarly to a Parent node statement.
/// With the difference that `button_img` will be called with `<expr>` and
/// an `EntityCommands`:
///
/// ```ignore
/// let mut x = <Dsl>::default();
/// x.color(Color::BLUE);
/// x.width(40.);
/// x.height(100.);
/// x.button_img(sprite, cmds.spawn_empty());
/// ```
///
/// ### Code
///
/// One last statement type exists, it gives the user back full control over
/// the `cmds`, even nested within a parent node.
/// It looks like this:
///
/// ```text
/// code(let <cmd_ident>) {
///     <rust code>
/// }
/// ```
/// Concretly:
/// ```ignore
/// code(let my_cmd) {
///     for n in &menu_buttons {
///         let name = format!("{n} button");
///         println!("{name}");
///         my_cmd.insert(Name::new(name));
///     }
/// }
/// ```
/// This is directly inserted as-is in the macro, so it would look as follow:
/// ```ignore
/// let my_cmd = cmds;
/// for n in &menu_buttons {
///     let name = format!("{n} button");
///     println!("{name}");
///     my_cmd.insert(Name::new(name));
/// }
/// ```
///
/// Nothing prevents you from using `code` inside a parent node,
/// neither using the `dsl!` macro within rust code within a code statement.
///
/// ```ignore
/// dsl!(cmds,
///     row(height 100., fill_main_axis) {
///         code(let my_cmds) {
///             for name in &button_names {
///                 dsl!(my_cmds, button_text(name, color Color::BLUE);)
///             }
///         }
///     }
/// )
/// ```
///
/// ## Dsl methods
///
/// Stuff within parenthesis in a dsl statement are **dsl methods**.
/// Dsl methods are nothing more than method calls on `Dsl`.
///
/// There are four kind of dsl methods:
/// - name litereal methods
/// - bare methods
/// - single argument methods
/// - multiple arguments methods
///
/// ### Name literals
///
/// The only "special" kind of dsl method that does more than call a method
/// is the name literal argument. It simply calls a method called `named` with
/// the string literal in question… Yep. You'd argue it's nothing special, yet
/// it's the _most_ special of all dsl methods.
///
/// A name literal argument looks as follow:
/// ```ignore
/// <literal>
/// ```
/// Or concretely, several examples:
/// ```ignore
/// "MyName"
/// "hello world"
/// ```
/// It gets translated into the following:
/// ```ignore
/// x.named("MyName");
/// x.named("hello world");
/// ```
/// Of course, if the `Dsl` doesn't have a `named` method, this will fail.
/// This crate exports [`BaseDsl`], which exposes the `named` method,
/// [`BaseDsl`], in fact, does nothing else than precisely that: providing a
/// `named` method.
///
/// ### Method calls
///
/// Otherwise, methods are translated directly into rust method calls on `Dsl`:
/// ```text
/// some_method               // bare method
/// method_with_arg <expr>    // single argument method
/// several_args ([<expr>],*) // multiple arguments method
/// ```
/// Which concretly looks as follow:
/// ```ignore
/// x.some_method();
/// x.method_with_arg(15 * 25. as u32);
/// x.several_args("hi folks", variable_name, Color::RED);
/// ```
///
/// # Extending `dsl!`
///
/// Since `dsl!` is streight up nothing more than sugar on top of rust's
/// method call syntax, it's trivial to add your own methods/statements.
///
/// TODO: example extension and example.
///
/// [`DslBundle`]: crate::DslBundle
/// [`BaseDsl`]: crate::BaseDsl
/// [`IntoEntityCommands`]: crate::IntoEntityCommands
#[rustfmt::skip]
#[macro_export]
macro_rules! dsl {
    (@arg $x:ident,) => {  };
    (@arg $x:ident, $nm:literal          $(,$($t:tt)*)?)=>{$x.named($nm) $(; dsl!(@arg $x, $($t)*))?};
    (@arg $x:ident, $m:ident $m_arg:expr $(,$($t:tt)*)?)=>{$x.$m($m_arg) $(; dsl!(@arg $x, $($t)*))?};
    (@arg $x:ident, $m:ident             $(,$($t:tt)*)?)=>{$x.$m()       $(; dsl!(@arg $x, $($t)*))?};

    (@statement [$d_ty:ty, $cmds:expr] ) => { };
    (@statement [$d_ty:ty, $cmds:expr] code (let $cmds_ident:ident) {$($code:tt)*} $($($t:tt)+)?) => {
        let $cmds_ident = $cmds;
        $($code)*
        $(; dsl!(@statement $cmds, $d_ty, $($t)*))?
    };
    (@statement [$d_ty:ty, $cmds:expr] $leaf_node:ident ( $value:expr $(, $($args:tt)*)? ) ; $($($t:tt)+)?) => {
        let mut leaf_cmd = $cmds.to_cmds();
        let mut x = <$d_ty>::default();
        dsl!(@arg x, $($($args)*)?);
        x.insert(&mut leaf_cmd);
        x.$leaf_node($value, leaf_cmd)
        $(; dsl!(@statement [$d_ty, $cmds] $($t)*))?
    };
    (@statement [$d_ty:ty, $cmds:expr] $parent_node:ident ($($args:tt)*) {$($inner:tt)*} $($($t:tt)+)?) => {
        let mut arg = <$d_ty>::default();
        dsl!(@arg arg, $($args)*, $parent_node);
        arg.node($cmds.to_cmds(), |mut cmds| {
            dsl!(@statement [$d_ty, cmds] $($inner)*);
        })
        $(; dsl!(@statement [$d_ty, $cmds] $($t)*))?
    };
    (<$builder:ty> $cmds:expr, $($t:tt)*) => {{
        use $crate::{IntoEntityCommands, DslBundle};
        dsl!(@statement [$builder, $cmds] $($t)*);
    }};
    ($cmds:expr, $($t:tt)*) => { dsl!(<Dsl> $cmds, $($t)*) };
}
