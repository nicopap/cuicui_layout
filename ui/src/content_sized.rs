//! Systems to update a [`cuicui_layout::Node`]'s size based on an image's
//! and text's size, rather that fixed at spawn time.
//!
//! This relies on the [`ContentSized`] component.
use bevy::prelude::CalculatedSize;
use bevy::{
    ecs::prelude::*,
    reflect::{FromReflect, Reflect},
    text::Text,
    ui::UiImage,
};
use cuicui_layout::{LeafRule, Node, Size};

/// Dynamically update the [`Node::Box`] rules fixed values of UI entities with
/// either the [`UiImage`] or [`Text`] component.
#[derive(Component, Clone, Copy, Debug, Default, Reflect, FromReflect)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct ContentSized;

const fn width_fixed(node: &Node) -> bool {
    matches!(node, Node::Box(Size { width: LeafRule::Fixed(_), .. }))
}
const fn height_fixed(node: &Node) -> bool {
    matches!(node, Node::Box(Size { height: LeafRule::Fixed(_), .. }))
}

/// Add [`ContentSized`] to [`Node`] entities when one of their axis
/// is [`LeafRule::Fixed`], and they have a [`UiImage`] or [`Text`] component.
#[allow(clippy::needless_pass_by_value)]
pub fn add(
    mut cmds: Commands,
    maybe_add_size: Query<
        (Entity, &Node),
        (
            Or<(With<UiImage>, With<Text>)>,
            Without<ContentSized>,
            Changed<Node>,
        ),
    >,
) {
    maybe_add_size.for_each(|(entity, node)| {
        let fixed_width = width_fixed(node);
        let fixed_height = height_fixed(node);
        if fixed_height || fixed_width {
            cmds.entity(entity).insert(ContentSized);
        }
    });
}

/// Remove [`ContentSized`] from [`Node`] entities when none of their axis
/// is [`LeafRule::Fixed`].
#[allow(clippy::needless_pass_by_value)]
pub fn clear(
    mut cmds: Commands,
    maybe_remove_size: Query<
        (Entity, &Node),
        (
            Or<(With<UiImage>, With<Text>)>,
            With<ContentSized>,
            Changed<Node>,
        ),
    >,
) {
    maybe_remove_size.for_each(|(entity, node)| {
        let fixed_width = width_fixed(node);
        let fixed_height = height_fixed(node);
        if !fixed_height && !fixed_width {
            cmds.entity(entity).remove::<ContentSized>();
        }
    });
}

/// Update the [`cuicui_layout`] [`Node::Box`] [`LeafRule::Fixed`] values of
/// entities with a [`CalculatedSize`] component.
pub fn update(
    mut query: Query<
        (&mut Node, &CalculatedSize),
        (
            Changed<CalculatedSize>,
            Or<(With<UiImage>, With<Text>)>,
            With<ContentSized>,
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
