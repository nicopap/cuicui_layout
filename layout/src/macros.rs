/// Dump wrapper around the [`crate::dsl::IntoLayoutCommands`] trait.
///
/// See [`crate::dsl::IntoLayoutCommands`] for details.
///
/// Basically, this is a way to use the methods on the dsl trait but, but reversed:
///
/// # Syntax
///
/// `layout!` accepts as argument a value that implements [`IntoLayoutCommands`],
/// followed by a series of **layout statements**.
///
/// ## Layout statements
///
/// A layout statement is one of the following:
/// - `[row|column]([args]*) { [layout statement]* }`: a [`row`]/[`column`] container, `args` is
///   a series of **layout arguments**, while the content of the curly braces
///   is another series of **layout statements**.
/// - `spawn_ui(bundle, [args]*);`: a [`spawn_ui`] with provided `args` **layout arguments**.
/// - `code(let [ident] [: &mut ChildBuilder]?) { [rust code] }`: Insert arbitrary rust code
///   (specified between braces). `ident` is an identifier to which to set the current `cmds`.
///
/// ## Layout arguments
///
/// `spawn_ui`, `column` and `row` might seem familiar. Indeed, they are methods of [`LayoutCommands`].
/// The remaining methods exist as **layout arguments**, they are specified within parenthesis
/// in a **layout statement**.
///
/// **Layout arguments** are methods on [`LayoutCommands`]. It is possible to add and access
/// additional **layout arguments** in this macro by using an extension trait and implementing it
/// for `LayoutCommands` (yep).
///
/// This is typically useful if you want to extend the layouting macro with 3rd party provided
/// components, or if you want to emulate "styles" (ie: set of presets).
///
/// The layout arguments are:
/// - `"<string literal>"`: Set the name of the UI node to spawn.
/// - `named <expr>`: Set the name of the UI node to spawn to value of `expr`
/// - `[height|width] [px|%] <expr>`: Set the node's height/width to value of `expr`,
///   If `px`, it is a fixed size container, while `%` is in percent of the parent
/// - `main_margin <expr>`: Add a fixed pixel size margin on the main axis to value of `expr`.
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
/// use cuicui_layout::{Rule, layout, LayoutRootCamera};
/// # enum BevyUi {} impl cuicui_layout::dsl::IntoUiBundle<BevyUi> for &'_ str {type Target=();fn into_ui_bundle(self) {}}
/// # fn sys(mut cmds: Commands) {
/// # let title_card = "";
/// let menu_buttons = ["CONTINUE", "QUIT", "LOAD"];
/// let mut menu_entities = Vec::<Entity>::new();
/// # let _defined_using_macro = || {
/// layout! {
///     &mut cmds,
///     row(screen_root, "root", main_margin 100., align_start) {
///         column("menu", width px 300, fill_main_axis) {
///             spawn_ui(title_card, "Title card", height px 100, width %100);
///             code(let cmds) {
///                 menu_entities.extend(menu_buttons.iter( ).map(|button_name| {
///                     let name = format!("{button_name} button");
///                     layout!(cmds, spawn_ui(*button_name, named name, height px 30);)
///                 }));
///             }
///         }
///     }
/// }
/// # };
/// // Is strictly equivalent to:
/// use cuicui_layout::dsl::IntoLayoutCommands;
/// cmds.lyout().align_start().main_margin(100.0).named("root").screen_root().row(|cmds| {
///     cmds.lyout().fill_main_axis().width_rule(Rule::Fixed(300.0)).named("menu").column(|cmds| {
///         cmds.lyout().width_rule(Rule::Parent(1.0))
///             .height_rule(Rule::Fixed(100.0))
///             .named("Title card")
///             .spawn_ui(title_card.clone());
///         menu_entities.extend(menu_buttons.iter().map(|button_name| {
///             cmds.lyout().height_rule(Rule::Fixed(30.0))
///                 .named(format!("{button_name} button"))
///                 .spawn_ui(*button_name)
///         }));
///     });
/// });
/// # }
/// ```
///
/// # This also works with extension traits
///
///
/// ```
/// use bevy::prelude::*;
/// use cuicui_layout::{Rule, layout, dsl::CommandLike, dsl::LayoutCommands};
/// # enum BevyUi {} impl cuicui_layout::dsl::IntoUiBundle<BevyUi> for &'_ str {type Target=();fn into_ui_bundle(self) {}}
/// # fn sys(mut cmds: Commands) {
///
/// trait MyStyles {
///     fn button(self, bg: Color) -> Self;
/// }
/// impl<C: CommandLike> MyStyles for LayoutCommands<C> {
///     fn button(self, bg: Color) -> Self {
///          self.height_rule(Rule::Fixed(30.0))
///              .height_rule(Rule::Fixed(30.0))
///              .main_margin(10.0)
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
///                     layout!(cmds, spawn_ui(*name, button *color);)
///                 });
///             }
///         }
///     }
/// }
/// # }
/// ```
///
/// [`IntoLayoutCommands`]: crate::dsl::IntoLayoutCommands
/// [`LayoutCommands`]: crate::dsl::LayoutCommands
/// [`row`]: crate::dsl::LayoutCommands::row
/// [`spawn_ui`]: crate::dsl::LayoutCommands::spawn_ui
/// [`column`]: crate::dsl::LayoutCommands::column
#[rustfmt::skip]
#[macro_export]
macro_rules! layout {
    (@rule px $rule:expr) => { Rule::Fixed($rule as f32) };
    (@rule % $rule:expr) => { Rule::Parent($rule as f32 / 100.0) };
    (@arg $cmds:expr,) => { $cmds.lyout() };
    (@arg $cmds:expr, width $kind:tt $rul:expr $(, $($t:tt)*)? ) => {
        layout!(@arg $cmds, $($($t)*)?).width_rule(layout!(@rule $kind $rul))
    };
    (@arg $cmds:expr, height $kind:tt $rul:expr $(, $($t:tt)*)? ) => {
        layout!(@arg $cmds, $($($t)*)?).height_rule(layout!(@rule $kind $rul))
    };
    (@arg $cmds:expr, $name:literal $(,$($t:tt)*)?)           => {layout!(@arg $cmds, $($($t)*)?).named($name)};
    (@arg $cmds:expr, $method:ident $arg:expr $(,$($t:tt)*)?) => {layout!(@arg $cmds, $($($t)*)?).$method($arg)};
    (@arg $cmds:expr, $method:ident $(,$($t:tt)*)?)           => {layout!(@arg $cmds, $($($t)*)?).$method()};
    (@statement $cmds:expr, row ($($args:tt)*) {$($inner:tt)*} $($($t:tt)+)?) => {
        { layout!(@arg $cmds, $($args)*).row( |mut cmds| { layout!(@statement cmds, $($inner)*); })
          $(; layout!(@statement $cmds, $($t)*))? }
    };
    (@statement $cmds:expr, column ($($args:tt)*) {$($inner:tt)*} $($($t:tt)+)?) => {
        { layout!(@arg $cmds, $($args)*).column( |mut cmds| { layout!(@statement cmds, $($inner)*); })
          $(; layout!(@statement $cmds, $($t)*))? }
    };
    (@statement $cmds:expr, spawn_ui ( $value:expr $(, $($args:tt)*)? ) ; $($($t:tt)+)?) => {
        { layout!(@arg $cmds, $($($args)*)?).spawn_ui( $value.clone())
          $(; layout!(@statement $cmds, $($t)*))? }
    };
    (@statement $cmds:expr, code (let $cmds_ident:ident $(: &mut ChildBuilder)?) {$($code:tt)*}  $($($t:tt)+)?) => {
        { let $cmds_ident = $cmds;
          $($code)*
          $(; layout!(@statement $cmds, $($t)*))? }
    };
    (<> $cmds:expr, $($t:tt)*) => {{
        layout!(@statement $cmds, $($t)*)
    }};
    ($cmds:expr, $($t:tt)*) => {{
        use $crate::dsl::IntoLayoutCommands;
        layout!(@statement $cmds, $($t)*)
    }};
}
