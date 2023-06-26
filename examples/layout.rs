//! Demonstrate how to use cuicui layout.

use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology, view::RenderLayers},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use cuicui_layout as layout;
use layout::Container;

const UI_LAYER: RenderLayers = RenderLayers::none().with(20);

macro_rules! root {
    (($name:literal, $dir:expr, $suse:ident, $width:expr, $height:expr), $($branch:expr),* $(,)?) => {
        UiRoot {
            name: $name,
            children: vec![$( $branch, )*],
            container: layout::Container::$suse ( $dir ),
            bounds: layout::Size { width: $width as f32, height: $height as f32 },
        }
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

struct Rng {
    seed: u64,
}
impl Rng {
    const P0: u64 = 0xa076_1d64_78bd_642f;
    const P1: u64 = 0xe703_7ed1_a0b4_28db;
    fn color(&mut self) -> Color {
        const fn random(a: u64, b: u64) -> u64 {
            let (hh, hl) = ((a >> 32) * (b >> 32), (a >> 32) * (b & 0xFFFF_FFFF));
            let lh = (a & 0xFFFF_FFFF) * (b >> 32);
            let ll = (a & 0xFFFF_FFFF) * (b & 0xFFFF_FFFF);
            (hl.rotate_left(32) ^ hh) ^ (lh.rotate_left(32) ^ ll)
        }
        self.seed = self.seed.wrapping_add(Self::P0);
        let rand_u64 = random(self.seed, self.seed ^ Self::P1);
        Color::rgba_u8(
            (rand_u64 & 0xff) as u8,
            ((rand_u64 & 0xff_00) >> 8) as u8,
            ((rand_u64 & 0xff_00_00) >> 16) as u8,
            200,
        )
    }
}

fn main() {
    use bevy_inspector_egui::quick::WorldInspectorPlugin;
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(layout::Plug)
        .add_system(layout::update_transforms)
        .add_system(layout::render::update_ui_camera_root)
        .add_system(stretch_boxes)
        .run();
}
struct ExtraSpawnArgs<'a, 'b, 'c> {
    rng: &'a mut Rng,
    assets: &'b mut Assets<ColorMaterial>,
    mesh: &'c Mesh2dHandle,
}

impl<'a, 'b, 'c> ExtraSpawnArgs<'a, 'b, 'c> {
    fn debug_child(&mut self) -> impl Bundle {
        (
            MaterialMesh2dBundle {
                mesh: self.mesh.clone(),
                material: self.assets.add(ColorMaterial::from(self.rng.color())),
                ..default()
            },
            DebugChild,
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
struct SpawnArgs<'w, 's, 'a, 'b, 'c, 'd> {
    cmds: EntityCommands<'w, 's, 'a>,
    inner: ExtraSpawnArgs<'b, 'c, 'd>,
}

struct UiRoot {
    name: &'static str,
    children: Vec<UiTree>,
    container: layout::Container,
    bounds: layout::Size<f32>,
}
impl UiRoot {
    fn spawn(self, cmds: &mut Commands, mut inner: ExtraSpawnArgs) {
        let Self { children, container, bounds, name } = self;
        let Container { direction, align, distrib, .. } = container;

        cmds.spawn(layout::render::UiCameraBundle::for_layer(1, 20));
        cmds.spawn((
            layout::render::RootBundle {
                node: layout::Root::new(bounds, direction, align, distrib),
                layer: UI_LAYER,
            },
            inner.debug_node(),
            Name::new(name),
        ))
        .with_children(|cmds| {
            cmds.spawn((inner.debug_child(), Name::new("DebugMesh")));
            for child in children.into_iter() {
                let inner = ExtraSpawnArgs {
                    rng: inner.rng,
                    assets: inner.assets,
                    mesh: inner.mesh,
                };
                let cmds = cmds.spawn_empty();
                child.spawn(SpawnArgs { cmds, inner });
            }
        });
    }
}
struct UiTree {
    name: &'static str,
    children: Vec<UiTree>,
    node: layout::Node,
}
impl UiTree {
    fn spawn(self, SpawnArgs { mut cmds, mut inner }: SpawnArgs) {
        let Self { children, node, name } = self;
        cmds.insert((node, inner.debug_node(), Name::new(name)))
            .with_children(|cmds| {
                cmds.spawn((inner.debug_child(), Name::new("DebugMesh")));
                for child in children.into_iter() {
                    let inner = ExtraSpawnArgs {
                        rng: inner.rng,
                        assets: inner.assets,
                        mesh: inner.mesh,
                    };
                    let cmds = cmds.spawn_empty();
                    child.spawn(SpawnArgs { cmds, inner });
                }
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
    use layout::Direction::*;
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
        ExtraSpawnArgs {
            rng: &mut Rng { seed: Rng::P0 },
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
