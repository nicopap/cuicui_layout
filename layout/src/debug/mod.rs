//! Debug overlay for `cuicui_layout`.
//!
//! See [`Plugin`].
//!
//! > **IMPORTANT**: If you are using `cuicui_layout` but not `cuicui_layout_bevy_ui`,
//! > and the outlines are drawn behind the UI, enable the `cuicui_layout/debug_bevy_ui`!
//!
#![doc = include_str!("../../debug.md")]
#![allow(clippy::needless_pass_by_value)]

use core::fmt;

use bevy::{
    core::DebugNameItem,
    core_pipeline::clear_color::ClearColorConfig,
    ecs::{prelude::*, query::Has, system::SystemParam},
    prelude::{
        default, info, warn, Camera, Camera2d, Camera2dBundle, Children, Color, DebugName,
        GizmoConfig, Gizmos, Input, KeyCode, Name, OrthographicProjection, Plugin as BevyPlugin,
        Update, Vec2,
    },
    render::view::RenderLayers,
    window::{PrimaryWindow, Window},
};

use crate::{
    direction::Axis, Alignment, Container, Distribution, Flow, LayoutRect, LayoutRootCamera,
    LeafRule, Node, Root, Rule, ScreenRoot, Size,
};

mod inset;
mod text;

use inset::InsetGizmo;
use text::TextGizmo;

pub use enumset::{EnumSet, EnumSetType};

/// The [`Camera::order`] index used by the layout debug camera.
pub const LAYOUT_DEBUG_CAMERA_ORDER: isize = 255;
/// The [`RenderLayers`] used by the debug gizmos and the debug camera.
pub const LAYOUT_DEBUG_LAYERS: RenderLayers = RenderLayers::none().with(16);
const MAX_TEXT_OFFSET_RATIO: f32 = 1.5;

/// For some reasons, gizmo lines' size is divided by 1.5, absolutely no idea why.
const MARGIN_LIGHTNESS: f32 = 0.85;
const NODE_LIGHTNESS: f32 = 0.7;
const NODE_SATURATION: f32 = 0.8;
const CHEVRON_RATIO: f32 = 1. / 4.;

// TODO(clean) shitty name
struct Gizmodor<'w, 's> {
    inset: InsetGizmo<'w, 's>,
    text: TextGizmo<'w, 's>,
}
impl<'w, 's> Gizmodor<'w, 's> {
    fn clear_scope(&mut self, rect: LayoutRect, margin: Size<f32>) {
        self.inset.clear_scope(rect, margin);
    }

    fn new(
        draw: Gizmos<'s>,
        cam: Query<'w, 's, (&'static Camera, &'static DebugOverlayCamera)>,
        line_width: f32,
        text: TextGizmo<'w, 's>,
    ) -> Self {
        Self {
            inset: InsetGizmo::new(draw, cam, line_width),
            text,
        }
    }

    pub(super) fn set_scope(&mut self, rect: LayoutRect, margin: Size<f32>) {
        self.inset.set_scope(rect, margin);
    }

    pub(super) fn rect_2d(&mut self, rect: LayoutRect, margin: Size<f32>, color: Color) {
        self.inset.rect_2d(rect, margin, color);
    }

    pub(super) fn rule(
        &mut self,
        center: Vec2,
        extents: Vec2,
        rule: RuleArrow,
        axis: Axis,
        color: Color,
    ) {
        self.inset.rule(center, extents, rule, axis, color);
    }
}

#[allow(clippy::cast_precision_loss)]
fn hue_from_entity(entity: Entity) -> f32 {
    const FRAC_U32MAX_GOLDEN_RATIO: u32 = 2_654_435_769; // (u32::MAX / Φ) rounded up
    const RATIO_360: f32 = 360.0 / u32::MAX as f32;
    entity.index().wrapping_mul(FRAC_U32MAX_GOLDEN_RATIO) as f32 * RATIO_360
}

/// The Kind of debug overlays available in `cuicui_layout`.
#[derive(EnumSetType, Debug)]
pub enum Flag {
    /// Show layout node outlines, and their margin as lighter color.
    Outlines,
    /// Show rules as arrows, and rule percentages/ratio as numbers on top
    /// of them.
    ///
    /// - [`Rule::Children`] are arrows pointing from edge of container inward
    /// - [`LeafRule::Fixed`] (content-sized), like above, but without number
    /// - [`Rule::Parent`], [`LeafRule::Parent`] are arrows pointing toward the edge of container
    /// - [`Rule::Fixed`], [`LeafRule::Fixed`] (not content-sized) are not shown.
    Rules,
    /// Hold shift to see detailed information about hovered container as tooltip.
    ///
    /// Currently unused.
    Tooltips,
    /// If there is room, just inline this information.
    InfoText,
}

/// The inputs used by the `cuicui_layout` debug overlay.
#[derive(Resource, Clone)]
pub struct InputMap {
    /// The key used for swapping between overlays, default is [`KeyCode::Space`].
    pub cycle_debug_flag: KeyCode,
}
impl Default for InputMap {
    fn default() -> Self {
        InputMap { cycle_debug_flag: KeyCode::Space }
    }
}

#[derive(Component, Debug, Clone, Default)]
struct DebugOverlayCamera {
    screen_space: bool,
}
impl DebugOverlayCamera {
    #[must_use]
    const fn with_options(options: &Options) -> Self {
        Self { screen_space: options.screen_space }
    }
}

/// The debug overlay options.
#[derive(Resource, Clone, Default)]
pub struct Options {
    /// Which overlays are set.
    pub flags: EnumSet<Flag>,
    /// The inputs used by the debug overlay.
    pub input_map: InputMap,
    /// Whether the debug overlay should be rendered in screen space
    /// or world space.
    ///
    /// This is usually `false` if not using cuicui_layout with bevy_ui.
    pub screen_space: bool,
    layout_gizmos_camera: Option<Entity>,
}

fn update_debug_camera(
    mut gizmo_config: ResMut<GizmoConfig>,
    mut options: ResMut<Options>,
    mut cmds: Commands,
    mut debug_cams: Query<&mut Camera, (Without<LayoutRootCamera>, With<DebugOverlayCamera>)>,
) {
    if !options.is_changed() && !gizmo_config.is_changed() {
        return;
    }
    if options.flags.is_empty() {
        let Some(cam) = options.layout_gizmos_camera  else {return;};
        let Ok(mut cam) = debug_cams.get_mut(cam) else {return;};
        cam.is_active = false;
        gizmo_config.render_layers = RenderLayers::all();
    } else {
        let debug_overlay_camera = DebugOverlayCamera::with_options(&options);
        let spawn_cam = || {
            cmds.spawn((
                #[cfg(feature = "debug_bevy_ui")]
                bevy::prelude::UiCameraConfig { show_ui: false },
                Camera2dBundle {
                    projection: OrthographicProjection {
                        far: 1000.0,
                        viewport_origin: Vec2::new(0.0, 0.0),
                        ..default()
                    },
                    camera: Camera { order: LAYOUT_DEBUG_CAMERA_ORDER, ..default() },
                    camera_2d: Camera2d { clear_color: ClearColorConfig::None },
                    ..default()
                },
                LAYOUT_DEBUG_LAYERS,
                debug_overlay_camera,
                Name::new("Layout Debug Camera"),
            ))
            .id()
        };
        gizmo_config.enabled = true;
        gizmo_config.depth_bias = 1.0;
        gizmo_config.render_layers = LAYOUT_DEBUG_LAYERS;
        let cam = *options.layout_gizmos_camera.get_or_insert_with(spawn_cam);
        let Ok(mut cam) = debug_cams.get_mut(cam) else {return;};
        cam.is_active = true;
    }
}

fn cycle_flags(
    input: Res<Input<KeyCode>>,
    mut options: ResMut<Options>,
    map: Res<InputMap>,
    mut text: TextGizmo,
) {
    use Flag::{InfoText, Outlines, Rules};
    let cycle: [EnumSet<Flag>; 4] = [
        EnumSet::EMPTY,
        Outlines.into(),
        Outlines | Rules,
        Outlines | Rules | InfoText,
    ];
    if input.just_pressed(map.cycle_debug_flag) {
        let current = cycle.iter().position(|f| *f == options.flags).unwrap_or(0);
        let next = cycle[(current + 1) % cycle.len()];
        info!("Setting layout debug mode to {:?}", next);
        if options.flags.contains(InfoText) && !next.contains(InfoText) {
            text.reset();
        }
        if next.contains(Outlines) {
            info!(
                "Displaying the outline of layout nodes. \
                Node boundaries are dark while node margins are light"
            );
        }
        if next.contains(Rules) {
            info!(
                "Displaying the layout nodes rules. Explanations: \
                each node have arrows pointing in or out on their sides. \
                **outward arrows**: the axis' size depends on the parent node. \
                **inward arrows**: the axis' size depends on its children or content. \
                **no arrows**: the axis' size is completely fixed."
            );
        }
        if next.contains(InfoText) {
            info!(
                "Displaying layout info, info per line: \
                (1) the entity name/id \
                (2) <width>x<height> + <x>,<y> \
                (3) the layout spec (see <https://docs.rs/cuicui_layout/latest/cuicui_layout/dsl/struct.LayoutDsl.html#method.layout>)");
        }
        options.flags = next;
    }
}

const fn node_margin(node: &Node) -> Size<f32> {
    match node {
        Node::Container(c) => c.margin,
        _ => Size::ZERO,
    }
}
fn node_rules(flow: Flow, node: &Node) -> Size<RuleArrow> {
    match node {
        Node::Container(c) => c.rules.map_into(),
        Node::Axis(oriented) => flow.absolute(*oriented).map_into(),
        Node::Box(absolute) => absolute.map_into(),
    }
}
fn outline_nodes(
    outline: &OutlineParam,
    draw: &mut Gizmodor,
    flow: Flow,
    debug_name: &DebugNameItem,
    this: LayoutRect,
) {
    let this_entity = debug_name.entity;
    let Ok(to_iter) = outline.children.get(this_entity) else { return; };
    for (debug_name, node, child) in outline.nodes.iter_many(to_iter) {
        let infos = if let Node::Container(c) = node { Some(c.into()) } else { None };
        let rules = node_rules(flow, node);
        let margin = node_margin(node);
        let mut rect = *child;
        rect.pos.width += this.pos.width;
        rect.pos.height += this.pos.height;
        let flags = outline.flags();
        outline_node(&debug_name, infos, rect, margin, rules, flags, draw);

        if let Node::Container(c) = node {
            outline_nodes(outline, draw, c.flow, &debug_name, rect);
        }
        if outline.flags().contains(Flag::Outlines) {
            draw.clear_scope(rect, margin);
        }
    }
}
#[derive(SystemParam)]
struct OutlineParam<'w, 's> {
    gizmo_config: Res<'w, GizmoConfig>,
    options: Res<'w, Options>,
    children: Query<'w, 's, &'static Children>,
    nodes: Query<'w, 's, (DebugName, &'static Node, &'static LayoutRect)>,
}
impl OutlineParam<'_, '_> {
    fn flags(&self) -> EnumSet<Flag> {
        self.options.flags
    }
}
type CameraQuery<'w, 's> = Query<'w, 's, (&'static Camera, &'static DebugOverlayCamera)>;

#[allow(clippy::cast_possible_truncation)] // The `window_scale` don't usually require f64 precision.
fn outline_roots(
    outline: OutlineParam,
    draw: Gizmos,
    text: TextGizmo,
    cam: CameraQuery,
    roots: Query<(DebugName, &Root, &LayoutRect, Has<ScreenRoot>)>,
    window: Query<&Window, With<PrimaryWindow>>,
    nonprimary_windows: Query<&Window, Without<PrimaryWindow>>,
) {
    if !nonprimary_windows.is_empty() {
        warn!(
            "The layout debug view only uses the primary window scale, \
            you might notice gaps between container lines"
        );
    }
    let scale_factor = Window::scale_factor;
    let window_scale = window.get_single().map_or(1., scale_factor) as f32;
    let line_width = outline.gizmo_config.line_width / window_scale;
    let mut draw = Gizmodor::new(draw, cam, line_width, text);
    for (name, root, rect, is_screen) in &roots {
        if !root.debug {
            continue;
        }
        let margin = root.node.margin;
        let rules = root.node.rules.map_into();
        if is_screen {
            // inset so that the root container is fully visible.
            draw.set_scope(*rect, Size::ZERO);
        }
        let flags = outline.flags();
        let info = Some(ContainerInfo::from(&root.node));
        outline_node(&name, info, *rect, margin, rules, flags, &mut draw);

        let flow = root.node.flow;
        outline_nodes(&outline, &mut draw, flow, &name, *rect);
    }
}
struct ContainerInfo {
    flow: Flow,
    distrib: Distribution,
    align: Alignment,
}
impl<'a> From<&'a Container> for ContainerInfo {
    fn from(value: &'a Container) -> Self {
        Self {
            flow: value.flow,
            distrib: value.distrib,
            align: value.align,
        }
    }
}
impl fmt::Display for ContainerInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.flow {
            Flow::Horizontal => write!(f, ">")?,
            Flow::Vertical => write!(f, "v")?,
        }
        write!(f, "d")?;
        match self.distrib {
            Distribution::Start => write!(f, "S")?,
            Distribution::FillMain => write!(f, "C")?,
            Distribution::End => write!(f, "E")?,
        }
        write!(f, "a")?;
        match self.align {
            Alignment::Start => write!(f, "S")?,
            Alignment::Center => write!(f, "C")?,
            Alignment::End => write!(f, "E")?,
        }
        Ok(())
    }
}
struct Describe<'a, 'b, 'c> {
    debug_name: &'a DebugNameItem<'b>,
    rect: &'c LayoutRect,
    infos: Option<ContainerInfo>,
}
impl<'a, 'b, 'c> Describe<'a, 'b, 'c> {
    const fn new(
        debug_name: &'a DebugNameItem<'b>,
        rect: &'c LayoutRect,
        infos: Option<ContainerInfo>,
    ) -> Self {
        Describe { debug_name, rect, infos }
    }
}
impl fmt::Display for Describe<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.debug_name.name {
            Some(name) => writeln!(f, "{name}")?,
            None => writeln!(f, "{:?}", self.debug_name.entity)?,
        }
        let r = self.rect;
        let (width, height, x, y) = (r.size.width, r.size.height, r.pos().x, r.pos().y);
        writeln!(f, "{width:.0}x{height:.0} ({x:.0},{y:.0})")?;
        if let Some(infos) = &self.infos {
            writeln!(f, "{infos}")?;
        }
        Ok(())
    }
}
fn outline_node(
    debug_name: &DebugNameItem,
    infos: Option<ContainerInfo>,
    rect: LayoutRect,
    margin: Size<f32>,
    rules: Size<RuleArrow>,
    flags: EnumSet<Flag>,
    draw: &mut Gizmodor,
) {
    let entity = debug_name.entity;
    let hue = hue_from_entity(entity);
    let main_color = Color::hsl(hue, NODE_SATURATION, NODE_LIGHTNESS);
    let margin_color = Color::hsl(hue, NODE_SATURATION, MARGIN_LIGHTNESS);

    if flags.contains(Flag::InfoText) {
        // TODO(perf)
        let text = Describe::new(debug_name, &rect, infos).to_string();
        // let margin = Vec2::splat(TEXT_MARGIN);
        let pos = rect.pos() + Vec2::Y * rect.size().height;
        let max_offset = rect.size.height * MAX_TEXT_OFFSET_RATIO;
        draw.text.print(entity, &text, pos, max_offset, main_color);
    }
    if flags.contains(Flag::Outlines) {
        // first draw margins, as we will draw the actual outline on top
        draw.rect_2d(rect, margin, margin_color);
        draw.rect_2d(rect, Size::ZERO, main_color);
        draw.set_scope(rect, margin);
    }
    if flags.contains(Flag::Rules) {
        let extents = Vec2::from(rect.size()) / 2.;
        let center = rect.pos() + extents;

        draw.rule(center, extents, rules.width, Axis::Horizontal, main_color);
        draw.rule(center, extents, rules.height, Axis::Vertical, main_color);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum RuleArrow {
    Outward(f32),
    Inward(f32),
    InwardBare,
    None,
}
impl RuleArrow {
    fn arrange<T>(self, inner: T, outer: T) -> Option<(T, T, Option<f32>)> {
        match self {
            RuleArrow::Outward(v) => Some((inner, outer, Some(v))),
            RuleArrow::Inward(v) => Some((outer, inner, Some(v))),
            RuleArrow::InwardBare => Some((outer, inner, None)),
            RuleArrow::None => None,
        }
    }
}
impl From<LeafRule> for RuleArrow {
    fn from(value: LeafRule) -> Self {
        match value {
            LeafRule::Content(_) => RuleArrow::InwardBare,
            LeafRule::Fixed(_) => RuleArrow::None,
            LeafRule::Parent(value) => RuleArrow::Outward(value),
        }
    }
}
impl From<Rule> for RuleArrow {
    fn from(value: Rule) -> Self {
        match value {
            Rule::Fixed(_) => RuleArrow::None,
            Rule::Parent(value) => RuleArrow::Outward(value),
            Rule::Children(value) => RuleArrow::Inward(value),
        }
    }
}

/// The debug overlay plugin.
///
/// This spawns a new camera with a low order, and draws gizmo.
///
/// Note that while the debug plugin is enabled, gizmos cannot be used by other
/// cameras (!)
///
/// > **IMPORTANT**: If you are using `cuicui_layout` but not `cuicui_layout_bevy_ui`,
/// > and the outlines are drawn behind the UI, enable the `cuicui_layout/debug_bevy_ui`!
///
/// disabling the plugin will give you back gizmo control.
pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<InputMap>().add_systems(
            Update,
            (
                cycle_flags,
                update_debug_camera,
                outline_roots.after(crate::ComputeLayoutSet),
                text::overlay_dark_background,
                |mut u: TextGizmo| u.update(),
            )
                .chain(),
        );
        app.init_resource::<text::ImmediateTexts>()
            .insert_resource(Options {
                screen_space: cfg!(feature = "debug_bevy_ui"),
                ..default()
            });
    }
    fn finish(&self, _app: &mut bevy::prelude::App) {
        info!(
            "The cuicui_layout debug overlay is active!\n\
            ----------------------------------------------\n\
            \n\
            This will show the outline of layout nodes.\n\
            Press `Space` to switch between debug mods."
        );
    }
}
