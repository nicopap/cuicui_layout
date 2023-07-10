//! Systems to update a [`cuicui_layout::Node`]'s size based on an image's
//! and text's size, rather that fixed at spawn time.
//!
//! This relies on the [`bevy::ui::Node`] component.
use bevy::{ecs::prelude::*, text::Text, ui, ui::UiImage};
use cuicui_layout::dsl::ContentSized;
use cuicui_layout::{LeafRule, Node, Size};

/// Update the [`cuicui_layout`] [`Node::Box`] [`LeafRule::Fixed`] values of
/// entities with a [`bevy::ui::Node`] component.
pub fn update(
    mut query: Query<
        (&mut Node, &ContentSized, &ui::Node),
        (Changed<ui::Node>, Or<(With<UiImage>, With<Text>)>),
    >,
) {
    for (mut node, sized, bevy_ui) in &mut query {
        // TODO(bug): If only a signle axis of an image is Fixed, then use the image's aspect
        // ratio to "fix" the other axis.
        if sized.managed_axis.width {
            if let Node::Box(Size { width: LeafRule::Fixed(value), .. }) = &mut *node {
                *value = bevy_ui.size().x;
            }
        }
        if sized.managed_axis.height {
            if let Node::Box(Size { height: LeafRule::Fixed(value), .. }) = &mut *node {
                *value = bevy_ui.size().y;
            }
        }
    }
}
