/// Wrapper around [`MakeBundle`].
///
/// Basically, this is a way to use the methods on `LayoutType` but in a
/// declarative way.
///
/// # Syntax
///
/// `layout!` accepts as argument:
///
/// 1. (optionally) between `<$ty>`, a type implementing [`MakeBundle`].
///    By default, it will use the identifier `LayoutType` in scope.
///    This will be referred as **`LayoutType`** in the rest of this documentation.
/// 2. An expression implementing [`MakeSpawner`].
/// 3. a series of **layout statements**.
///
/// ## Layout statements
///
/// A layout statement is one of the following:
/// - `[row|column]([args]*) { [layout statement]* }`: a [`row`]/[`column`] container, `args` is
///   a series of **layout arguments**, while the content of the curly braces
///   is another series of **layout statements**.
/// - `spawn_ui(bundle, [args]*);`: a [`spawn_ui`] with provided `args` **layout arguments**.
/// - `code(let [ident]) { [rust code] }`: Insert arbitrary rust code
///   (specified between braces). `ident` is an identifier to which to set the current `cmds`.
///
/// Note that the API is extensible, `row` and `column` can be swapped with any
/// method on the `LayoutType` you provided, as long as that method has the `fn method(&mut self)`
/// signature.
///
/// ## Layout arguments
///
/// `spawn_ui`, `column` and `row` might seem familiar. Indeed, they are methods on
/// the default [`cuicui_layout::dsl::LayoutType`].
///
/// The remaining methods exist as **layout arguments**, they are specified within parenthesis
/// in a **layout statement**.
///
/// **Layout arguments** are methods on `LayoutType`.
///
/// Again, since the type in question is **provided by you**, any method on
/// `LayoutType` is accepted. As long as they have a signature similar to:
///
/// ```ignore
/// fn method(&mut self) {}
/// fn method(&mut self, argument: AnyType) {}
/// ```
///
/// The layout arguments are:
/// - `"<string literal>"`: Set the name of the UI node to spawn.
/// - `named <expr>`: Set the name of the UI node to spawn to value of `expr`
/// - `[height|width] [px|%|^] <expr>`: Set the node's height/width to value of `expr`,
///   If `px`, it is a fixed size container, while `%` is in percent of the parent,
///   And `^` is how many time larger that children.
/// - `[main_margin|cross_margin] <expr>`: Add a fixed pixel size margin
///   on the main/cross axis to value of `expr`.
/// - `[align_start|align_end]`: Align the content of the container to the start/end of it.
/// - `[screen_root|root]`: Mark this container as "root".
/// - `fill_main_axis`: distribute the content of container spaced evenly so that it fills its
///   main axis. This requires the container to have an parent-dependent or fixed size on
///   the main axis.
/// - `distrib_end`: Push the content on the main axis to the end of the container.
///   This requires the container to have an parent-dependent or fixed size on the main axis.
///
/// # Example
///
/// ```
/// use bevy::prelude::*;
/// use cuicui_layout::{Rule, layout, LayoutRootCamera, dsl::MakeBundle, dsl::LayoutType};
/// # enum BevyUi {} impl cuicui_layout::dsl::IntoUiBundle<BevyUi> for &'_ str {type Target=();fn into_ui_bundle(self) {}}
/// # fn sys(mut cmds: Commands) {
/// # let title_card = "";
/// let menu_buttons = ["CONTINUE", "QUIT", "LOAD"];
/// # let _defined_using_macro = || {
///
/// layout! {
///     &mut cmds,
///     row(screen_root, "root", main_margin 100., align_start) {
///         column("menu", width px 300, fill_main_axis) {
///             spawn_ui(title_card, "Title card", height px 100, width %100);
///             code(let cmds) {
///                 menu_buttons.iter( ).map(|button_name| {
///                     let name = format!("{button_name} button");
///                     layout!(cmds, spawn_ui(*button_name, named name, height px 30);)
///                 });
///             }
///         }
///     }
/// }
/// # };
/// // Is strictly equivalent to:
/// use cuicui_layout::dsl::MakeSpawner;
/// let mut x = <LayoutType>::default();
/// x.row();
/// x.screen_root();
/// x.named("root");
/// x.main_margin(100.0);
/// x.align_start();
/// x.node(cmds.make_spawner(), |cmds| {
///     let mut x = <LayoutType>::default();
///     x.column();
///     x.named("menu");
///     x.width_rule(Rule::Fixed(300.0));
///     x.fill_main_axis();
///     x.node(cmds.make_spawner(), |cmds| {
///         let mut x = <LayoutType>::default();
///         x.named("Title card");
///         x.height_rule(Rule::Fixed(100.0));
///         x.width_rule(Rule::Parent(1.0));
///         x.spawn_ui(cmds.make_spawner(), title_card.clone());
///
///         menu_buttons.iter().map(|button_name| {
///             let mut x = <LayoutType>::default();
///             x.named(format!("{button_name} button"));
///             x.height_rule(Rule::Fixed(30.0));
///             x.spawn_ui(cmds.make_spawner(), *button_name)
///         });
///     })
/// });
/// # }
/// ```
///
/// # How to extend yourself
///
/// We are not limited to the `cuicui_layout` `LayoutType`!
/// The trick to extend an existing set of methods is to use the `DerefMut`
/// trait.
///
/// This is a somewhat sacrilegous use of `DerefMut`, but hey it works and it
/// makes defining extensions surprisingly easy.
///
/// ```
/// use std::ops;
/// use bevy::prelude::*;
/// use cuicui_layout::{Rule, layout, Size};
/// use cuicui_layout::dsl::{MakeBundle, LayoutType, InsertKind, EntityCommands};
/// # enum BevyUi {} impl cuicui_layout::dsl::IntoUiBundle<BevyUi> for &'_ str {type Target=();fn into_ui_bundle(self) {}}
/// # fn sys(mut cmds: Commands) {
///
/// #[derive(Default)]
/// struct MyStyles<C = ()> {
///     layout: LayoutType<C>,
///     color: Color,
/// }
/// impl<T> ops::Deref for MyStyles<T> {
///     type Target = LayoutType<T>;
///     fn deref(&self) -> &Self::Target {
///         &self.layout
///     }
/// }
/// impl<T> ops::DerefMut for MyStyles<T> {
///     fn deref_mut(&mut self) -> &mut Self::Target {
///         &mut self.layout
///     }
/// }
///
/// impl<C: MakeBundle> MyStyles<C> {
///     pub fn button(&mut self, bg: Color) {
///         self.layout.height_rule(Rule::Fixed(30.0));
///         self.layout.main_margin(10.0);
///         self.color = bg;
///     }
/// }
/// impl<C: MakeBundle> MakeBundle for MyStyles<C> {
///     fn insert(self, insert: InsertKind, cmds: &mut EntityCommands) -> Entity {
///         let id = self.layout.insert(insert, cmds);
///         cmds.insert(BackgroundColor(self.color));
///         id
///     }
///     fn ui_content_axis(&self) -> Size<bool> {
///         self.layout.ui_content_axis()
///     }
/// }
///
/// # let title_card = "";
/// let menu_buttons = [("CONTINUE", Color::BLUE), ("QUIT", Color::WHITE), ("LOAD", Color::RED)];
/// layout! {
///     &mut cmds,
///     row(screen_root, "root", main_margin 100., align_start) {
///         column("menu", width px 300, fill_main_axis) {
///             spawn_ui(title_card, "Title card", height px 100, width %100);
///             code(let cmds) {
///                 menu_buttons.iter().map(|(name, color)| {
///                     /// This is important! another option is to define a
///                     /// type alias such as:
///                     /// `type LayoutType = MyStyles;`
///                     layout!(<MyStyles> cmds, spawn_ui(*name, button *color);)
///                 });
///             }
///         }
///     }
/// }
/// # }
/// ```
///
/// [`MakeBundle`]: crate::dsl::MakeBundle
/// [`MakeSpawner`]: crate::dsl::MakeSpawner
/// [`cuicui_layout::dsl::LayoutType`]: crate::dsl::LayoutType
/// [`row`]: crate::dsl::LayoutType::row
/// [`column`]: crate::dsl::LayoutType::column
/// [`spawn_ui`]: crate::dsl::MakeBundle::spawn_ui
#[rustfmt::skip]
#[macro_export]
macro_rules! layout {
    (@rule px $rule:expr) => { Rule::Fixed($rule as f32) };
    (@rule % $rule:expr) => { Rule::Parent($rule as f32 / 100.0) };
    (@rule ^ $rule:expr) => { Rule::Children($rule) };

    (@arg_end $($t:tt)*) => { $($t)* };
    (@args [$($call:tt)*] $d_ty:ty, $($args:tt)*) => {
        let mut arg = <$d_ty>::default();
        layout!(@arg arg, $($args)*);
        layout!(@arg_end arg . $($call)*)
    };

    (@arg $arg:ident,) => {  };
    (@arg $arg:ident, width $kind:tt $rul:expr $(, $($t:tt)*)? ) => {
        $arg.width_rule(layout!(@rule $kind $rul));
        layout!(@arg $arg, $($($t)*)?)
    };
    (@arg $arg:ident, height $kind:tt $rul:expr $(, $($t:tt)*)? ) => {
        $arg.height_rule(layout!(@rule $kind $rul));
        layout!(@arg $arg, $($($t)*)?)
    };
    (@arg $arg:ident, $name:literal $(,$($t:tt)*)?)             => { $arg.named($name); layout!(@arg $arg, $($($t)*)?) };
    (@arg $arg:ident, $method:ident $m_arg:expr $(,$($t:tt)*)?) => { $arg.$method($m_arg); layout!(@arg $arg, $($($t)*)?) };
    (@arg $arg:ident, $method:ident $(,$($t:tt)*)?)             => { $arg.$method(); layout!(@arg $arg, $($($t)*)?) };

    (@statement [$d_ty:ty, $cmds:expr] ) => { };
    (@statement [$d_ty:ty, $cmds:expr] code (let $cmds_ident:ident) {$($code:tt)*}  $($($t:tt)+)?) => {
        let $cmds_ident = $cmds;
        $($code)*
        $(; layout!(@statement $cmds, $d_ty, $($t)*))?
    };
    (@statement [$d_ty:ty, $cmds:expr] spawn_ui ( $value:expr $(, $($args:tt)*)? ) ; $($t:tt)*) => {
        layout!(@args [spawn_ui($cmds.make_spawner(), $value.clone() )] $d_ty, $($($args)*)?);
        layout!(@statement [$d_ty, $cmds] $($t)*)
    };
    (@statement [$d_ty:ty, $cmds:expr] $node_method:ident ($($args:tt)*) {$($inner:tt)*} $($t:tt)*) => {
        layout!(@args
            [node($cmds.make_spawner(), |mut cmds| {layout!(@statement [$d_ty, cmds] $($inner)*);} )]
            $d_ty, $($args)*, $node_method
        );
        layout!(@statement [$d_ty, $cmds] $($t)*)
    };
    (<$builder:ty> $cmds:expr, $($t:tt)*) => {{
        use $crate::dsl::{MakeBundle, MakeSpawner};
        layout!(@statement [$builder, $cmds] $($t)*);
    }};
    ($cmds:expr, $($t:tt)*) => { layout!(<LayoutType> $cmds, $($t)*) };
}
