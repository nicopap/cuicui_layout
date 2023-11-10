use std::marker::PhantomData;

use bevy::{prelude::*, reflect::GetTypeRegistration};

#[macro_export]
macro_rules! switchable_impl {
    ($($button:ident [$root:ident, $event:ident]),* $(,)?) => {
        $(
            #[derive(Event)]
            pub struct $event(pub u8);

            #[derive(Component, Default, Reflect)]
            #[reflect(Component)]
            pub struct $root;

            #[derive(Component, Default, Reflect)]
            #[reflect(Component)]
            pub struct $button(pub u8);

            impl $crate::Switchable for $button { type Event = $event; type Marker = $root; }
            $crate::switchable_impl! {@getindex $event, $button }
        )*
    };
    (@getindex $($impls:ident),* $(,)?) => {
        $(impl $crate::switch::GetIndex for $impls { fn index(&self) -> u8 { self.0 } })*
    };
}

pub trait GetIndex {
    fn index(&self) -> u8;
}
pub trait Switchable: Component + GetIndex + Reflect + GetTypeRegistration {
    type Event: Event + GetIndex;
    type Marker: Component + Reflect + GetTypeRegistration + Default;
}

fn switch_tab<T: Switchable>(
    mut tab_requests: EventReader<T::Event>,
    mut vis: Query<&mut Visibility>,
    tab_menu: Query<&Children, With<T::Marker>>,
) {
    // ANCHOR: system
    use Visibility::{Hidden, Inherited};

    for req in tab_requests.read() {
        let Ok(menu_children) = tab_menu.get_single() else {
            continue;
        };
        let mut i = 0;
        let mut iter = vis.iter_many_mut(menu_children);
        while let Some(mut vis) = iter.fetch_next() {
            *vis = if i == req.index() { Inherited } else { Hidden };
            i += 1;
        }
    }
    // ANCHOR_END: system
}

pub struct SwitchPlugin<T>(PhantomData<T>);

#[rustfmt::skip]
impl<T: Switchable> Default for SwitchPlugin<T> {
    fn default() -> Self { Self::new() }
}
#[rustfmt::skip]
impl<T: Switchable> SwitchPlugin<T> {
    #[must_use] pub fn new() -> Self { Self(PhantomData) }
}

impl<T: Switchable> Plugin for SwitchPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_event::<T::Event>()
            .register_type::<T::Marker>()
            .register_type::<T>()
            .add_systems(Update, switch_tab::<T>);
    }
}
