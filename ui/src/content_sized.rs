//! Systems to update a [`cuicui_layout::Node`]'s size based on an image's
//! and text's size, rather that fixed at spawn time.
//!
//! This relies on the [`bevy::ui::Node`] component.
use bevy::asset::Assets;
use bevy::ecs::{prelude::*, schedule::SystemSetConfig, system::SystemParam};
use bevy::log::trace;
use bevy::prelude::Vec2;
use bevy::text::{Font, Text, TextPipeline};
use bevy::ui::widget::UiImageSize;
use cuicui_layout::{require_layout_recompute, Node, Size};
use cuicui_layout::{ComputeContentParam, ComputeContentSize, ContentSizedComputeSystem};

#[derive(SystemParam)]
pub(crate) struct UiContentSize<'w> {
    fonts: Res<'w, Assets<Font>>,
}
impl ComputeContentParam for UiContentSize<'static> {
    type Components = AnyOf<(&'static Text, &'static UiImageSize)>;

    fn condition(label: ContentSizedComputeSystem<Self>) -> SystemSetConfig {
        use bevy::ecs::schedule::common_conditions as cond;

        let cond = cond::resource_changed::<Assets<Font>>()
            .or_else(|c: Query<(), (Changed<UiImageSize>, With<Node>)>| !c.is_empty())
            .or_else(|c: Query<(), (Changed<Text>, With<Node>)>| !c.is_empty());

        label.run_if(require_layout_recompute.or_else(cond))
    }
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
fn compute_image_size(size: Vec2, set_size: Size<Option<f32>>) -> Vec2 {
    let size = match (set_size.width, set_size.height) {
        (None, None) => size,
        (Some(width), None) => Vec2::new(width, width * size.y / size.x),
        (None, Some(height)) => Vec2::new(height * size.x / size.y, height),
        (Some(_), Some(_)) => unreachable!(
            "This is a bug in cuicui_layout, the API promises that \
            compute_content is never called with two set values."
        ),
    };
    // `UiImageSize` is NaN when the image is not loaded yet. This messes
    // with cuicui_layout which is picky about errors.
    Vec2::select(size.is_nan_mask(), Vec2::ZERO, size)
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
            (Some(text), _) => self.bounds(text, size_vec),
            (None, Some(image)) => compute_image_size(image.size(), set_size),
            (None, None) => {
                unreachable!("This is a bevy bug: AnyOf should at least have one element")
            }
        };
        bevy_ui.into()
    }
}
