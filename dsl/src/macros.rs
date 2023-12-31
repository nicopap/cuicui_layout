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
///   - [**Entity**](#entity)
///   - [**leaf node**](#leaf-node)
///   - [**parent node**](#parent-node)
///   - [**code**](#code)
/// - [**dsl methods**](#dsl-methods)
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
/// struct BlinkDsl<D = ()> {
///     #[deref]
///     inner_dsl: D,
///     pub blink: Blink,
/// }
/// impl<D: DslBundle> DslBundle for BlinkDsl<D> {
///     fn insert(&mut self, cmds: &mut EntityCommands) {
///         // We insert first `Blink`, as to avoid overwriting things
///         // `inner_dsl.insert`  might insert itself.
///         cmds.insert(BlinkBundle { blink: self.blink, ..default() });
///         self.inner_dsl.insert(cmds);
///     }
/// }
///
/// // `dsl!` relies on method calls, so we need to define methods:
/// impl<D> BlinkDsl<D> {
///     pub fn frequency(&mut self, frequency: f32) {
///         self.blink.frequency = frequency;
///     }
///     pub fn amplitude(&mut self, amplitude: f32) {
///         self.blink.amplitude = amplitude;
///     }
/// }
///
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// type Dsl = BlinkDsl<BaseDsl>;
/// dsl! {
///     &mut cmds,
///     Entity {
///         FastBlinker(frequency(0.5))
///         SlowBlinker(amplitude(2.) frequency(3.0))
///     }
/// }
/// ```
///
/// If we want to use a pre-existing DSL with ours, we would nest them.
/// Since we `#[deref] inner: D`, all methods on the inner DSL are available
/// on the outer DSL.
///
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl; type BlinkDsl<T> = DocDsl<T>;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// type Dsl = BlinkDsl<LayoutDsl>;
/// dsl! {
///     &mut cmds,
///     Entity {
///         Entity(ui("Fast blink") frequency(0.5) color(Color::GREEN))
///         Entity(row frequency(1.) amplitude(1.0) main_margin(10.) fill_main_axis) {
///             Entity(ui("Some text") amplitude(10.0) color(Color::BLUE))
///         }
///         Entity(ui("Slow blink") frequency(2.) color(Color::RED))
///     }
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
/// 2. An expression of type `&mut EntityCommands`.
/// 3. A single [**DSL statement**](#dsl-statements).
///    * DSL statements contain themselves series of [**DSL methods**](#dsl-methods).
///
/// ## DSL statements
///
/// A DSL statement spawns a single entity.
///
/// There are three kinds of DSL statements:
/// - Entity statements
/// - leaf node statement
/// - parent node statement
/// - code statement
///
/// ### Entity
///
/// Entity statements create an `Entity` and calls [`DslBundle::insert`].
/// They basically spawn an entity with the given [**DSL methods**](#dsl-methods).
///
/// Optionally, they can act like parent nodes if they are directly followed
/// by curly braces:
///
/// ```text
/// Entity([dsl methods]*)
/// Entity([dsl methods]*) {
///     [dsl statements]*
/// }
/// ```
/// Concretely:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// dsl!{ &mut cmds,
///     Entity(color(Color::BLUE) rules(px(40), pct(100)))
/// };
/// dsl!{ &mut cmds,
///     Entity(fill_main_axis) {
///         Entity(color(Color::GREEN))
///     }
/// };
/// ```
/// This will expand to the following code:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// let mut x = <Dsl>::default();
/// x.color(Color::BLUE);
/// x.rules(px(40), pct(100));
/// x.insert(&mut cmds);
///
/// let mut x = <Dsl>::default();
/// x.fill_main_axis();
/// x.node(&mut cmds, |cmds| {
///     let mut x = <Dsl>::default();
///     x.color(Color::GREEN);
///     x.insert(&mut cmds.spawn_empty());
/// });
/// ```
///
/// ### Leaf node
///
/// Leaf node statements are statements without subsequent braces.
///
/// The head identifier is used as the spawned entity's name. You may also use
/// any [**rust literal**][literal] (including strings) instead of an identifier.
///
/// It looks as follow:
///
/// ```text
/// <ident>([dsl methods]*)
/// ```
/// Concretely:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// # dsl!{ &mut cmds, Entity {
/// ButtonText(color(Color::BLUE) width(px(40)) height(pct(100)) button_named)
/// # } }
/// ```
/// This expands to:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// let mut x = <Dsl>::default();
/// let mut c: &mut EntityCommands = &mut cmds;
/// x.named("ButtonText");
/// x.color(Color::BLUE);
/// x.width(px(40));
/// x.height(pct(100));
/// x.button_named();
/// x.insert(c);
/// ```
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
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let (bg, board) = ((),()); let mut cmds = cmds.spawn_empty();
/// # dsl!{ &mut cmds,
/// Root(screen_root main_margin(100.) align_start image(&bg) row) {
///     ButtonText1(color(Color::BLUE) rules(px(40), pct(100)) button_named)
///     ButtonText2(color(Color::RED) rules(px(40), pct(100)) button_named)
///     Menu(width(px(310)) main_margin(10.) fill_main_axis image(&board) column) {
///         TitleCard(rules(pct(100), px(100)))
///     }
/// }
/// # }
/// ```
///
/// The part between parenthesis (`()`) is a list of [DSL methods](#dsl-methods).
/// They are applied to the `Dsl` [`DslBundle`] each one after the other.
/// Then, an entity is spawned with the so-constructed bundle,
/// following, the DSL statements within braces (`{}`) are spawned
/// as children of the parent node entity.
///
/// For the visually-minded, this is how the previous code would look like without
/// the macro:
///
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let bg = (); let mut cmds = cmds.spawn_empty();
/// let mut x = <Dsl>::default();
/// x.named("Root");
/// x.screen_root();
/// x.main_margin(100.);
/// x.align_start();
/// x.image(&bg);
/// x.row();
/// x.node(&mut cmds, |cmds| {
///     // Same goes with the children:
///     // ButtonText1(color(Color::BLUE) rules(px(40), pct(100)) button_named)
///     // ButtonText2(color(Color::RED) rules(px(40), pct(100)) button_named)
///     // Menu(width(px(310)) main_margin(10.) fill_main_axis image(&board) column) {
///     //     TitleCard(rules(pct(100), px(100)))
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
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// let menu_buttons = ["Hello", "This is a", "Menu"];
///
/// dsl!{ &mut cmds,
///    code(let my_cmds) {
///        my_cmds.with_children(|mut cmds| {
///            for n in &menu_buttons {
///                let name = format!("{n} button");
///                println!("{name}");
///                cmds.spawn(Name::new(name));
///            }
///        });
///    }
/// }
/// ```
/// This is directly inserted as-is in the macro, so it would look as follow:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// # let menu_buttons = ["Hello", "This is a", "Menu"];
/// let my_cmd = &mut cmds;
/// my_cmd.with_children(|mut cmds| {
///     for n in &menu_buttons {
///         let name = format!("{n} button");
///         println!("{name}");
///         cmds.spawn(Name::new(name));
///     }
/// });
/// ```
/// Nothing prevents you from using `code` inside a parent node,
/// neither using the `dsl!` macro within rust code within a code statement:
/// ```
/// # use cuicui_dsl::macros::__doc_helpers::*; use cuicui_dsl::dsl;
/// # let mut w = WorldCheck::new(); let mut cmds = w.cmd(); let mut cmds = cmds.spawn_empty();
/// # let menu_buttons = ["Hello", "This is a", "Menu"];
/// dsl!(&mut cmds,
///     Entity(height(pct(100)) fill_main_axis row) {
///         code(let my_cmds) {
///             my_cmds.with_children(|mut cmds| {
///                 for name in &menu_buttons {
///                     let mut entity = cmds.spawn_empty();
///                     dsl!(&mut entity, Entity(button(name) color(Color::BLUE)))
///                 }
///             });
///         }
///     }
/// )
/// ```
///
/// ## DSL methods
///
/// Stuff within parenthesis in a DSL statement are **DSL methods**.
/// Methods are translated directly into rust method calls on `Dsl`:
///
/// ```text
/// some_method                   // bare method
/// method_with_args ([<expr>],*) // arguments method
/// ```
/// Which would be translated into rust code as follow:
/// ```ignore
/// x.some_method();
/// x.method_with_args(15 * 25. as u32);
/// x.method_with_args("hi folks", variable_name, Color::RED);
/// ```
///
/// [literal]: https://doc.rust-lang.org/reference/expressions/literal-expr.html
/// [`DslBundle`]: crate::DslBundle
/// [`DslBundle::insert`]: crate::DslBundle::insert
/// [`BaseDsl`]: crate::BaseDsl
/// [`IntoEntityCommands`]: crate::IntoEntityCommands
#[rustfmt::skip]
#[macro_export]
macro_rules! dsl {
    (@arg [$x:tt] ) => {};
    (@arg [$x:tt] $m:ident ($($arg:tt)*) $($t:tt)*)=>{$x.$m($($arg)*) ; dsl!(@arg [$x] $($t)*)};
    (@arg [$x:tt] $m:ident               $($t:tt)*)=>{$x.$m()         ; dsl!(@arg [$x] $($t)*)};

    (@statement [$d_ty:ty, $cmds:expr] ) => { };
    (@statement [$d_ty:ty, $cmds:expr] code (let $cmds_ident:ident) {$($code:tt)*} $($($t:tt)+)?) => {
        let mut $cmds_ident: &mut EntityCommands = $cmds;
        $($code)*
        // Generate the rest of the code
        $(; dsl!(@statement [$d_ty, $cmds] $($t)*))?
    };
    (@statement [$d_ty:ty, $cmds:expr] Entity ($($args:tt)*) {} $($t:tt)*) => {
        let mut x = <$d_ty>::default();
        dsl!(@arg [x] $($args)*);
        x.insert($cmds);
        // Generate the rest of the code
        dsl!(@statement [$d_ty, $cmds] $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] Entity ($($args:tt)*) {$($inner:tt)*} $($t:tt)*) => {
        let mut x = <$d_ty>::default();
        dsl!(@arg [x] $($args)*);
        x.node($cmds, |mut child_builder| {
            // Generate code for statements inside curly braces
            dsl!(@statement [$d_ty, &mut child_builder.spawn_empty()] $($inner)*);
        });
        // Generate the rest of the code
        dsl!(@statement [$d_ty, $cmds] $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] spawn ($($args:tt)*) $($t:tt)*) => { // spawn: requires trailing ()
        dsl!(@statement [$d_ty, $cmds] Entity ($($args)*) $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] Entity ($($args:tt)*) $($t:tt)*) => { // no {}
        dsl!(@statement [$d_ty, $cmds] Entity ($($args)*) {} $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] Entity $($t:tt)*) => { // no ()
        dsl!(@statement [$d_ty, $cmds] Entity () $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] $entity_name:literal ($($args:tt)*) $($t:tt)*) => {
        dsl!(@statement [$d_ty, $cmds] Entity (named($entity_name.to_string()) $($args)*) $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] $entity_name:literal $($t:tt)*) => {
        dsl!(@statement [$d_ty, $cmds] Entity (named($entity_name.to_string())) $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] $entity_name:ident ($($args:tt)*) $($t:tt)*) => {
        dsl!(@statement [$d_ty, $cmds] Entity (named(stringify!($entity_name)) $($args)*) $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] $entity_name:ident $($t:tt)*) => {
        dsl!(@statement [$d_ty, $cmds] Entity (named(stringify!($entity_name))) $($t)*)
    };
    (<$builder:ty> $cmds:expr, $($t:tt)*) => {{
        use $crate::{DslBundle, EntityCommands};
        fn is_dsl_bundle<D: DslBundle>() {} is_dsl_bundle::<$builder>();
        let cmds: &mut EntityCommands = $cmds;
        // Generate code for all statements
        dsl!(@statement [$builder, cmds] $($t)*);
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

    pub use crate::{BaseDsl, BuildChildren, ChildBuilder, DslBundle};
    pub use bevy::ecs::system::EntityCommands;
    pub use bevy::prelude::{
        default, AssetServer, Bundle, Commands, Component, Deref, DerefMut, Handle, Image, Name,
        Res, Transform,
    };
    use bevy::{ecs::system::CommandQueue, prelude::World};

    #[derive(Component, Default, Clone)]
    pub struct Style {
        pub height: Val,
        pub flex_direction: FlexDirection,
    }
    #[derive(Default, Clone)]
    pub enum FlexDirection {
        #[default]
        Column,
    }
    #[derive(Component, Default)]
    pub struct BackgroundColor(pub Color);

    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct Color;
    impl Color {
        pub const WHITE: Self = Self;
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
    pub struct DocDsl<D = BaseDsl> {
        #[deref]
        pub inner: D,
        pub blink: Blink,
    }
    impl<D> DocDsl<D> {
        pub fn column(&mut self) {}
        pub fn main_margin(&mut self, _: f32) {}
        pub fn align_start(&mut self) {}
        pub fn image(&mut self, _: &impl Into<ImageMock>) {}
        pub fn row(&mut self) {}
        pub fn width(&mut self, _: Val) {}
        pub fn height(&mut self, _: Val) {}
        pub fn rules(&mut self, _: Val, _: Val) {}
        pub fn button(&mut self, _: &str) {}
        pub fn button_named(&mut self) {}
        pub fn screen_root(&mut self) {}
        pub fn fill_main_axis(&mut self) {}
        pub fn color(&mut self, _color: Color) {}
        pub fn ui(&mut self, _: &str) {}

        pub fn amplitude(&mut self, _: f32) {}
        pub fn frequency(&mut self, _: f32) {}

        pub fn distrib_start(&mut self) {}
    }
    impl<D: DslBundle> DslBundle for DocDsl<D> {
        fn insert(&mut self, cmds: &mut EntityCommands) {
            self.inner.insert(cmds);
        }
    }
    pub type Dsl = DocDsl;
    pub type LayoutDsl = DocDsl;
    pub type BlinkDsl = DocDsl;

    #[derive(Default, Clone, Copy)]
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
