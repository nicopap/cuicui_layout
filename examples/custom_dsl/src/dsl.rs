use bevy::{prelude::*, reflect::TypeRegistryInternal as TypeRegistry};
use cuicui_chirp::{
    anyhow::{Context, Result},
    parse_dsl_impl,
};
use cuicui_dsl::{DslBundle, EntityCommands};
use cuicui_examples::{switchable_impl, GetIndex, Highlight, SwitchPlugin, Switchable};
use cuicui_layout_bevy_ui::UiDsl;

use crate::reflect_on_click::ReflectOnClick;

switchable_impl! {
    TabButton[Tabs, SwitchTab],
    GraphButton[Graphs, SwitchGraph],
}

#[allow(clippy::explicit_iter_loop)]
fn switch_color<T: Switchable>(
    mut tab_requests: EventReader<T::Event>,
    mut tab_buttons: Query<(&mut BackgroundColor, &T)>,
) {
    for req in tab_requests.iter() {
        for (mut bg, button) in &mut tab_buttons {
            let (highlight, shade) = (Color::rgb_u8(222, 184, 135), Color::rgb_u8(107, 77, 34));
            bg.0 = if button.index() == req.index() { highlight } else { shade };
        }
    }
}

struct Marker(Box<dyn FnOnce(&mut EntityCommands)>);
impl Marker {
    fn reflect_from_world(builder: ReflectComponent) -> Self {
        Self(Box::new(move |cmds| {
            cmds.add(move |id, world: &mut World| {
                let component = builder.from_world(world);
                builder.insert(&mut world.entity_mut(id), &*component);
            });
        }))
    }
}

#[derive(Default)]
pub struct BetterFactorioDsl {
    inner: UiDsl,
    switch_graph: Option<u8>,
    is_hidden: bool,
    is_highlight: bool,
    markers: Vec<Marker>,
    switch_tab: Option<u8>,
    text_to_print: Option<Box<str>>,
}

fn parse_marker<T>(reg: &TypeRegistry, _: T, input: &str) -> Result<Marker> {
    let not_reg = || format!("{input} not registered");
    let not_comp = || format!("{input}'s ReflectComponent is not registered.");

    let type_id = reg.get_with_short_name(input);
    let type_id = type_id.with_context(not_reg)?.type_id();

    let builder = reg.get_type_data::<ReflectComponent>(type_id);
    let builder = builder.with_context(not_comp)?;

    Ok(Marker::reflect_from_world(builder.clone()))
}

#[parse_dsl_impl(delegate = inner, type_parsers(Marker = parse_marker))]
// ANCHOR: hidden_method
impl BetterFactorioDsl {
    fn hidden(&mut self) {
        self.is_hidden = true;
    }
    // ANCHOR: game_menu_methods ANCHOR_END: hidden_method
    fn print_text(&mut self, text: &str) {
        self.text_to_print = Some(text.into());
    }
    fn highlight(&mut self) {
        self.is_highlight = true;
    }
    // ANCHOR: switch_tab_method ANCHOR_END: game_menu_methods
    fn switch_tab(&mut self, index: u8) {
        self.switch_tab = Some(index);
    }
    // ANCHOR_END: switch_tab_method
    fn switch_graph(&mut self, index: u8) {
        self.switch_graph = Some(index);
    }
    fn marked(&mut self, marker: Marker) {
        self.markers.push(marker);
    }
}

impl DslBundle for BetterFactorioDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        for marker in self.markers.drain(..) {
            (marker.0)(cmds);
        }
        if let Some(index) = self.switch_graph {
            cmds.insert((ReflectOnClick::EmitSwitchGraph(index), GraphButton(index)));
        }
        // ANCHOR: switch_tab_insert
        if let Some(index) = self.switch_tab {
            cmds.insert((ReflectOnClick::EmitSwitchTab(index), TabButton(index)));
        }
        // ANCHOR: game_menu_inserts ANCHOR_END: switch_tab_insert
        if let Some(text) = self.text_to_print.take() {
            cmds.insert(ReflectOnClick::LogInfo(text.into()));
        }
        if self.is_highlight {
            cmds.insert(Highlight::new(Color::BEIGE));
        }
        // ANCHOR: add_hidden ANCHOR_END: game_menu_inserts
        let id = self.inner.insert(cmds);
        if self.is_hidden {
            cmds.insert(Visibility::Hidden);
        }
        id
        // ANCHOR_END: add_hidden
    }
}

pub struct DslPlugin;

impl Plugin for DslPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SwitchPlugin::<TabButton>::new(),
            SwitchPlugin::<GraphButton>::new(),
        ))
        .add_systems(
            Update,
            (switch_color::<TabButton>, switch_color::<GraphButton>),
        );
    }
}
