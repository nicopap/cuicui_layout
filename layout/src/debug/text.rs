//! Display node information at top left of node's bounds.

use bevy::ecs::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Color, Transform, Vec2};
use bevy::sprite::Anchor;
use bevy::text::{Text, Text2dBounds, Text2dBundle, TextStyle};
use bevy::utils::hashbrown::hash_map::Entry;
use bevy::utils::{default, HashMap};

use super::LAYOUT_DEBUG_LAYERS;

struct DebugText {
    text: Box<str>,
    color: Color,
    at: Vec2,
    text_entity: Entity,
}
impl DebugText {}

#[derive(Resource, Default)]
pub(super) struct ImmediateTexts {
    texts: HashMap<Entity, DebugText>,
    set_texts: HashMap<Entity, bool>,
}
impl ImmediateTexts {
    fn insert_text(&mut self, source: Entity, text: &str, color: Color, at: Vec2) -> bool {
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
    pub(super) fn print(&mut self, source: Entity, text: &str, at: Vec2, color: Color) {
        let changed = self.insert_text(source, text, color, at);
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
    cmds: Commands<'w, 's>,
    existing: Query<'w, 's, (&'static mut Text, &'static mut Transform), With<Text2dBounds>>,
}
impl<'w, 's> TextGizmo<'w, 's> {
    /// Spawn/update accumulated text gizmos.
    pub(super) fn update(&mut self) {
        self.texts.for_each_changed(|debug| {
            if debug.text_entity == Entity::PLACEHOLDER {
                let style = TextStyle { color: debug.color, ..default() };
                let text = Text::from_section(debug.text.clone(), style);
                let entity = self.cmds.spawn((
                    Text2dBundle { text, text_anchor: Anchor::TopLeft, ..default() },
                    LAYOUT_DEBUG_LAYERS,
                ));
                debug.text_entity = entity.id();
            } else {
                let Ok((mut text, mut transform)) = self.existing.get_mut(debug.text_entity) else {
                    panic!("This is a programming error on the debug impl of cuicui_layout");
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
