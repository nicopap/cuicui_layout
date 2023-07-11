//! Systems to update a [`cuicui_layout::Node`]'s size based on an image's
//! and text's size, rather that fixed at spawn time.
//!
//! This relies on the [`bevy::ui::Node`] component.
use bevy::{
    ecs::{prelude::*, system::SystemParam},
    prelude::{trace, Assets, Vec2},
    text::{Font, Text, TextPipeline},
    ui::{self, widget::UiImageSize},
};
use cuicui_layout::dsl::ContentSized;
use cuicui_layout::{LeafRule, Node, Size};

/// Update the [`cuicui_layout`] [`Node::Box`] [`LeafRule::Fixed`] values of
/// entities with a [`bevy::ui::Node`] component.
#[allow(clippy::needless_pass_by_value)]
pub fn update(
    compute: TextCompute,
    mut query: Query<
        (&mut Node, &ContentSized, AnyOf<(&Text, &UiImageSize)>),
        Or<(
            Changed<ui::Node>,
            Changed<Text>,
            Changed<UiImageSize>,
            Changed<ContentSized>,
        )>,
    >,
) {
    for (mut node, sized, bevy_ui) in &mut query {
        let bevy_ui = match bevy_ui {
            (Some(text), None) => compute.bounds(text, Vec2::INFINITY),
            (None, Some(image)) => image.size(),
            _ => unreachable!("This is a bevy bug"),
        };
        if sized.managed_axis.width {
            if let Node::Box(Size { width, .. }) = &mut *node {
                *width = LeafRule::Fixed(bevy_ui.x);
            }
        }
        if sized.managed_axis.height {
            if let Node::Box(Size { height, .. }) = &mut *node {
                *height = LeafRule::Fixed(bevy_ui.y);
            }
        }
    }
}

/// [`update`] parameter to compute text extents based on `cuicui_layout`, rather
/// than `bevy_ui`'s flexbox.
#[derive(SystemParam)]
pub struct TextCompute<'w> {
    fonts: Res<'w, Assets<Font>>,
}
impl TextCompute<'_> {
    /// Due to a regression in bevy 0.11, it is now impossible to access
    /// text size pre-layouting, therefore this nonsense is needed.
    fn bounds(&self, text: &Text, bounds: Vec2) -> Vec2 {
        trace!("Recomputing text sizes");

        let measure = TextPipeline::default().create_text_measure(
            &self.fonts,
            &text.sections,
            // Seems like this requires an epsilon, otherwise text wraps poorly.
            1.01,
            text.alignment,
            text.linebreak_behavior,
        );
        measure.map_or(Vec2::ZERO, |m| m.compute_size(bounds))
    }
}
