//! Demonstrate how to use cuicui layout.

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology, view::RenderLayers},
    sprite::MaterialMesh2dBundle,
};
use cuicui_layout::{
    dsl::{IntoUiBundle, UiBundle},
    layout, Node, PosRect, Root, Rule, Size,
};
use cuicui_layout_bevy_sprite as render;

const UI_LAYER: RenderLayers = RenderLayers::none().with(20);

#[allow(clippy::cast_precision_loss)]
fn color_from_entity(entity: Entity) -> Color {
    use std::hash::{Hash, Hasher};
    const U64_TO_DEGREES: f32 = 360.0 / u64::MAX as f32;

    let mut hasher = bevy::utils::AHasher::default();
    entity.hash(&mut hasher);

    let hue = hasher.finish() as f32 * U64_TO_DEGREES;
    Color::hsla(hue, 0.8, 0.5, 0.5)
}

fn main() {
    use bevy_inspector_egui::quick::WorldInspectorPlugin;
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(setup_debug.in_base_set(StartupSet::PostStartup))
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(cuicui_layout::Plug::new())
        .add_system(cuicui_layout::update_transforms)
        .add_system(render::update_ui_camera_root)
        .add_system(stretch_boxes)
        .run();
}

type LayoutType = cuicui_layout::dsl::LayoutType;

#[allow(clippy::type_complexity)]
fn setup_debug(
    mut cmds: Commands,
    nodes: Query<Entity, Or<(With<Node>, With<Root>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(top_left_quad());
    for node in &nodes {
        cmds.entity(node)
            .insert((
                SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.01)),
                UI_LAYER,
            ))
            .with_children(|cmds| {
                cmds.spawn((
                    MaterialMesh2dBundle {
                        mesh: mesh.clone().into(),
                        material: mats.add(color_from_entity(node).into()),
                        ..default()
                    },
                    DebugChild,
                    Name::new("DebugMesh"),
                    UI_LAYER,
                ));
            });
    }
}

#[derive(Component)]
struct DebugChild;

#[allow(clippy::needless_pass_by_value)]
fn stretch_boxes(
    query: Query<(&Children, &PosRect), Changed<PosRect>>,
    mut trans: Query<&mut Transform, With<DebugChild>>,
) {
    for (children, pos) in &query {
        let mut iter = trans.iter_many_mut(children);
        while let Some(mut trans) = iter.fetch_next() {
            trans.scale.x = pos.size().width.max(1.0);
            trans.scale.y = pos.size().height.max(1.0);
        }
    }
}

#[derive(Bundle)]
struct ElementBundle {
    node: Node,
    pos: PosRect,
    layer: RenderLayers,
}
impl Default for ElementBundle {
    fn default() -> Self {
        ElementBundle {
            node: Node::default(),
            pos: default(),
            layer: UI_LAYER,
        }
    }
}
#[derive(Component, Clone)]
struct Fixed(i32, i32);

#[derive(Component, Clone)]
struct Space(i8);

impl IntoUiBundle<Fixed> for Fixed {
    type Target = ElementBundle;
    fn into_ui_bundle(self) -> Self::Target {
        ElementBundle {
            node: Node::fixed(Size::new(self.0 as f32, self.1 as f32)),
            ..default()
        }
    }
}
impl IntoUiBundle<Space> for Space {
    type Target = ElementBundle;
    fn into_ui_bundle(self) -> Self::Target {
        ElementBundle {
            node: Node::spacer_percent(self.0 as f32).unwrap(),
            ..default()
        }
    }
}
impl UiBundle for ElementBundle {
    fn set_fixed_width(&mut self) {}
    fn set_fixed_height(&mut self) {}
}

fn setup(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle {
        projection: OrthographicProjection { scale: 0.25, ..default() },
        transform: Transform::from_xyz(108.7, 142.0, 999.9),
        ..default()
    });
    cmds.spawn(render::UiCameraBundle::for_layer(1, 20));
    layout! {
        &mut cmds,
        column("root", screen_root, fill_main_axis, main_margin 50., cross_margin 100.) {
            spawn_ui(Space(10), "spacer1");
            row("horiz_cont1", fill_main_axis, height * 1.0) {
                spawn_ui(Fixed(10, 10), "h1_1_fix");
                spawn_ui(Fixed(30, 10), "h1_2_fix");
                spawn_ui(Fixed(50, 20), "h1_3_fix");
                spawn_ui(Space(10), "h1_4_spacer");
                spawn_ui(Fixed(51, 32), "h1_5_fix");
            }
            spawn_ui(Fixed(10, 20), "fix1");
            spawn_ui(Fixed(40, 30), "fix2");
            row("horiz_cont2", height * 1.0) {
                spawn_ui(Fixed(10, 14), "h2_1_fix");
                spawn_ui(Fixed(12, 12), "h2_2_fix");
                spawn_ui(Fixed(14, 10), "h2_3_fix");
            }
            row("horiz_cont3", fill_main_axis, height * 1.) {
                spawn_ui(Space(4), "spacer5");
                // row("horiz_cont4", fill_main) {
                //     spawn_ui(Fixed(10, 14), "h4_1" );
                //     spawn_ui(Fixed(12, 12), "h4_2" );
                //     spawn_ui(Fixed(14, 10), "h4_3" );
                // }
                column("vert_cont1", height * 1., width * 1.) {
                    spawn_ui(Fixed(10, 21), "v1_1_fix");
                    spawn_ui(Fixed(12, 12), "v1_2_fix");
                    spawn_ui(Fixed(14, 20), "v1_3_fix");
                    spawn_ui(Fixed(16, 21), "v1_4_fix");
                    spawn_ui(Fixed(18, 12), "v1_5_fix");
                    spawn_ui(Fixed(20, 20), "v1_6_fix");
                }
                row("horiz_inner", height * 1., width * 1.) {
                    spawn_ui(Fixed(10, 21), "v2_1_fix");
                    spawn_ui(Fixed(12, 12), "v2_2_fix");
                    spawn_ui(Fixed(14, 20), "v2_3_fix");
                    spawn_ui(Fixed(16, 21), "v2_4_fix");
                    spawn_ui(Fixed(18, 12), "v2_5_fix");
                    spawn_ui(Fixed(20, 20), "v2_6_fix");
                }
                column("vert_cont3", height * 1., width * 1.) {
                    spawn_ui(Fixed(10, 21), "v3_1_fix");
                    spawn_ui(Fixed(12, 12), "v3_2_fix");
                    spawn_ui(Fixed(14, 20), "v3_3_fix");
                    spawn_ui(Fixed(16, 21), "v3_4_fix");
                    spawn_ui(Fixed(18, 12), "v3_5_fix");
                    spawn_ui(Fixed(20, 20), "v3_6_fix");
                }
                spawn_ui(Space(4), "spacer4");
            }
            spawn_ui(Space(10), "spacer3");
        }
    }
}
fn top_left_quad() -> Mesh {
    let vertices = [
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
        ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
        ([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
    ];

    let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
