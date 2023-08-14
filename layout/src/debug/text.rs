//! Display node information at top left of node's bounds.

use bevy::ecs::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::log::error;
use bevy::math::Vec2Swizzles;
use bevy::prelude::{Color, GlobalTransform, Image, Transform, Vec2};
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
use bevy::sprite::{Anchor, Sprite};
use bevy::text::{Text, Text2dBounds, Text2dBundle, TextLayoutInfo, TextStyle};
use bevy::utils::hashbrown::hash_map::Entry;
use bevy::utils::{default, HashMap};
use bevy::window::{PrimaryWindow, Window};

use super::{CameraQuery, LAYOUT_DEBUG_LAYERS};

const TEXT_BG: Color = Color::rgba(0., 0., 0., 0.8);

#[derive(Component)]
pub(super) struct OverlayText;

struct DebugText {
    text: Box<str>,
    color: Color,
    at: Vec2,
    text_entity: Entity,
}

#[derive(Resource, Default)]
pub(super) struct ImmediateTexts {
    texts: HashMap<Entity, DebugText>,
    set_texts: HashMap<Entity, bool>,
}
impl ImmediateTexts {
    fn offset(&self, _source: Entity, at: Vec2, _max_offset: f32) -> Vec2 {
        at
    }
    fn insert_text(
        &mut self,
        source: Entity,
        text: &str,
        color: Color,
        at: Vec2,
        max_offset: f32,
    ) -> bool {
        let at = self.offset(source, at, max_offset);
        match self.texts.entry(source) {
            Entry::Occupied(debug) => {
                let debug = debug.into_mut();
                let changed = debug.at != at || debug.color != color || debug.text.as_ref() != text;
                if changed {
                    debug.text = text.into();
                    debug.color = color;
                    debug.at = at;
                }
                changed
            }
            Entry::Vacant(vacant) => {
                let text = text.into();
                vacant.insert(DebugText { text, color, at, text_entity: Entity::PLACEHOLDER });
                true
            }
        }
    }
    fn print(&mut self, source: Entity, text: &str, at: Vec2, max_offset: f32, color: Color) {
        let changed = self.insert_text(source, text, color, at, max_offset);
        self.set_texts.insert(source, changed);
    }
    // Callback-based because Rust doesn't support lending iterators.
    fn for_each_changed(&mut self, mut f: impl FnMut(&mut DebugText)) {
        let changed = self.set_texts.iter().filter_map(|(k, v)| v.then_some(k));
        for entity in changed {
            let debug = unsafe { self.texts.get_mut(entity).unwrap_unchecked() };
            f(debug);
        }
    }
    fn reset(&mut self, cmds: &mut Commands) {
        self.texts.retain(|entity, v| {
            if !self.set_texts.contains_key(entity) {
                cmds.entity(v.text_entity).despawn();
                return false;
            }
            true
        });
        self.set_texts.clear();
    }
}

#[derive(SystemParam)]
pub(super) struct TextGizmo<'w, 's> {
    texts: ResMut<'w, ImmediateTexts>,
    cam: CameraQuery<'w, 's>,
    cmds: Commands<'w, 's>,
    existing: Query<'w, 's, (&'static mut Text, &'static mut Transform), With<Text2dBounds>>,
}
impl<'w, 's> TextGizmo<'w, 's> {
    fn relative(&self, mut position: Vec2) -> Vec2 {
        let zero = GlobalTransform::IDENTITY;
        let Ok((cam, debug)) = self.cam.get_single() else { return Vec2::ZERO;};
        if debug.screen_space {
            if let Some(new_position) = cam.world_to_viewport(&zero, position.extend(0.)) {
                position = new_position;
            };
        }
        position.xy()
    }
    pub(super) fn print(
        &mut self,
        source: Entity,
        text: &str,
        at: Vec2,
        max_offset: f32,
        color: Color,
    ) {
        let at = self.relative(at);
        self.texts.print(source, text, at, max_offset, color);
    }
    /// Spawn/update accumulated text gizmos.
    pub(super) fn update(&mut self) {
        let anchor = get_anchor(&self.cam);

        self.texts.for_each_changed(|debug| {
            if debug.text_entity == Entity::PLACEHOLDER {
                let style = TextStyle { color: debug.color, ..default() };
                let text = Text::from_section(debug.text.clone(), style);
                let entity = self.cmds.spawn((
                    Text2dBundle { text, text_anchor: anchor.clone(), ..default() },
                    OverlayText,
                    LAYOUT_DEBUG_LAYERS,
                ));
                debug.text_entity = entity.id();
            } else {
                let Ok((mut text, mut transform)) = self.existing.get_mut(debug.text_entity) else {
                    error!("This is a programming error on the debug impl of cuicui_layout");
                    return;
                };
                transform.translation = debug.at.extend(0.);
                text.sections[0].style.color = debug.color;
                if text.sections[0].value != debug.text.as_ref() {
                    text.sections[0].value.clear();
                    text.sections[0].value.push_str(&debug.text);
                }
            }
        });
    }
    /// Remove all debug texts from the world.
    pub(super) fn reset(&mut self) {
        self.texts.reset(&mut self.cmds);
    }
}
fn get_anchor(cam: &CameraQuery) -> Anchor {
    use Anchor::{BottomLeft, TopLeft};
    let Ok((_, debug)) = cam.get_single() else { return TopLeft;};
    if debug.screen_space {
        BottomLeft
    } else {
        TopLeft
    }
}
pub(super) fn overlay_dark_background(
    mut cmds: Commands,
    mut query: Query<
        (Entity, &TextLayoutInfo, Option<&mut Sprite>),
        (Added<OverlayText>, Changed<TextLayoutInfo>),
    >,
    cam: CameraQuery,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let scale_factor = Window::scale_factor;
    let window_scale = window.get_single().map_or(1., scale_factor) as f32;
    for (entity, new_layout, sprite) in &mut query {
        if let Some(mut sprite) = sprite {
            sprite.anchor = get_anchor(&cam);
            sprite.custom_size = Some(new_layout.size / window_scale);
        } else {
            let anchor = get_anchor(&cam);
            let custom_size = Some(new_layout.size / window_scale);
            cmds.entity(entity).insert((
                Sprite {
                    color: TEXT_BG,
                    custom_size,
                    anchor,
                    ..Default::default()
                },
                DEFAULT_IMAGE_HANDLE.typed::<Image>(),
            ));
        }
    }
}
