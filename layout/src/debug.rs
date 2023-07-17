//! Debug overlay for `cuicui_layout`.
//!
//! See [`Plugin`].
//!
#![doc = include_str!("../debug.md")]

#[cfg(feature = "bevy_ui")]
use bevy::prelude::UiCameraConfig;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    ecs::{prelude::*, system::SystemParam},
    math::Vec2Swizzles,
    prelude::{
        default, info, BVec2, Camera, Camera2d, Camera2dBundle, Children, Color, GizmoConfig,
        Gizmos, GlobalTransform, Input, KeyCode, OrthographicProjection, Plugin as BevyPlugin,
        Update, Vec2,
    },
    render::view::RenderLayers,
};

use crate::{direction::Axis, Flow, LayoutRootCamera, LeafRule, Node, PosRect, Root, Rule, Size};

pub use enumset::{EnumSet, EnumSetType};

/// The [`Camera::order`] index used by the layout debug camera.
pub const LAYOUT_DEBUG_CAMERA_ORDER: isize = 255;
/// The [`RenderLayers`] used by the debug gizmos and the debug camera.
pub const LAYOUT_DEBUG_LAYERS: RenderLayers = RenderLayers::none().with(16);

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

#[derive(Component)]
struct DebugOverlayCamera;

/// The debug overlay options.
#[derive(Resource, Clone, Default)]
pub struct Options {
    /// Which overlays are set.
    pub flags: EnumSet<Flag>,
    /// The inputs used by the debug overlay.
    pub input_map: InputMap,
    layout_gizmos_camera: Option<Entity>,
}

fn update_debug_camera(
    mut gizmo_config: ResMut<GizmoConfig>,
    mut options: ResMut<Options>,
    mut cmds: Commands,
    _layout_cams: Query<&Camera, With<LayoutRootCamera>>,
    mut debug_cams: Query<&mut Camera, (Without<LayoutRootCamera>, With<DebugOverlayCamera>)>,
) {
    if !options.is_changed() {
        return;
    }
    if options.flags.is_empty() {
        let Some(cam) = options.layout_gizmos_camera  else {return;};
        let Ok(mut cam) = debug_cams.get_mut(cam) else {return;};
        cam.is_active = false;
        gizmo_config.render_layers = RenderLayers::all();
    } else {
        let spawn_cam = || {
            cmds.spawn((
                #[cfg(feature = "bevy_ui")]
                UiCameraConfig { show_ui: false },
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
                DebugOverlayCamera,
                LAYOUT_DEBUG_LAYERS,
            ))
            .id()
        };
        let cam = *options.layout_gizmos_camera.get_or_insert_with(spawn_cam);
        let Ok(mut cam) = debug_cams.get_mut(cam) else {return;};
        gizmo_config.enabled = true;
        gizmo_config.render_layers = LAYOUT_DEBUG_LAYERS;
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
// TODO(clean)TODO(bug): This doesn't work. only kinda.
fn inset(
    line_width: f32,
    parent: PosRect,
    parent_margin: Size<f32>,
    mut child: PosRect,
    mut parent_inset: Size<f32>,
) -> (PosRect, Size<f32>) {
    let line_width = line_width + parent_inset.width;
    let line_height = line_width + parent_inset.height;
    // horizontal
    let start = parent_margin.width;
    let end = parent.size.width - parent_margin.width;
    if child.pos.width.is(start) {
        child.pos.width += line_width;
        child.size.width -= line_width;
        parent_inset.width = line_width;
    }
    if (child.pos.width + child.size.width).is(end) {
        child.size.width -= line_width;
    }
    // vertical
    let start = parent_margin.height;
    let end = parent.size.height - parent_margin.height;
    if child.pos.height.is(start) {
        child.pos.height += line_height;
        child.size.height -= line_height;
        parent_inset.height = line_height;
    }
    if (child.pos.height + child.size.height).is(end) {
        child.size.height -= line_height;
    }
    // returns
    let pos = Size {
        width: child.pos.width + parent.pos.width,
        height: child.pos.height + parent.pos.height,
    };
    (PosRect { pos, ..child }, parent_inset)
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
    params: &OutlineParam,
    draw: &mut ViewportGizmo,
    flow: Flow,
    this_entity: Entity,
    this_margin: Size<f32>,
    this_inset: Size<f32>,
    this: PosRect,
) {
    let Ok(to_iter) = params.children.get(this_entity) else { return; };
    for (entity, node, child) in params.nodes.iter_many(to_iter) {
        let rules = node_rules(flow, node);
        let margin = node_margin(node);
        let (pos, inset) = inset(params.line_width(), this, this_margin, *child, this_inset);
        let flags = params.options.flags;
        outline_node(entity, pos, margin, rules, flags, draw);

        if let Node::Container(c) = node {
            let mut rect = *child;
            rect.pos.width += this.pos.width;
            rect.pos.height += this.pos.height;
            outline_nodes(params, draw, c.flow, entity, margin, inset, rect);
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
    fn line_width(&self) -> f32 {
        self.gizmo_config.line_width / 2.
    }
}
fn outline_roots(
    params: OutlineParam,
    mut draw: ViewportGizmo,
    roots: Query<(Entity, &Root, &PosRect)>,
) {
    for (entity, root, pos) in &roots {
        if !root.debug {
            continue;
        }

        let (inset_pos, inset) = inset(params.line_width(), *pos, Size::ZERO, *pos, Size::ZERO);
        let rules = root.node.rules.map_into();
        let margin = root.node.margin;
        let flags = params.options.flags;
        outline_node(entity, inset_pos, margin, rules, flags, &mut draw);

        let flow = root.node.flow;
        outline_nodes(&params, &mut draw, flow, entity, margin, inset, *pos);
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
struct InnerSize(PosRect);

// returns the inner-size of container
fn outline_node(
    entity: Entity,
    pos: PosRect,
    margins: Size<f32>,
    rules: Size<RuleArrow>,
    flags: EnumSet<Flag>,
    draw: &mut ViewportGizmo,
) {
    let hue = hue_from_entity(entity);
    let main_color = Color::hsl(hue, 0.8, 0.7);
    let margin_color = Color::hsl(hue, 0.8, 0.85);

    let extents = Vec2::from(pos.size()) / 2.;
    let center = pos.pos() + extents;

    if flags.contains(Flag::Outlines) {
        // first draw margins, as we will draw the actual outline on top
        let m = Vec2::from(margins);
        if m.x != 0. || m.y != 0. {
            draw.rect_2d(center, (extents - m) * 2., margin_color);
        }
        draw.rect_2d(center, extents * 2., main_color);
    }
    if flags.contains(Flag::InfoText) {}
    if flags.contains(Flag::Rules) {
        // TODO: avoid drawing on top of text
        // TODO: ratio on top of arrow
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

trait ApproxF32 {
    fn is(self, other: f32) -> bool;
}
impl ApproxF32 for f32 {
    fn is(self, other: f32) -> bool {
        let diff = (self - other).abs();
        diff < 0.001
    }
}

#[derive(SystemParam)]
struct ViewportGizmo<'w, 's> {
    draw: Gizmos<'s>,
    cam: Query<'w, 's, &'static Camera, With<DebugOverlayCamera>>,
}
impl ViewportGizmo<'_, '_> {
    fn relative(&self, position: Vec2) -> Vec2 {
        let zero = GlobalTransform::IDENTITY;
        let Ok(cam) = self.cam.get_single() else { return Vec2::ZERO;};
        let Some(p) = cam.world_to_viewport(&zero, position.extend(0.)) else { return Vec2::ZERO };
        p.xy()
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
        self.arrow(start1, end1, color, start1.distance(end1) / 4.);
        self.arrow(start2, end2, color, start2.distance(end2) / 4.);
    }
    fn line_2d(&mut self, start: Vec2, end: Vec2, color: Color) {
        let (start, end) = (self.relative(start), self.relative(end));
        self.draw.line_2d(start, end, color);
    }
    fn rect_2d(&mut self, pos: Vec2, size: Vec2, color: Color) {
        self.draw.rect_2d(self.relative(pos), 0., size, color);
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
/// disabling the plugin will give you back gizmo control.
pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<InputMap>()
            .init_resource::<Options>()
            .add_systems(
                Update,
                (
                    (update_debug_camera, cycle_flags),
                    outline_roots.after(crate::ComputeLayoutSet),
                ),
            );
    }
    fn finish(&self, _app: &mut bevy::prelude::App) {
        info!(
            "The cuicui_layout debug overlay is activated!\n\
            ----------------------------------------------\n\
            \n\
            This will show the outline of layout nodes.\n\
            Press `Space` to switch between debug mods."
        );
    }
}
