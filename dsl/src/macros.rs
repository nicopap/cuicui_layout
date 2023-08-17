/// Reorganize rust method syntax to play wonderfully with bevy's
/// hierarchy spawning mechanism.
///
/// Basically, this is a way to use `&mut self` methods on an arbitrary type
/// but in a declarative way.
///
/// # Usage
///
/// The crate-level doc for this has a nice example, you can check it out:
/// [`crate`].
///
/// ## Cheat sheet
///
/// You already know how to use `dsl!`? here are the quick links:
///
/// - [**dsl statements**](#dsl-statements):
///   - [**spawn**](#spawn)
///   - [**leaf node**](#leaf-node)
///   - [**parent node**](#parent-node)
///   - [**code**](#code)
/// - [**dsl methods**](#dsl-methods):
///   - [**name literal**](#name-literal)
///   - [**bare**](#method-calls)
///   - [**field setting**](#method-calls)
///   - [**single argument**](#method-calls)
///   - [**multiple arguments**](#method-calls)
///
/// ## Extending `dsl!`
///
/// Since `dsl!` is straight up nothing more than sugar on top of rust's
/// method call syntax, it's trivial to add your own methods/statements.
///
/// With bevy's `DerefMut` derive, it's even possible to build on top of
/// existing implementations.
///
/// > **Warning**: Is it wise to abuse the `DerefMut` trait this way?
/// >
/// > I dunno, but it makes everything so much more convenient.
/// > See <https://github.com/nicopap/cuicui_layout/issues/26>
///
/// Consider [`BaseDsl`], it only has a single method: `named`. But we want
/// to create blinking UI. How do we do it?
///
/// Like in any bevy project, we would do as follow:
///
/// 1. Define a `Blink` component.
/// 2. Define a system that reads the `Blink` component and update some color/sprite.
/// 3. Optionally create a `BlinkBundle` that adds to an entity all things necessary
///    for blinking to work.
///
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// #[derive(Component, Default)]
/// struct Blink {
///     frequency: f32,
///     amplitude: f32,
/// }
/// #[derive(Bundle, Default)]
/// struct BlinkBundle {
///     blink: Blink,
///     spatial: SpatialBundle,
/// }
/// ```
///
/// We want to have a DSL that let us set the `frequency` and `amplitude`
/// of the `Blink` component.
///
/// More importantly though, we want our DSL to compose with any other DSL!
/// For this, we will add an `inner` field and use the bevy `DerefMut` derive
/// macro:
///
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// #[derive(Deref, DerefMut, Default)]
/// struct BlinkDsl<C = ()> {
///     #[deref]
///     inner_dsl: C,
///     pub blink: Blink,
/// }
/// impl<C: DslBundle> DslBundle for BlinkDsl<C> {
///     fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
///         // We insert first `Blink`, as to avoid overwriting things
///         // `inner_dsl.insert`  might insert itself.
///         cmds.insert(BlinkBundle { blink: self.blink, ..default() });
///         self.inner_dsl.insert(cmds)
///     }
/// }
///
/// // That's it! Now we can use `frequency` and `amplitude` in `dsl!`
/// // as if it wasn't even a big deal!
///
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// type Dsl = BlinkDsl<BaseDsl>;
/// dsl! {
///     &mut cmds,
///     spawn(.blink.frequency 0.5, "FastBlinker");
///     spawn(.blink.amplitude 2., .blink.frequency 3.0, "SlowBlinker");
/// }
/// ```
///
/// If we wanted a shorter way to set the `amplitude`, we would define a
/// `pub fn amplitude(&mut self, value: f32)` method on `BlinkDsl`.
///
/// If we want to use a pre-existing DSL with ours, we would nest them.
/// Since we `#[deref] inner: C`, all methods on the inner DSL are available
/// on the outer DSL.
///
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl; type BlinkDsl<T> = DocDsl<T>;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// type Dsl = BlinkDsl<LayoutDsl>;
/// dsl! {
///     &mut cmds,
///     spawn_ui("Fast blink", frequency 0.5, color Color::GREEN);
///     row(.blink.frequency 1., amplitude 1.0, main_margin 10., fill_main_axis) {
///         spawn_ui("Some text", .blink.amplitude 10.0, color Color::BLUE);
///     }
///     spawn_ui("Slow blink", frequency 2., color Color::RED);
/// }
/// ```
///
/// We made our DSL nestable so that it is itself composable. Say we are making
/// a public crate, and our users want the UI DSL on top of ours. They would
/// simply define their own DSL as follow:
///
/// ```ignore
/// type UserDsl = UiDsl<BlinkDsl<LayoutDsl>>;
/// ```
///
/// And it would work as is.
///
/// # Syntax
///
/// `dsl!` accepts as argument:
///
/// 1. (optionally) between `<$ty>`, a [`DslBundle`] type.
///    By default, it will use the identifier `Dsl` in scope.
///    This will be referred as **`Dsl`** in the rest of this documentation.
/// 2. An expression implementing [`IntoEntityCommands`].
/// 3. A series of [**DSL statements**](#dsl-statements).
///    * DSL statements contain themselves series of [**DSL methods**](#dsl-methods).
///
/// ## DSL statements
///
/// A DSL statement spawns a single entity.
///
/// There are three kinds of DSL statements:
/// - spawn statements
/// - leaf node statement
/// - parent node statement
/// - code statement
///
/// ### Spawn
///
/// Spawn statements create an `Entity` and calls [`DslBundle::insert`].
/// They basically spawn an entity with the given [**DSL methods**](#dsl-methods).
///
/// Optionally, they can act like parent nodes if they are directly followed
/// by curly braces:
///
/// ```text
/// spawn([dsl methods]*);
/// spawn([dsl methods]*) {
///     [dsl statements]*
/// }
/// ```
/// Concretely:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// # dsl!{ &mut cmds,
/// spawn(color Color::BLUE, width px(40), height pct(100));
/// spawn(fill_main_axis) {
///     spawn(color Color::GREEN);
/// }
/// # }
/// ```
/// This will expand to the following code:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// let mut x = <Dsl>::default();
/// x.color(Color::BLUE);
/// x.width(px(40));
/// x.height(pct(100));
/// x.insert(&mut cmds.to_cmds());
///
/// let mut x = <Dsl>::default();
/// x.fill_main_axis();
/// x.node(&mut cmds.to_cmds(), |cmds| {
///     let mut x = <Dsl>::default();
///     x.color(Color::GREEN);
///     x.insert(&mut cmds.to_cmds());
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
/// Concretely:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// # dsl!{ &mut cmds,
/// button("Button Text", color Color::BLUE, width px(40), height pct(100));
/// # }
/// ```
///
/// The methods are called similarly to a Parent node statement.
/// With the difference that `button` will be called with `<expr>` and
/// an `EntityCommands`:
///
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// let mut x = <Dsl>::default();
/// let mut c = cmds.to_cmds();
/// x.color(Color::BLUE);
/// x.width(px(40));
/// x.height(pct(100));
/// x.insert(&mut c);
/// x.button("Button Text", &mut c);
/// ```
///
/// This means that methods valid in leaf node position have the following
/// signature, where `_` is the type of `<expr>`:
///
/// ```ignore
/// fn leaf_node_method(value: _, cmds: &mut EntityCommands) -> Entity;
/// ```
///
/// > **Note**: Notice that we call `insert` followed by `button`.
/// > This is to accommodate `DerefMut`-based [`DslBundle`]s.
/// >
/// > If we were to call `button` directly, and `button` is a
/// > method on a `DerefMut` target of `Dsl` rather than `Dsl` itself,
/// > we would lose all the data related to `Dsl`!
///
/// ### Parent node
///
/// The parent node statement has the following syntax:
/// ```text
/// <ident>([dsl method]*) {
///     [dsl statement]*
/// }
/// ```
/// Concretely, it looks like the following:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let (bg, board) = ((),());
/// # dsl!{ &mut cmds,
/// row(screen_root, "root", main_margin 100., align_start, image &bg) {
///     button("Button text 1", color Color::BLUE, width px(40), height pct(100));
///     button("Button text 2", color Color::RED, width px(40), height pct(100));
///     column("menu", width px(310), main_margin 10., fill_main_axis, image &board) {
///         spawn("Title card", height px(100), width pct(100));
///     }
/// }
/// # }
/// ```
///
/// The part between parenthesis (`()`) is a list of [DSL methods](#dsl-methods).
/// They are applied to the `Dsl` [`DslBundle`] each one after the other.
/// Then, the `<ident>` (here `row`) DSL method is applied.
/// And finally, an entity is spawn with the so-constructed bundle,
/// following, the DSL statements within braces (`{}`) are spawned
/// as children of the parent node entity.
///
/// For the visually-minded, this is how the previous code would look like without
/// the macro:
///
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let bg = ();
/// let mut x = <Dsl>::default();
/// x.screen_root();
/// x.named("root");
/// x.main_margin(100.);
/// x.align_start();
/// x.image(&bg);
/// x.row();
/// x.node(&mut cmds.to_cmds(), |cmds| {
///     // Same goes with the children:
///     // button("Button text 1", color Color::BLUE, width 40., height 100.);
///     // button("Button text 2", color Color::RED, width 40., height 100.);
///     // column("menu", width px 310, main_margin 40., fill_main_axis, image &board) {
///     //     spawn(title_card, "Title card", height px 100, width %100);
///     // }
/// });
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
/// Concretely:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// let menu_buttons = ["Hello", "This is a", "Menu"];
/// let mut cmds = cmds.spawn_empty();
///
/// dsl!{ cmds,
///    code(let my_cmd) {
///        for n in &menu_buttons {
///            let name = format!("{n} button");
///            println!("{name}");
///            my_cmd.insert(Name::new(name));
///        }
///    }
/// }
/// ```
/// This is directly inserted as-is in the macro, so it would look as follow:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// # let menu_buttons = ["Hello", "This is a", "Menu"];
/// # let mut cmds = cmds.spawn_empty();
/// let my_cmd = &mut cmds;
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
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd();
/// # let menu_buttons = ["Hello", "This is a", "Menu"];
/// dsl!(&mut cmds,
///     row(height pct(100), fill_main_axis) {
///         code(let my_cmds) {
///             for name in &menu_buttons {
///                 dsl!(my_cmds, button(name, color Color::BLUE);)
///             }
///         }
///     }
/// )
/// ```
///
/// ## DSL methods
///
/// Stuff within parenthesis in a DSL statement are **DSL methods**.
/// DSL methods are nothing more than method calls on `Dsl`.
///
/// There are four kind of DSL methods:
/// - name literal methods
/// - bare methods
/// - single argument methods
/// - multiple arguments methods
/// - field setting method
///
/// ### Name literals
///
/// The only "special" kind of DSL method that does more than call a method
/// is the name literal argument. It simply calls a method called `named` with
/// the string literal in question… Yep. You'd argue it's nothing special, yet
/// it's the _most_ special of all DSL methods.
///
/// A name literal argument looks as follow:
/// ```text
/// <literal>
/// ```
/// Or concretely, several examples:
/// ```text
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
/// .some.nested.field <expr> // field setting method
/// method_with_arg <expr>    // single argument method
/// several_args ([<expr>],*) // multiple arguments method
/// ```
/// Which would be translated into rust code as follow:
/// ```ignore
/// x.some_method();
/// x.some.nested.field = vec![10, 34];
/// x.method_with_arg(15 * 25. as u32);
/// x.several_args("hi folks", variable_name, Color::RED);
/// ```
///
/// [`DslBundle`]: crate::DslBundle
/// [`DslBundle::insert`]: crate::DslBundle::insert
/// [`BaseDsl`]: crate::BaseDsl
/// [`IntoEntityCommands`]: crate::IntoEntityCommands
#[rustfmt::skip]
#[macro_export]
macro_rules! dsl {
    (@before_coma [$($prefix:tt)*], $($_:tt)*) => {
        stringify!($($prefix)*)
    };
    (@before_coma [$($prefix:tt)*] $head:tt $($tail:tt)*) => {
        dsl!(@before_coma [$($prefix)* $head] $($tail)*)
    };
    (@arg $x:ident, $m:ident ($($m_args:expr),*) $(,$($t:tt)*)?) => {
        $x.$m($($m_args),*) $(; dsl!(@arg $x, $($t)*))?
    };
    (@arg $x:ident, $(.$f:ident)+ $set:expr $(,$($t:tt)*)?) => {
        // #[deprecated(since = "0.9.0", note = "The Field setting method syntax is \
        //     not compatible with cuicui_chirp. To simplify DslBundle implementation, \
        //     you can now use the DslBundle derive macro.")]
        // fn field_setting_method() {};
        // field_setting_method();
        $x $(.$f)+ = $set $(; dsl!(@arg $x, $($t)*))?
    };
    (@arg $x:ident,) => {  };
    (@arg $x:ident, $nm:literal          $(,$($t:tt)*)?)=>{$x.named($nm) $(; dsl!(@arg $x, $($t)*))?};
    (@arg $x:ident, $m:ident $m_arg:expr $(,$($t:tt)*)?)=>{$x.$m($m_arg) $(; dsl!(@arg $x, $($t)*))?};
    (@arg $x:ident, $m:ident             $(,$($t:tt)*)?)=>{$x.$m()       $(; dsl!(@arg $x, $($t)*))?};
    (@arg $x:ident, $($t:tt)*)=> {
        compile_error!(concat!(
            "`", dsl!(@before_coma [] $($t)*), "` is an invalid DSL method",
            "\n\nPossible methods syntax is described at:",
            "https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html#dsl-methods"
        ));
    };

    (@statement [$d_ty:ty, $cmds:expr] ) => { };
    (@statement [$d_ty:ty, $cmds:expr] code (let $cmds_ident:ident) {$($code:tt)*} $($($t:tt)+)?) => {
        let $cmds_ident = &mut $cmds;
        $($code)*
        // Generate the rest of the code
        $(; dsl!(@statement [$d_ty, $cmds] $($t)*))?
    };
    (@statement [$d_ty:ty, $cmds:expr] spawn ($($args:tt)*) ; $($($t:tt)+)?) => {
        let mut x = <$d_ty>::default();
        dsl!(@arg x, $($args)*);
        x.insert(&mut $cmds.to_cmds());
        // Generate the rest of the code
        $(; dsl!(@statement [$d_ty, $cmds] $($t)*))?
    };
    (@statement [$d_ty:ty, $cmds:expr] spawn ($($args:tt)*) {$($inner:tt)*} $($($t:tt)+)?) => {
        let mut arg = <$d_ty>::default();
        dsl!(@arg arg, $($args)*);
        arg.node(&mut $cmds.to_cmds(), |mut cmds| {
            // Generate code for statements inside curly braces
            dsl!(@statement [$d_ty, cmds] $($inner)*);
        })
        // Generate the rest of the code
        $(; dsl!(@statement [$d_ty, $cmds] $($t)*))?
    };
    (@statement [$d_ty:ty, $cmds:expr] $leaf_node:ident ( $value:expr $(, $($args:tt)*)? ) ; $($($t:tt)+)?) => {
        let mut leaf_cmd = $cmds.to_cmds();
        let mut x = <$d_ty>::default();
        dsl!(@arg x, $($($args)*)?);
        x.insert(&mut leaf_cmd);
        x.$leaf_node($value, &mut leaf_cmd)
        // Generate the rest of the code
        $(; dsl!(@statement [$d_ty, $cmds] $($t)*))?
    };
    (@statement [$d_ty:ty, $cmds:expr] $leaf_node:ident ($($($args:tt)+)?) ; $($($t:tt)+)?) => {
        compile_error!(concat!(
            "Leaf node DSL statements (see dsl! docs) need a valid expression (<expr>) as first method \
            within parenthesis, this expression will be passed as argument to the method. \
            If you need to call method `",
            stringify!($leaf_node), "` on `", stringify!($d_ty), "` \
            make sure to use to use a leaf node in the form:\n`",
            stringify!($leaf_node), "(<expr>, ", stringify!($($($args)+)?), ");`",
            "\n\nThe precise syntax spec is available at:",
            "https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html#dsl-statements"
        ));
    };
    (@statement [$d_ty:ty, $cmds:expr] $parent_node:ident ($($($args:tt)+)?) {$($inner:tt)*} $($t:tt)*) => {
        // Call the @statement spawn with curly braces
        dsl!(@statement [$d_ty, $cmds] spawn ($($($args)+,)? $parent_node) {$($inner)*} $($t)*)
    };
    (<$builder:ty> $cmds:expr, $($t:tt)*) => {{
        use $crate::{IntoEntityCommands, DslBundle};
        fn is_dsl_bundle<D: DslBundle>() {} is_dsl_bundle::<$builder>();
        // Generate code for all statements
        dsl!(@statement [$builder, $cmds] $($t)*);
    }};
    // Just call the match above with <Dsl>
    ($cmds:expr, $($t:tt)*) => { dsl!(<Dsl> $cmds, $($t)*) };
}

#[allow(clippy::all, clippy::pedantic, clippy::nursery)]
#[doc(hidden)]
#[cfg(feature = "test_and_doc")]
pub mod __doc_helpers {
    use std::fmt;
    use std::num::ParseIntError;
    use std::str::FromStr;

    pub use crate::{BaseDsl, DslBundle, IntoEntityCommands};
    pub use bevy::ecs::system::EntityCommands;
    pub use bevy::prelude::{
        default, AssetServer, Bundle, Commands, Component, Deref, DerefMut, Entity, Handle, Image,
        Name, Res,
    };
    use bevy::{ecs::system::CommandQueue, prelude::World};

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Color;
    impl Color {
        pub const RED: Self = Self;
        pub const GREEN: Self = Self;
        pub const BLUE: Self = Self;
    }
    impl std::error::Error for Color {}
    impl fmt::Display for Color {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Color")
        }
    }
    impl FromStr for Color {
        type Err = Color;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "red" => Ok(Color),
                "green" => Ok(Color),
                "blue" => Ok(Color),
                _ => Err(Color),
            }
        }
    }

    #[derive(Bundle, Default)]
    pub struct SpatialBundle {
        f: (),
    }

    #[derive(Bundle, Default)]
    pub struct BlinkBundle {
        pub blink: Blink,
        pub bundle: SpatialBundle,
    }

    #[derive(Deref, DerefMut, Default)]
    pub struct DocDsl<C = BaseDsl> {
        #[deref]
        pub inner: C,
        pub blink: Blink,
    }
    impl<C> DocDsl<C> {
        pub fn column(&mut self) {}
        pub fn main_margin(&mut self, _: f32) {}
        pub fn align_start(&mut self) {}
        pub fn image(&mut self, _: &impl Into<ImageMock>) {}
        pub fn row(&mut self) {}
        pub fn width(&mut self, _: Val) {}
        pub fn height(&mut self, _: Val) {}
        pub fn button(&mut self, _: &str, _: &mut EntityCommands) -> Entity {
            Entity::PLACEHOLDER
        }
        pub fn screen_root(&mut self) {}
        pub fn fill_main_axis(&mut self) {}
        pub fn color(&mut self, _color: Color) {}
        pub fn spawn_ui(&mut self, _: &str, _: &mut EntityCommands) -> Entity {
            Entity::PLACEHOLDER
        }

        pub fn amplitude(&mut self, _: f32) {}
        pub fn frequency(&mut self, _: f32) {}

        pub fn distrib_start(&mut self) {}
    }
    impl<C: DslBundle> DslBundle for DocDsl<C> {
        fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
            self.inner.insert(cmds)
        }
    }
    pub type Dsl = DocDsl;
    pub type LayoutDsl = DocDsl;

    pub struct Val;
    impl FromStr for Val {
        type Err = ParseIntError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match () {
                () if s.starts_with("px(") => {
                    let number = &s[3..s.len() - 1];
                    let _ = number.parse::<i32>()?;
                    Ok(Val)
                }
                () if s.starts_with("pct(") => {
                    let number = &s[4..s.len() - 1];
                    let _ = number.parse::<i32>()?;
                    Ok(Val)
                }
                () => Err("badnumber".parse::<i32>().unwrap_err()),
            }
        }
    }
    pub fn px(_: i32) -> Val {
        Val
    }
    pub fn pct(_: i32) -> Val {
        Val
    }

    #[derive(Default, Component, Clone, Copy)]
    pub struct Blink {
        pub frequency: f32,
        pub amplitude: f32,
    }

    pub struct WorldCheck(World, CommandQueue);
    impl WorldCheck {
        pub fn new() -> Self {
            WorldCheck(World::new(), CommandQueue::default())
        }
        pub fn cmd<'a>(&'a mut self) -> Commands<'a, 'a> {
            Commands::new(&mut self.1, &self.0)
        }
        pub fn check(&self) {
            todo!(
                "This would be called with some sort of hierarchy, and \
                we would compare it to what's in the World, but not \
                implemented yet"
            )
        }
    }

    pub struct ImageMock;
    impl From<()> for ImageMock {
        fn from(_: ()) -> Self {
            ImageMock
        }
    }
    impl From<Handle<Image>> for ImageMock {
        fn from(_: Handle<Image>) -> Self {
            ImageMock
        }
    }
}
