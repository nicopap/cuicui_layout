//! Systems to update a [`cuicui_layout::Node`]'s size based on an image's
//! and text's size, rather that fixed at spawn time.
use bevy::prelude::CalculatedSize;
use bevy::{
    ecs::prelude::*,
    reflect::{FromReflect, Reflect},
    text::Text,
    ui::UiImage,
};
use cuicui_layout::{LeafRule, Node, Size};

#[derive(Component, Clone, Copy, Debug, Default, Reflect, FromReflect)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct ContentSize;

fn width_fixed(node: &Node) -> bool {
    matches!(node, Node::Box(Size { width: LeafRule::Fixed(_), .. }))
}
fn height_fixed(node: &Node) -> bool {
    matches!(node, Node::Box(Size { height: LeafRule::Fixed(_), .. }))
}
pub fn add_content_size(
    mut cmds: Commands,
    maybe_add_size: Query<
        (Entity, &Node),
        (
            Or<(With<UiImage>, With<Text>)>,
            Without<ContentSize>,
            Changed<Node>,
        ),
    >,
) {
    for (entity, node) in &maybe_add_size {
        let fixed_width = width_fixed(node);
        let fixed_height = height_fixed(node);
        if fixed_height || fixed_width {
            cmds.entity(entity).insert(ContentSize);
        }
    }
}

pub fn clear_content_size(
    mut cmds: Commands,
    maybe_remove_size: Query<
        (Entity, &Node),
        (
            Or<(With<UiImage>, With<Text>)>,
            With<ContentSize>,
            Changed<Node>,
        ),
    >,
) {
    for (entity, node) in &maybe_remove_size {
        let fixed_width = width_fixed(node);
        let fixed_height = height_fixed(node);
        if !fixed_height && !fixed_width {
            cmds.entity(entity).remove::<ContentSize>();
        }
    }
}

pub fn update_node(
    mut query: Query<
        (&mut Node, &CalculatedSize),
        (
            Changed<CalculatedSize>,
            Or<(With<UiImage>, With<Text>)>,
            With<ContentSize>,
        ),
    >,
) {
    for (mut node, size) in &mut query {
        // TODO(bug): If only a signle axis of an image is Fixed, then use the image's aspect
        // ratio to "fix" the other axis.
        if let Node::Box(Size { width: LeafRule::Fixed(value), .. }) = &mut *node {
            *value = size.size.x;
        }
        if let Node::Box(Size { height: LeafRule::Fixed(value), .. }) = &mut *node {
            *value = size.size.y;
        }
    }
}
