use bevy::ecs::query::QueryItem;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy::reflect::ReflectRef;
use cuicui_dsl::EntityCommands;

use crate::{button_shift, BgColor};

type StyleComponents = AnyOf<(
    &'static mut Text,
    &'static mut BgColor,
    &'static mut UiImage,
    &'static mut button_shift::Animation,
)>;

#[derive(Reflect, Debug, Clone, Copy, Default)]
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
    pub text_inner_offset: u8,
    pub enable_speed: Fract,
    pub disable_speed: Fract,
}
const RED_INDEX: u8 = 0;
const BLUE_INDEX: u8 = 2;
const YELLOW_INDEX: u8 = 4;
const GREY_INDEX: u8 = 5;
const TRANSPARENT_INDEX: u8 = 6;
#[derive(Reflect, Debug)]
pub struct Palette {
    pub red: Color,
    pub red_dim: Color,
    pub blue: Color,
    pub purple: Color,
    pub yellow: Color,
    pub grey: Color,
    pub transparent: Color,
    pub standby_item_outline_index: u8,
    pub standby_text_index: u8,
    pub hover_index: u8,
    pub settings_category_bg_index: u8,
    pub settings_category_text_index: u8,
}
impl Palette {
    fn get_color(&self, field_index: usize) -> Color {
        let ReflectRef::Struct(self_reflect) = self.reflect_ref() else {
            unreachable!();
        };
        let Some(color) = self_reflect.field_at(field_index) else {
            warn!("Invalid field_index: {field_index};");
            return Color::PINK;
        };
        *color.as_any().downcast_ref().unwrap()
    }
    pub fn standby_item_outline(&self) -> Color {
        self.get_color(usize::from(self.standby_item_outline_index))
    }
    pub fn standby_text(&self) -> Color {
        self.get_color(usize::from(self.standby_text_index))
    }
    pub fn hover(&self) -> Color {
        self.get_color(usize::from(self.hover_index))
    }
    pub fn settings_category_bg(&self) -> Color {
        self.get_color(usize::from(self.settings_category_bg_index))
    }
    pub fn settings_category_text(&self) -> Color {
        self.get_color(usize::from(self.settings_category_text_index))
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
#[derive(Reflect, Debug)]
pub struct Images {
    item_button: Handle<Image>,
}
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct Bevypunk {
    pub fonts: Fonts,
    pub images: Images,
    pub palette: Palette,
    pub button_animation: ButtonAnimation,
}

impl Default for ButtonAnimation {
    fn default() -> Self {
        ButtonAnimation {
            item_offset: 50,
            text_inner_offset: 0,
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
            transparent: Color::rgba_u8(255, 98, 81, 0),
            standby_item_outline_index: TRANSPARENT_INDEX,
            standby_text_index: RED_INDEX,
            hover_index: BLUE_INDEX,
            settings_category_bg_index: GREY_INDEX,
            settings_category_text_index: YELLOW_INDEX,
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
            size: 16,
            main_item_size: 22,
        }
    }
}
impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Images {
            item_button: assets.load("images/main_menu/button.png"),
        }
    }
}
impl FromWorld for Bevypunk {
    fn from_world(world: &mut World) -> Self {
        Bevypunk {
            fonts: Fonts::from_world(world),
            images: Images::from_world(world),
            palette: Palette::from_world(world),
            button_animation: ButtonAnimation::from_world(world),
        }
    }
}

#[derive(Component, Clone, Default, Reflect, Debug)]
#[reflect(Component)]
pub enum Element {
    #[default]
    Panel,
    MainMenuItemButton,
    MainMenuItemText,
    TabButton,
    OptionEntry,
    SettingsHeader,
    SettingsHeaderText,
}
impl Element {
    fn shift_animation(&self, style: &Bevypunk) -> Option<button_shift::Animation> {
        use Element::{OptionEntry, Panel, SettingsHeader, SettingsHeaderText, TabButton};
        match self {
            Panel | SettingsHeader | SettingsHeaderText => None,
            Element::MainMenuItemButton => Some(button_shift::Animation {
                rest_color: style.palette.standby_item_outline(),
                active_color: style.palette.hover(),
                active_right_shift: style.button_animation.item_offset,
                enable_speed: style.button_animation.enable_speed,
                disable_speed: style.button_animation.disable_speed,
            }),
            Element::MainMenuItemText => Some(button_shift::Animation {
                rest_color: style.palette.standby_text(),
                active_color: style.palette.hover(),
                active_right_shift: style.button_animation.text_inner_offset,
                enable_speed: style.button_animation.enable_speed,
                disable_speed: style.button_animation.disable_speed,
            }),
            TabButton | OptionEntry => Some(button_shift::Animation {
                rest_color: style.palette.standby_text(),
                active_color: style.palette.hover(),
                active_right_shift: 0,
                enable_speed: style.button_animation.enable_speed,
                disable_speed: style.button_animation.disable_speed,
            }),
        }
    }
    fn set_style(&self, style: &Bevypunk, (text, bg, ui_image, anim): QueryItem<StyleComponents>) {
        match self {
            Element::Panel => {}
            Element::MainMenuItemButton => {
                let mut ui_image = ui_image.unwrap();
                let mut anim = anim.unwrap();
                ui_image.texture = style.images.item_button.clone_weak();
                *anim = self.shift_animation(style).unwrap();
            }
            Element::MainMenuItemText => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.main_menu.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.main_item_size);
                *anim = self.shift_animation(style).unwrap();
            }
            Element::TabButton => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.tabline.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.size);
                *anim = self.shift_animation(style).unwrap();
            }
            Element::OptionEntry => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.options.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.size);
                *anim = self.shift_animation(style).unwrap();
            }
            Element::SettingsHeader => {
                let mut bg = bg.unwrap();
                bg.0 = style.palette.settings_category_bg();
            }
            Element::SettingsHeaderText => {
                let mut text = text.unwrap();
                text.sections[0].style.font = style.fonts.options.clone_weak();
                text.sections[0].style.color = style.palette.settings_category_text();
                text.sections[0].style.font_size = f32::from(style.fonts.size);
            }
        }
    }

    pub(crate) fn insert(self, cmds: &mut EntityCommands) {
        let animation = button_shift::Animation::default;
        let state = button_shift::State::default;
        let shift = || (animation(), state());
        let ui_image = UiImage::default;
        let text = || Text::from_section("", default());
        let bg = BgColor::default;
        match self.clone() {
            Element::Panel => {}
            Element::MainMenuItemButton => {
                cmds.insert((ui_image(), shift(), self));
            }
            Element::MainMenuItemText | Element::TabButton | Element::OptionEntry => {
                cmds.insert((text(), shift(), self));
            }
            Element::SettingsHeader => {
                cmds.insert((bg(), self));
            }
            Element::SettingsHeaderText => {
                cmds.insert((text(), self));
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
