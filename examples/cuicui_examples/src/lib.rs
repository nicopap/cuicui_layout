#![allow(
    clippy::needless_pass_by_value,
    clippy::missing_const_for_fn,
    clippy::module_name_repetitions
)]
use bevy::{log::LogPlugin, prelude::default};

pub use highlight::{Highlight, HighlightPlugin};
pub use mirror::{FromMirror, MirrorPlugin, MirrorSystems, ToMirror};
pub use switch::{GetIndex, SwitchPlugin, Switchable};

pub mod highlight;
pub mod mirror;
pub mod switch;

#[must_use]
pub fn log_plugin(advanced: bool) -> LogPlugin {
    if advanced {
        LogPlugin {
            level: bevy::log::Level::TRACE,
            filter: "\
          cuicui_layout=info,cuicui_layout_bevy_ui=info,\
          cuicui_chirp=trace,\
          cuicui_chirp::interpret=trace,\
          gilrs_core=info,gilrs=info,\
          naga=info,wgpu=error,wgpu_hal=error,\
          bevy_app=info,bevy_render::render_resource::pipeline_cache=info,\
          bevy_render::view::window=info,bevy_ecs::world::entity_ref=info"
                .to_string(),
        }
    } else {
        default()
    }
}
