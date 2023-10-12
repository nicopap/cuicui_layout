use bevy::ecs::query::QueryItem;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy::reflect::ReflectRef;
use cuicui_dsl::EntityCommands;

use crate::ui_offset::UiOffset;
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
    pub fn new(arg: f32) -> Self {
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
        Ok(Self((value * 128.) as u8))
    }
}
impl From<Fract> for f32 {
    fn from(Fract(value): Fract) -> Self {
        Self::from(value) / 128.
    }
}
impl From<Fract> for f64 {
    fn from(Fract(value): Fract) -> Self {
        Self::from(value) / 128.
    }
}

#[derive(Reflect, Debug)]
struct ButtonAnimation {
    item_offset: u8,
    text_inner_offset: u8,
    enable_speed: Fract,
    disable_speed: Fract,
}
const RED_INDEX: u8 = 0;
const BLUE_INDEX: u8 = 2;
const GREY_INDEX: u8 = 5;
const TRANSPARENT_INDEX: u8 = 6;
const SUBTILE_MARINE_INDEX: u8 = 7;

#[derive(Reflect, Debug)]
struct Palette {
    red: Color,
    red_dim: Color,
    blue: Color,
    purple: Color,
    yellow: Color,
    grey: Color,
    transparent: Color,
    subtile_marine: Color,
    standby_item_outline_index: u8,
    standby_text_index: u8,
    hover_index: u8,
    settings_category_bg_index: u8,
    settings_category_text_index: u8,
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
    fn standby_item_outline(&self) -> Color {
        self.get_color(usize::from(self.standby_item_outline_index))
    }
    fn standby_text(&self) -> Color {
        self.get_color(usize::from(self.standby_text_index))
    }
    fn hover(&self) -> Color {
        self.get_color(usize::from(self.hover_index))
    }
    fn settings_category_bg(&self) -> Color {
        self.get_color(usize::from(self.settings_category_bg_index))
    }
    fn settings_category_text(&self) -> Color {
        self.get_color(usize::from(self.settings_category_text_index))
    }
}
#[derive(Reflect, Debug)]
struct Fonts {
    navigation: Handle<Font>,
    options: Handle<Font>,
    item: Handle<Font>,
    tabline: Handle<Font>,
    main_menu: Handle<Font>,
    size: u8,
    main_item_size: u8,
}
#[derive(Reflect, Debug)]
struct Images {
    item_button: Handle<Image>,
    // left and right arrows have sprite flipped duh.
    option_arrow: Handle<Image>,
    option_shadow: Handle<Image>,
}
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
struct Bevypunk {
    fonts: Fonts,
    images: Images,
    palette: Palette,
    button_animation: ButtonAnimation,
}

impl Default for ButtonAnimation {
    fn default() -> Self {
        Self {
            item_offset: 20,
            text_inner_offset: 0,
            enable_speed: Fract::new(0.1),
            disable_speed: Fract::new(0.3),
        }
    }
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            red: Color::rgb_u8(255, 98, 81),
            red_dim: Color::rgb_u8(204, 56, 51),
            blue: Color::rgb_u8(42, 237, 247),
            purple: Color::rgb_u8(255, 34, 245),
            yellow: Color::rgb_u8(255, 245, 34),
            grey: Color::rgb_u8(199, 186, 174),
            transparent: Color::rgba_u8(255, 98, 81, 0),
            subtile_marine: Color::rgba_u8(17, 17, 55, 62),
            standby_item_outline_index: TRANSPARENT_INDEX,
            standby_text_index: RED_INDEX,
            hover_index: BLUE_INDEX,
            settings_category_bg_index: SUBTILE_MARINE_INDEX,
            settings_category_text_index: GREY_INDEX,
        }
    }
}
impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            navigation: assets.load("fonts/rajdhani/Rajdhani-Bold.ttf"),
            options: assets.load("fonts/rajdhani/Rajdhani-SemiBold.ttf"),
            item: assets.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
            tabline: assets.load("fonts/blender/BlenderPro-Medium.ttf"),
            main_menu: assets.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
            size: 25,
            main_item_size: 29,
        }
    }
}
impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            item_button: assets.load("images/main_menu/button.png"),
            option_arrow: assets.load("images/settings/arrow_left_empty.png"),
            option_shadow: assets.load("images/settings/selection_shadow.png"),
        }
    }
}
impl FromWorld for Bevypunk {
    fn from_world(world: &mut World) -> Self {
        Self {
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
    TabText,
    OptionEntry,
    OptionRow,
    SettingsHeader,
    SettingsHeaderText,
    OptionBoxLArrow,
    OptionBoxRArrow,
    OptionBoxChoice,
    OptionBox,
    BackText,
    OptionTick,
}
impl Element {
    fn shift_animation(&self, style: &Bevypunk) -> Option<button_shift::Animation> {
        use Element::{OptionBox, OptionBoxChoice, OptionBoxLArrow, OptionBoxRArrow, TabText};
        use Element::{OptionEntry, Panel, SettingsHeader, SettingsHeaderText, TabButton};
        use Element::{OptionRow, OptionTick};
        match self {
            Panel | SettingsHeader | SettingsHeaderText | OptionTick => None,
            Self::MainMenuItemButton => Some(button_shift::Animation {
                rest_color: style.palette.standby_item_outline(),
                active_color: style.palette.hover(),
                active_right_shift: style.button_animation.item_offset,
                enable_speed: style.button_animation.enable_speed,
                disable_speed: style.button_animation.disable_speed,
            }),
            Self::MainMenuItemText | Self::BackText | TabText => Some(button_shift::Animation {
                rest_color: style.palette.standby_text(),
                active_color: style.palette.hover(),
                active_right_shift: style.button_animation.text_inner_offset,
                enable_speed: style.button_animation.enable_speed,
                disable_speed: style.button_animation.disable_speed,
            }),
            OptionRow | OptionEntry | OptionBox | OptionBoxLArrow | OptionBoxRArrow
            | OptionBoxChoice => Some(button_shift::Animation {
                rest_color: style.palette.red_dim,
                active_color: style.palette.hover(),
                active_right_shift: 0,
                enable_speed: style.button_animation.enable_speed,
                disable_speed: style.button_animation.disable_speed,
            }),
            TabButton => Some(button_shift::Animation {
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
            Self::Panel => {}
            Self::MainMenuItemButton => {
                let mut ui_image = ui_image.unwrap();
                let mut anim = anim.unwrap();
                ui_image.texture = style.images.item_button.clone_weak();
                *anim = self.shift_animation(style).unwrap();
            }
            Self::MainMenuItemText => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.main_menu.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.main_item_size);
                *anim = self.shift_animation(style).unwrap();
            }
            Self::TabText => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.tabline.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.main_item_size);
                *anim = self.shift_animation(style).unwrap();
            }
            Self::BackText => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.navigation.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.main_item_size);
                *anim = self.shift_animation(style).unwrap();
            }
            Self::TabButton | Self::OptionRow => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                text.sections[0].style.font = style.fonts.tabline.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.size);
                *anim = self.shift_animation(style).unwrap();
            }
            Self::OptionEntry => {
                let mut text = text.unwrap();
                let mut anim = anim.unwrap();
                let mut ui_image = ui_image.unwrap();
                text.sections[0].style.font = style.fonts.options.clone_weak();
                text.sections[0].style.font_size = f32::from(style.fonts.size);
                ui_image.texture = style.images.option_shadow.clone_weak();
                *anim = self.shift_animation(style).unwrap();
            }
            Self::SettingsHeader => {
                let mut bg = bg.unwrap();
                bg.0 = style.palette.settings_category_bg();
            }
            Self::SettingsHeaderText => {
                let mut text = text.unwrap();
                text.sections[0].style.font = style.fonts.item.clone_weak();
                text.sections[0].style.color = style.palette.settings_category_text();
                text.sections[0].style.font_size = f32::from(style.fonts.main_item_size);
            }
            Self::OptionBoxLArrow => {
                let mut ui_image = ui_image.unwrap();
                let mut anim = anim.unwrap();
                ui_image.texture = style.images.option_arrow.clone_weak();
                *anim = self.shift_animation(style).unwrap();
            }
            Self::OptionBoxRArrow => {
                let mut ui_image = ui_image.unwrap();
                let mut anim = anim.unwrap();
                ui_image.texture = style.images.option_arrow.clone_weak();
                ui_image.flip_x = true;
                *anim = self.shift_animation(style).unwrap();
            }
            Self::OptionBoxChoice => {
                let mut text = text.unwrap();
                text.sections[0].style.font = style.fonts.options.clone_weak();
                text.sections[0].style.color = style.palette.red_dim;
                text.sections[0].style.font_size = f32::from(style.fonts.size);
            }
            Self::OptionBox => {
                let mut ui_image = ui_image.unwrap();
                let mut bg = bg.unwrap();
                ui_image.texture = style.images.item_button.clone_weak();
                bg.0 = style.palette.red_dim;
            }
            Self::OptionTick => {
                let mut bg = bg.unwrap();
                bg.0 = style.palette.red_dim;
            }
        }
    }

    pub(crate) fn insert(self, cmds: &mut EntityCommands) {
        use Element::{BackText, OptionRow, OptionTick, SettingsHeader};
        use Element::{MainMenuItemButton, MainMenuItemText, TabButton, TabText};

        let ui_offset = UiOffset::default;
        let animation = button_shift::Animation::default;
        let state = button_shift::State::default;
        let shift = || (ui_offset(), animation(), state());
        let ui_image = UiImage::default;
        let text = || Text::from_section("", default());
        let bg = BgColor::default;
        match self.clone() {
            Self::Panel => {}
            BackText | MainMenuItemText | TabText | TabButton | OptionRow => {
                cmds.insert((text(), shift(), self));
            }
            Self::OptionEntry => {
                cmds.insert((ui_image(), text(), shift(), self));
            }
            SettingsHeader | OptionTick => {
                cmds.insert((bg(), self));
            }
            Self::OptionBox => {
                cmds.insert((bg(), ui_image(), self));
            }
            MainMenuItemButton | Self::OptionBoxRArrow | Self::OptionBoxLArrow => {
                cmds.insert((ui_image(), shift(), self));
            }
            Self::SettingsHeaderText | Self::OptionBoxChoice => {
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
