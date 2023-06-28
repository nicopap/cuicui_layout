//! Demonstrate how to use cuicui layout.

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology, view::RenderLayers},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use cuicui_layout as layout;
use cuicui_layout_bevy_sprite as render;
use layout::Container;

const UI_LAYER: RenderLayers = RenderLayers::none().with(20);

macro_rules! root {
    (($name:literal, $dir:expr, $suse:ident, $width:expr, $height:expr), $($branch:expr),* $(,)?) => {
        UiRoot(UiTree {
            name: $name,
            children: vec![$( $branch, )*],
            node: layout::Node::Container(Container {
                size: layout::Size::new($width as f32, $height as f32).map(layout::Rule::Fixed),
                ..Container::$suse ( $dir )
            })
        })
    };
}
macro_rules! spacer {
    ($name:literal, $parent_ratio:literal % $(,$branch:expr)* $(,)? ) => {
        UiTree {
            name: $name,
            children: vec![$( $branch, )*],
            node: layout::Node::spacer_percent($parent_ratio as f32).unwrap(),
        }
    };
}
macro_rules! fix {
    ($name:literal, $width:expr, $height:expr $(,$branch:expr)* $(,)? ) => {
        UiTree {
            name: $name,
            children: vec![$( $branch, )*],
            node: layout::Node::fixed(layout::Size { width: $width as f32, height: $height as f32 })
        }
    };
}
macro_rules! cont {
    (($name:literal, $dir:expr, $suse:ident), $($branch:expr),* $(,)?) => {
        UiTree {
            name: $name,
            children: vec![$( $branch, )*],
            node: layout::Node::Container(layout::Container::$suse ( $dir ))
        }
    };
}

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
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(layout::Plug)
        .add_system(layout::update_transforms)
        .add_system(render::update_ui_camera_root)
        .add_system(stretch_boxes)
        .run();
}
struct ExtraSpawnArgs<'a, 'm> {
    entity: Entity,
    assets: &'a mut Assets<ColorMaterial>,
    mesh: &'m Mesh2dHandle,
}

impl<'a, 'm> ExtraSpawnArgs<'a, 'm> {
    fn debug_mesh(&mut self) -> impl Bundle {
        (
            MaterialMesh2dBundle {
                mesh: self.mesh.clone(),
                material: self.assets.add(color_from_entity(self.entity).into()),
                ..default()
            },
            DebugChild,
            Name::new("DebugMesh"),
            UI_LAYER,
        )
    }
    fn debug_node(&mut self) -> impl Bundle {
        (
            layout::PosRect::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.01)),
        )
    }
}
struct UiRoot(UiTree);
struct UiTree {
    name: &'static str,
    children: Vec<UiTree>,
    node: layout::Node,
}
impl UiRoot {
    fn spawn(self, cmds: &mut Commands, inner: &mut ExtraSpawnArgs) {
        let Self(UiTree { children, name, node }) = self;
        let layout::Node::Container(Container { flow, align, distrib, size }) = node else {
            return;
        };
        let bounds = size.map(|v| if let layout::Rule::Fixed(v) = v { v } else { 0.0 });
        cmds.spawn(render::UiCameraBundle::for_layer(1, 20));

        let bundle = (
            render::RootBundle {
                node: layout::Root::new(bounds, flow, align, distrib),
                layer: UI_LAYER,
            },
            inner.debug_node(),
            Name::new(name),
        );
        cmds.spawn(bundle).with_children(|cmds| {
            inner.entity = cmds.parent_entity();
            cmds.spawn(inner.debug_mesh());
            children.into_iter().for_each(|child| {
                child.spawn(cmds, inner);
            });
        });
    }
}
impl UiTree {
    fn spawn(self, cmds: &mut ChildBuilder, inner: &mut ExtraSpawnArgs) {
        let Self { children, node, name } = self;
        let bundle = (node, inner.debug_node(), Name::new(name));
        cmds.spawn(bundle).with_children(|cmds| {
            inner.entity = cmds.parent_entity();
            cmds.spawn(inner.debug_mesh());
            children.into_iter().for_each(|child| {
                child.spawn(cmds, inner);
            });
        });
    }
}

#[derive(Component)]
struct DebugChild;
fn stretch_boxes(
    query: Query<(&Children, &layout::PosRect), Changed<layout::PosRect>>,
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
fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut assets: ResMut<Assets<ColorMaterial>>,
) {
    use layout::Flow::*;

    let tree = root! { ("root", Vertical, stretch, 300, 270),
        spacer!("spacer1", 10%),
        cont! { ("horiz_cont1", Horizontal, stretch),
            fix!("h1_1_fix", 10, 10), fix!("h1_2_fix", 30, 10), fix!("h1_3_fix", 50, 20),
            spacer!("h1_4_spacer", 10%), fix!("h1_5_fix", 51, 32),
        },
        fix!("fix1", 10, 20),
        fix!("fix2", 40, 30),
        cont! { ("horiz_cont2", Horizontal, compact),
            fix!("h2_1_fix", 10, 14), fix!("h2_2_fix", 12, 12), fix!("h2_3_fix", 14, 10),
        },
        cont! { ("horiz_cont3", Horizontal, stretch),
            spacer!("spacer5", 4%),
            // cont! { ("horiz_cont4", Horizontal, stretch),
            //     fix!("h4_1", 10, 14), fix!("h4_2", 12, 12), fix!("h4_3", 14, 10),
            // }
            cont! { ("vert_cont1", Vertical, compact),
                fix!("v1_1_fix",10, 21),
                fix!("v1_2_fix",12, 12),
                fix!("v1_3_fix",14, 20),
                fix!("v1_4_fix",16, 21),
                fix!("v1_5_fix",18, 12),
                fix!("v1_6_fix",20, 20),
            },
            cont! { ("horiz_inner", Horizontal, compact),
                fix!("v2_1_fix",10, 21),
                fix!("v2_2_fix",12, 12),
                fix!("v2_3_fix",14, 20),
                fix!("v2_4_fix",16, 21),
                fix!("v2_5_fix",18, 12),
                fix!("v2_6_fix",20, 20),
            },
            cont! { ("vert_cont3", Vertical, compact),
                fix!("v3_1_fix",10, 21),
                fix!("v3_2_fix",12, 12),
                fix!("v3_3_fix",14, 20),
                fix!("v3_4_fix",16, 21),
                fix!("v3_5_fix",18, 12),
                fix!("v3_6_fix",20, 20),
            },
            spacer!("spacer4", 4%),
        },
        spacer!("spacer3", 10%),
    };
    tree.spawn(
        &mut cmds,
        &mut ExtraSpawnArgs {
            entity: Entity::PLACEHOLDER,
            assets: &mut assets,
            mesh: &meshes.add(top_left_quad()).into(),
        },
    );
    cmds.spawn(Camera2dBundle {
        projection: OrthographicProjection { scale: 0.25, ..default() },
        transform: Transform::from_xyz(108.7, 142.0, 999.9),
        ..default()
    });
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
