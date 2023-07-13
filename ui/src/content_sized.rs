//! Systems to update a [`cuicui_layout::Node`]'s size based on an image's
//! and text's size, rather that fixed at spawn time.
//!
//! This relies on the [`bevy::ui::Node`] component.
use bevy::{
    ecs::{prelude::*, system::SystemParam},
    prelude::{trace, Assets, Vec2},
    text::{Font, Text, TextPipeline},
    ui::widget::UiImageSize,
};
use cuicui_layout::{ComputeContentParam, ComputeContentSize, Size};

#[derive(SystemParam)]
pub(crate) struct UiContentSize<'w> {
    fonts: Res<'w, Assets<Font>>,
}
impl ComputeContentParam for UiContentSize<'static> {
    type Components = AnyOf<(&'static Text, &'static UiImageSize)>;
}
impl UiContentSize<'_> {
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
impl ComputeContentSize for UiContentSize<'_> {
    type Components = AnyOf<(&'static Text, &'static UiImageSize)>;

    fn compute_content(
        &self,
        components: (Option<&Text>, Option<&UiImageSize>),
        set_size: Size<Option<f32>>,
    ) -> Size<f32> {
        let inf = f32::INFINITY;
        let size_vec = Vec2::new(
            set_size.width.unwrap_or(inf),
            set_size.height.unwrap_or(inf),
        );
        let bevy_ui = match components {
            (Some(text), None) => {
                trace!("Ui Text content size to re-compute");
                self.bounds(text, size_vec)
            }
            (None, Some(image)) => {
                trace!("UiImage content size to re-compute");
                image.size()
            }
            _ => unreachable!("This is a bevy bug"),
        };
        bevy_ui.into()
    }
}
