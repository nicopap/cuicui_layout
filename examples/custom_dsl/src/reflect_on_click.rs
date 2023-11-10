use bevy::log::info;
use bevy::prelude::{Component, EventWriter, Reflect, ReflectComponent};
use bevy_mod_picking::prelude::{Click, On, Pointer};

use crate::dsl::{SwitchGraph, SwitchTab};

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
pub enum ReflectOnClick {
    LogInfo(String),
    EmitSwitchTab(u8),
    EmitSwitchGraph(u8),
    #[default]
    Invalid,
}
type OnClick = On<Pointer<Click>>;

impl<'a> From<&'a ReflectOnClick> for OnClick {
    fn from(value: &'a ReflectOnClick) -> Self {
        match value {
            ReflectOnClick::LogInfo(text) => {
                let text = text.clone();
                Self::run(move || info!("{text}"))
            }
            &ReflectOnClick::EmitSwitchTab(index) => {
                Self::run(move |mut ev: EventWriter<_>| ev.send(SwitchTab(index)))
            }
            &ReflectOnClick::EmitSwitchGraph(index) => {
                Self::run(move |mut ev: EventWriter<_>| ev.send(SwitchGraph(index)))
            }
            ReflectOnClick::Invalid => unreachable!("Should never spawn an invalid ReflectOnClick"),
        }
    }
}
