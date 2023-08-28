use bevy::{
    ecs::query::QueryItem,
    prelude::{Plugin as BevyPlugin, *},
};

use crate::{button_shift, StyleComponents};

#[derive(Reflect, Debug, Clone, Copy)]
pub struct Fract(u8);
impl Fract {
    pub fn get(self) -> f32 {
        self.into()
    }
    pub fn new(arg: f32) -> Fract {
        arg.try_into().unwrap()
    }
}
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl TryFrom<f32> for Fract {
    type Error = ();
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !(0_f32..=2.).contains(&value) {
            return Err(());
        }
        Ok(Fract((value * 128.) as u8))
    }
}
impl From<Fract> for f32 {
    fn from(Fract(value): Fract) -> Self {
        f32::from(value) / 128.
    }
}
impl From<Fract> for f64 {
    fn from(Fract(value): Fract) -> Self {
        f64::from(value) / 128.
    }
}

#[derive(Reflect, Debug)]
pub struct ButtonAnimation {
    pub item_offset: u8,
    pub enable_speed: Fract,
    pub disable_speed: Fract,
}
#[derive(Reflect, Debug)]
pub struct Palette {
    pub red: Color,
    pub red_dim: Color,
    pub blue: Color,
    pub purple: Color,
    pub yellow: Color,
    pub grey: Color,
}
impl Palette {
    pub const fn standby(&self) -> Color {
        self.red
    }
    pub const fn hover(&self) -> Color {
        self.blue
    }
    pub const fn settings_category(&self) -> Color {
        self.grey
    }
}
#[derive(Reflect, Debug)]
pub struct Fonts {
    pub navigation: Handle<Font>,
    pub options: Handle<Font>,
    pub item: Handle<Font>,
    pub tabline: Handle<Font>,
    pub main_menu: Handle<Font>,
    pub size: u8,
    pub main_item_size: u8,
}
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct Bevypunk {
    pub fonts: Fonts,
    pub palette: Palette,
    pub button_animation: ButtonAnimation,
}

impl Default for ButtonAnimation {
    fn default() -> Self {
        ButtonAnimation {
            item_offset: 50,
            enable_speed: Fract::new(0.1),
            disable_speed: Fract::new(1.0),
        }
    }
}
impl Default for Palette {
    fn default() -> Self {
        Palette {
            red: Color::rgb_u8(255, 98, 81),
            red_dim: Color::rgb_u8(204, 56, 51),
            blue: Color::rgb_u8(42, 237, 247),
            purple: Color::rgb_u8(255, 34, 245),
            yellow: Color::rgb_u8(255, 245, 34),
            grey: Color::rgb_u8(199, 186, 174),
        }
    }
}
impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Fonts {
            navigation: assets.load("fonts/rajdhani/Rajdhani-Bold.ttf"),
            options: assets.load("fonts/rajdhani/Rajdhani-SemiBold.ttf"),
            item: assets.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
            tabline: assets.load("fonts/blender/BlenderPro-Medium.ttf"),
            main_menu: assets.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
            size: 40,
            main_item_size: 80,
        }
    }
}
impl FromWorld for Bevypunk {
    fn from_world(world: &mut World) -> Self {
        Bevypunk {
            fonts: Fonts::from_world(world),
            palette: Palette::from_world(world),
            button_animation: ButtonAnimation::from_world(world),
        }
    }
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component)]
enum Element {
    #[default]
    Panel,
    MainMenuItemButton,
    TabButton,
    OptionEntry,
}
impl Element {
    fn set_style(&self, style: &Bevypunk, (_ui_style, text, _, anim): QueryItem<StyleComponents>) {
        match self {
            Element::Panel => {}
            Element::MainMenuItemButton => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.item.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.main_item_size);
                text.sections[0].style.color = style.palette.standby();
                *anim = button_shift::Animation::Toggle {
                    rest_color: style.palette.standby(),
                    active_color: style.palette.hover(),
                    active_left_shift: style.button_animation.item_offset,
                    enable_speed: style.button_animation.enable_speed,
                    disable_speed: style.button_animation.disable_speed,
                };
            }
            Element::TabButton => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.tabline.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.size);
                text.sections[0].style.color = style.palette.standby();
                *anim = button_shift::Animation::Toggle {
                    rest_color: style.palette.standby(),
                    active_color: style.palette.hover(),
                    active_left_shift: 0,
                    enable_speed: style.button_animation.enable_speed,
                    disable_speed: style.button_animation.disable_speed,
                };
            }
            Element::OptionEntry => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.options.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.size);
                text.sections[0].style.color = style.palette.standby();
                *anim = button_shift::Animation::Toggle {
                    rest_color: style.palette.standby(),
                    active_color: style.palette.hover(),
                    active_left_shift: 0,
                    enable_speed: style.button_animation.enable_speed,
                    disable_speed: style.button_animation.disable_speed,
                };
            }
        }
    }
}

// TODO(clean): replace this with query bindings
// .run_if(resource_changed::<BevypunkStyle>)
fn element_on_style_change(style: Res<Bevypunk>, mut elements: Query<(&Element, StyleComponents)>) {
    for (element, style_components) in &mut elements {
        element.set_style(&style, style_components);
    }
}
fn element_on_element_change(
    style: Res<Bevypunk>,
    mut elements: Query<(&Element, StyleComponents), Changed<Element>>,
) {
    for (element, style_components) in &mut elements {
        element.set_style(&style, style_components);
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        let systems = (
            element_on_style_change.run_if(resource_changed::<Bevypunk>()),
            element_on_element_change,
        );
        app.init_resource::<Bevypunk>()
            .register_type::<Element>()
            .register_type::<Fonts>()
            .register_type::<Palette>()
            .register_type::<Fract>()
            .register_type::<Bevypunk>()
            .add_systems(Update, systems);
    }
}
