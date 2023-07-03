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
#[reflect(Component)]
pub struct ContentSize;

// TODO(bug): update when text updates
// TODO(bug): keep image size
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
        // width
        if let Node::Box(Size { width: LeafRule::Fixed(value), .. }) = &mut *node {
            *value = size.size.x;
        }
        // height
        if let Node::Box(Size { height: LeafRule::Fixed(value), .. }) = &mut *node {
            *value = size.size.y;
        }
    }
}
