//! Debug overlay for `cuicui_layout`.
//!
//! See [`Plugin`].
//!
//! > **IMPORTANT**: If you are using `cuicui_layout` but not `cuicui_layout_bevy_ui`,
//! > and the outlines are drawn behind the UI, enable the `cuicui_layout/debug_bevy_ui`!
//!
#![doc = include_str!("../debug.md")]
#![allow(clippy::needless_pass_by_value)]

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    ecs::{prelude::*, query::Has, system::SystemParam},
    math::Vec2Swizzles,
    prelude::{
        default, info, warn, BVec2, Camera, Camera2d, Camera2dBundle, Children, Color, GizmoConfig,
        Gizmos, GlobalTransform, Input, KeyCode, Name, OrthographicProjection,
        Plugin as BevyPlugin, Update, Vec2,
    },
    render::view::RenderLayers,
    utils::HashSet,
    window::{PrimaryWindow, Window},
};

use crate::{
    direction::Axis, Flow, LayoutRootCamera, LeafRule, Node, PosRect, Root, Rule, ScreenRoot, Size,
};

pub use enumset::{EnumSet, EnumSetType};

/// The [`Camera::order`] index used by the layout debug camera.
pub const LAYOUT_DEBUG_CAMERA_ORDER: isize = 255;
/// The [`RenderLayers`] used by the debug gizmos and the debug camera.
pub const LAYOUT_DEBUG_LAYERS: RenderLayers = RenderLayers::none().with(16);

/// For some reasons, gizmo lines' size is divided by 1.5, absolutely no idea why.
const MARGIN_LIGHTNESS: f32 = 0.85;
const NODE_LIGHTNESS: f32 = 0.7;
const NODE_SATURATION: f32 = 0.8;
const CHEVRON_RATIO: f32 = 1. / 4.;

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
    ///
    /// Currently unused.
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
        gizmo_config.render_layers = LAYOUT_DEBUG_LAYERS;
        let cam = *options.layout_gizmos_camera.get_or_insert_with(spawn_cam);
        let Ok(mut cam) = debug_cams.get_mut(cam) else {return;};
        cam.is_active = true;
    }
}

fn cycle_flags(input: Res<Input<KeyCode>>, mut options: ResMut<Options>, map: Res<InputMap>) {
    use Flag::{Outlines, Rules};
    let cycle: [EnumSet<Flag>; 3] = [EnumSet::EMPTY, Outlines.into(), Outlines | Rules];
    if input.just_pressed(map.cycle_debug_flag) {
        let current = cycle.iter().position(|f| *f == options.flags).unwrap_or(0);
        let next = cycle[(current + 1) % cycle.len()];
        info!("Setting layout debug mode to {:?}", next);
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
        options.flags = next;
    }
}
/// Collection of axis aligned "lines" (actually just their coordinate on
/// a given axis).
#[derive(Debug, Clone)]
struct DrawnLines {
    lines: HashSet<i64>,
    width: f32,
}
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
impl DrawnLines {
    fn new(width: f32) -> Self {
        DrawnLines { lines: HashSet::new(), width }
    }
    /// Return `value` offset by as many `increment`s as necessary to make it
    /// not overlap with already drawn lines.
    fn inset(&self, value: f32, increment: i64) -> f32 {
        let scaled = value / self.width;
        let fract = scaled.fract();
        let mut on_grid = scaled.floor() as i64;
        loop {
            if !self.lines.contains(&on_grid) {
                return ((on_grid as f32) + fract) * self.width;
            }
            on_grid += increment;
        }
    }
    /// Remove a line from the collection of drawn lines.
    ///
    /// Typically, we only care for pre-existing lines when drawing the children
    /// of a container, nothing more. So we remove it after we are done with
    /// the children.
    fn remove(&mut self, value: f32, increment: i64) {
        let mut on_grid = (value / self.width).floor() as i64;
        loop {
            let next_cell = on_grid + increment;
            if !self.lines.contains(&next_cell) {
                self.lines.remove(&on_grid);
                return;
            }
            on_grid = next_cell;
        }
    }
    /// Add a line from the collection of drawn lines.
    fn add(&mut self, value: f32, increment: i64) {
        let mut on_grid = (value / self.width).floor() as i64;
        loop {
            let did_not_exist = self.lines.insert(on_grid);
            if did_not_exist {
                return;
            }
            on_grid += increment;
        }
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
    draw: &mut InsetGizmo,
    flow: Flow,
    this_entity: Entity,
    this: PosRect,
) {
    let Ok(to_iter) = outline.children.get(this_entity) else { return; };
    for (entity, node, child) in outline.nodes.iter_many(to_iter) {
        let rules = node_rules(flow, node);
        let margin = node_margin(node);
        let mut rect = *child;
        rect.pos.width += this.pos.width;
        rect.pos.height += this.pos.height;
        outline_node(entity, rect, margin, rules, outline.flags(), draw);

        if let Node::Container(c) = node {
            outline_nodes(outline, draw, c.flow, entity, rect);
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
    nodes: Query<'w, 's, (Entity, &'static Node, &'static PosRect)>,
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
    cam: CameraQuery,
    roots: Query<(Entity, &Root, &PosRect, Has<ScreenRoot>)>,
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
    let mut draw = InsetGizmo::new(draw, cam, line_width);
    for (entity, root, rect, is_screen) in &roots {
        if !root.debug {
            continue;
        }
        let margin = root.node.margin;
        let rules = root.node.rules.map_into();
        if is_screen {
            // inset so that the root container is fully visible.
            draw.set_scope(*rect, Size::ZERO);
        }
        outline_node(entity, *rect, margin, rules, outline.flags(), &mut draw);

        let flow = root.node.flow;
        outline_nodes(&outline, &mut draw, flow, entity, *rect);
    }
}
fn outline_node(
    entity: Entity,
    rect: PosRect,
    margin: Size<f32>,
    rules: Size<RuleArrow>,
    flags: EnumSet<Flag>,
    draw: &mut InsetGizmo,
) {
    let hue = hue_from_entity(entity);
    let main_color = Color::hsl(hue, NODE_SATURATION, NODE_LIGHTNESS);
    let margin_color = Color::hsl(hue, NODE_SATURATION, MARGIN_LIGHTNESS);

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
            LeafRule::Fixed(_, true) => RuleArrow::InwardBare,
            LeafRule::Fixed(_, false) => RuleArrow::None,
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

fn rect_border_axis(rect: PosRect, margin: Size<f32>) -> (f32, f32, f32, f32) {
    let pos = rect.pos() + Vec2::from(margin);
    let size = Vec2::from(rect.size()) - Vec2::from(margin) * 2.;
    let offset = pos + size;
    (pos.x, offset.x, pos.y, offset.y)
}

trait ApproxF32 {
    fn is(self, other: f32) -> bool;
}
impl ApproxF32 for f32 {
    fn is(self, other: f32) -> bool {
        let diff = (self - other).abs();
        diff < 0.001
    }
}

struct InsetGizmo<'w, 's> {
    draw: Gizmos<'s>,
    cam: CameraQuery<'w, 's>,
    known_y: DrawnLines,
    known_x: DrawnLines,
}
impl<'w, 's> InsetGizmo<'w, 's> {
    fn new(draw: Gizmos<'s>, cam: CameraQuery<'w, 's>, line_width: f32) -> Self {
        InsetGizmo {
            draw,
            cam,
            known_y: DrawnLines::new(line_width),
            known_x: DrawnLines::new(line_width),
        }
    }
    fn relative(&self, mut position: Vec2) -> Vec2 {
        let zero = GlobalTransform::IDENTITY;
        let Ok((cam, debug)) = self.cam.get_single() else { return Vec2::ZERO;};
        if debug.screen_space {
            if let Some(new_position) = cam.world_to_viewport(&zero, position.extend(0.)) {
                position = new_position;
            };
        }
        position.xy()
    }
    /// Draw rule at edge of container on given axis.
    fn rule(&mut self, center: Vec2, extents: Vec2, rule: RuleArrow, axis: Axis, color: Color) {
        use crate::Flow::{Horizontal as Width, Vertical as Height};

        let select = BVec2::new(axis == Width, axis == Height);
        let c = center;
        let e = Vec2::select(select, extents, Vec2::ZERO);
        let trim_e = (e * 0.25).min(Vec2::splat(100.));

        let Some((start1, end1, _)) = rule.arrange(c - e + trim_e, c - e) else { return; };
        let Some((start2, end2, _)) = rule.arrange(c + e - trim_e, c + e) else { return; };
        self.arrow(start1, end1, color, start1.distance(end1) * CHEVRON_RATIO);
        self.arrow(start2, end2, color, start2.distance(end2) * CHEVRON_RATIO);
    }
    fn line_2d(&mut self, start: Vec2, end: Vec2, color: Color) {
        let (start, end) = (self.relative(start), self.relative(end));
        self.draw.line_2d(start, end, color);
    }
    fn set_scope(&mut self, rect: PosRect, margin: Size<f32>) {
        let (left, right, top, bottom) = rect_border_axis(rect, margin);
        self.known_x.add(left, 1);
        self.known_x.add(right, -1);
        self.known_y.add(top, 1);
        self.known_y.add(bottom, -1);
    }
    fn clear_scope(&mut self, rect: PosRect, margin: Size<f32>) {
        let (left, right, top, bottom) = rect_border_axis(rect, margin);
        self.known_x.remove(left, 1);
        self.known_x.remove(right, -1);
        self.known_y.remove(top, 1);
        self.known_y.remove(bottom, -1);
    }
    fn rect_2d(&mut self, rect: PosRect, margin: Size<f32>, color: Color) {
        let (left, right, top, bottom) = rect_border_axis(rect, margin);
        if left.is(right) {
            self.line_2d(Vec2::new(left, top), Vec2::new(left, bottom), color);
        }
        if top.is(bottom) {
            self.line_2d(Vec2::new(left, top), Vec2::new(right, top), color);
        }
        let inset_x = |v, incr| self.known_x.inset(v, incr);
        let inset_y = |v, incr| self.known_y.inset(v, incr);
        let (left, right) = (inset_x(left, 1), inset_x(right, -1));
        let (top, bottom) = (inset_y(top, 1), inset_y(bottom, -1));
        self.draw.linestrip_2d(
            [
                Vec2::new(left, top),
                Vec2::new(left, bottom),
                Vec2::new(right, bottom),
                Vec2::new(right, top),
                Vec2::new(left, top),
            ]
            .map(|v| self.relative(v)),
            color,
        );
    }
    fn arrow(&mut self, start: Vec2, end: Vec2, color: Color, chevron_size: f32) {
        let Some(angle) = (end - start).try_normalize() else { return; };

        let top = Vec2::new(-1., 1.);
        let bottom = Vec2::new(-1., -1.);
        let len = chevron_size;
        self.line_2d(start, end, color);
        self.line_2d(end, end + angle.rotate(top) * len, color);
        self.line_2d(end, end + angle.rotate(bottom) * len, color);
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
            )
                .chain(),
        );
        app.insert_resource(Options {
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
