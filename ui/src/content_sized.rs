//! Systems to update a [`cuicui_layout::Node`]'s size based on an image's
//! and text's size, rather that fixed at spawn time.
//!
//! This relies on the [`ContentSized`] component.
use bevy::prelude::CalculatedSize;
use bevy::{ecs::prelude::*, text::Text, ui::UiImage};
use cuicui_layout::dsl::ContentSized;
use cuicui_layout::{LeafRule, Node, Size};

/// Update the [`cuicui_layout`] [`Node::Box`] [`LeafRule::Fixed`] values of
/// entities with a [`CalculatedSize`] component.
pub fn update(
    mut query: Query<
        (&mut Node, &ContentSized, &CalculatedSize),
        (Changed<CalculatedSize>, Or<(With<UiImage>, With<Text>)>),
    >,
) {
    for (mut node, sized, size) in &mut query {
        // TODO(bug): If only a signle axis of an image is Fixed, then use the image's aspect
        // ratio to "fix" the other axis.
        if sized.0.width {
            if let Node::Box(Size { width: LeafRule::Fixed(value), .. }) = &mut *node {
                *value = size.size.x;
            }
        }
        if sized.0.height {
            if let Node::Box(Size { height: LeafRule::Fixed(value), .. }) = &mut *node {
                *value = size.size.y;
            }
        }
    }
}
