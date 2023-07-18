//! Makes a somewhat complex layout with nested element and rules going forward
//! or backward.
//!
//! The goal is to test `cuicui_layout` in non-trival situations.
#![allow(clippy::cast_precision_loss, clippy::wildcard_imports)]

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology, view::RenderLayers},
    sprite::MaterialMesh2dBundle,
};
use cuicui_dsl::dsl;
use cuicui_layout::{dsl::IntoUiBundle, dsl_functions::*, Node, PosRect, Root, Size};
use cuicui_layout_bevy_sprite as render;
use cuicui_layout_bevy_sprite::SpriteDsl as Dsl;

const UI_LAYER: RenderLayers = RenderLayers::none().with(20);

fn van_der_corput(bits: u32) -> f32 {
    let leading_zeros = if bits == 0 { 0 } else { bits.leading_zeros() };
    let nominator = bits.reverse_bits() >> leading_zeros;
    let denominator = bits.next_power_of_two();

    nominator as f32 / denominator as f32
}
fn color_from_entity(entity: Entity) -> Color {
    Color::hsla(van_der_corput(entity.index()) * 360., 1., 0.5, 0.6)
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, cuicui_layout::Plugin))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(PostStartup, setup_debug)
        // .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
        .add_systems(
            Update,
            (
                update_transforms,
                render::update_ui_camera_root,
                stretch_boxes,
            ),
        )
        .run();
}

/// Update transform of things that have a `PosRect` component.
fn update_transforms(mut positioned: Query<(&PosRect, &mut Transform), Changed<PosRect>>) {
    for (pos, mut transform) in &mut positioned {
        let z = transform.translation.z;
        transform.translation = pos.pos().extend(z);
    }
}

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

impl IntoUiBundle<Fixed> for Fixed {
    type Target = ElementBundle;
    fn into_ui_bundle(self) -> Self::Target {
        #[allow(clippy::cast_precision_loss)]
        ElementBundle {
            node: Node::fixed(Size::new(self.0 as f32, self.1 as f32)),
            ..default()
        }
    }
}

fn setup(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle {
        projection: OrthographicProjection { scale: 0.25, ..default() },
        transform: Transform::from_xyz(108.7, 142.0, 999.9),
        ..default()
    });
    cmds.spawn(render::UiCameraBundle::for_layer(1, 20));
    dsl! {
        &mut cmds,
        column("root", screen_root, margins(50., 100.)) {
            row("horiz_cont1", align_start, width pct(85), main_margin 30.) {
                spawn_ui(Fixed(10, 10), "h1_1_fix");
                spawn_ui(Fixed(30, 10), "h1_2_fix");
                spawn_ui(Fixed(50, 20), "h1_3_fix");
                empty_pct(10, "h1_4_spacer");
                spawn_ui(Fixed(51, 32), "h1_5_fix");
            }
            row("deep1", rules(pct(80), child(1.5))) {
                row("deep2", rules(child(1.5), child(3.))) {
                    row("deep3", rules(child(4.), child(1.5))) {
                        spawn_ui(Fixed(10, 10), "deep4");
                    }
                }
            }
            row("single_child", rules(child(2.), child(2.))) {
                spawn_ui(Fixed(40, 40), "fix2");
            }
            spawn("horiz_cont2", layout ">dSaC", main_margin 30.) {
                spawn_ui(Fixed(10, 14), "h2_1_fix");
                spawn_ui(Fixed(12, 12), "h2_2_fix");
                spawn_ui(Fixed(14, 10), "h2_3_fix");
            }
            row("horiz_cont3", width pct(100), main_margin 30.) {
                // row("horiz_cont4", fill_main) {
                //     spawn_ui(Fixed(10, 14), "h4_1" );
                //     spawn_ui(Fixed(12, 12), "h4_2" );
                //     spawn_ui(Fixed(14, 10), "h4_3" );
                // }
                column("vert_cont1", align_start, width pct(25), margins(30., 5.0)) {
                    spawn_ui(Fixed(10, 21), "v1_1_fix");
                    spawn_ui(Fixed(12, 12), "v1_2_fix");
                    spawn_ui(Fixed(14, 20), "v1_3_fix");
                    spawn_ui(Fixed(16, 21), "v1_4_fix");
                    spawn_ui(Fixed(18, 12), "v1_5_fix");
                    spawn_ui(Fixed(20, 20), "v1_6_fix");
                }
                row("horiz_inner", distrib_end, height child(4.), margins(30., 5.0)) {
                    spawn_ui(Fixed(10, 21), "v2_1_fix");
                    spawn_ui(Fixed(12, 12), "v2_2_fix");
                    spawn_ui(Fixed(14, 20), "v2_3_fix");
                    spawn_ui(Fixed(16, 21), "v2_4_fix");
                    spawn_ui(Fixed(18, 12), "v2_5_fix");
                    spawn_ui(Fixed(20, 20), "v2_6_fix");
                }
                spawn("vert_cont3", layout "vdSaE", margins(30., 5.)) {
                    spawn_ui(Fixed(10, 21), "v3_1_fix");
                    spawn_ui(Fixed(12, 12), "v3_2_fix");
                    spawn_ui(Fixed(14, 20), "v3_3_fix");
                    spawn_ui(Fixed(16, 21), "v3_4_fix");
                    spawn_ui(Fixed(18, 12), "v3_5_fix");
                    spawn_ui(Fixed(20, 20), "v3_6_fix");
                }
                empty_pct(4, "spacer4");
            }
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
