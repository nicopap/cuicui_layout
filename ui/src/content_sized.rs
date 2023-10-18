//! Systems to update a [`cuicui_layout::Node`]'s size based on an image's
//! and text's size, rather that fixed at spawn time.
use bevy::asset::Assets;
use bevy::ecs::prelude::*;
use bevy::prelude::{App, Plugin, Update, Vec2};
use bevy::text::{Font, Text, TextPipeline};
use bevy::ui::widget::UiImageSize;
use cuicui_layout::{ContentSized, ContentSizedSet, Node};

/// Adjust the [`cuicui_layout`] [`ContentSized`] component based on `bevy_ui`
/// sizeable content.
pub struct UiContentSizePlugin;

impl Plugin for UiContentSizePlugin {
    fn build(&self, app: &mut App) {
        use bevy::ecs::schedule::common_conditions::resource_changed as res;

        fn changed<C: Component>(q: Query<(), (Changed<C>, With<Node>)>) -> bool {
            !q.is_empty()
        }
        app.add_systems(
            Update,
            (
                image_content_size.run_if(changed::<UiImageSize>),
                text_content_size.run_if(changed::<Text>.or_else(res::<Assets<Font>>())),
            )
                .in_set(ContentSizedSet),
        );
    }
}

type QueryContentSize<'w, 's, T> =
    Query<'w, 's, (&'static mut ContentSized, &'static T), Or<(Added<ContentSized>, Changed<T>)>>;

fn image_content_size(mut query: QueryContentSize<UiImageSize>) {
    for (mut content_size, image) in &mut query {
        content_size.0 = image.size().into();
    }
}

fn text_content_size(fonts: Res<Assets<Font>>, mut query: QueryContentSize<Text>) {
    for (mut content_size, text) in &mut query {
        let set_size = content_size.0;
        let bounds = Vec2::from(set_size);
        let measure = TextPipeline::default().create_text_measure(
            &fonts,
            &text.sections,
            // Seems like this requires an epsilon, otherwise text wraps poorly.
            1.01,
            text.alignment,
            text.linebreak_behavior,
        );
        if let Ok(measure) = measure {
            content_size.0 = measure.compute_size(bounds).into();
        }
    }
}
