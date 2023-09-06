//! Remove components by id, storing them in an erased bundle type

use std::{iter, ptr};

use bevy::{
    ecs::{
        bundle::DynamicBundle,
        component::{ComponentId, ComponentInfo, StorageType},
        storage::Table,
    },
    prelude::{Bundle, Entity, World},
    ptr::{OwningPtr, Ptr},
    reflect::TypeRegistryInternal as TypeRegistry,
};

struct MovedErasedBundle {
    // NOTE: memory is unaligned here.
    data: Box<[u8]>,
    components: Box<[(usize, ComponentId)]>,
    consumed: bool,
}
struct MovedErasedBundleBuilder {
    // NOTE: memory is unaligned here.
    data: Vec<u8>,
    // (offset_in_data, id)
    components: Vec<(usize, ComponentId)>,
}
impl MovedErasedBundleBuilder {
    fn new() -> Self {
        Self { data: Vec::new(), components: Vec::new() }
    }
    fn build(self) -> MovedErasedBundle {
        MovedErasedBundle {
            data: self.data.into(),
            components: self.components.into(),
            consumed: false,
        }
    }
    fn push(&mut self, info: &ComponentInfo, data: Ptr<Aligned>) {
        let id = info.id();
        let offset_in_data = self.data.len();
        let size = info.layout().size();
        self.data.extend(iter::repeat(0).take(size));
        let to_write = &mut self.data[offset_in_data..offset_in_data + size];
        let to_write = to_write.as_mut_ptr();
        // SAFETY:
        // - `u8` always aligned
        // - we just allocated `size` new bytes at this exact memory location
        // - `self.data` is guarenteed to be distinct from `data`
        unsafe { ptr::copy_nonoverlapping(data.as_ptr(), to_write, size) };
        self.components.push((offset_in_data, id));
    }
}
fn take_by_ids(
    world: &mut World,
    entity: Entity,
    ids: &[ComponentId],
) -> Option<MovedErasedBundle> {
    let mut builder = MovedErasedBundleBuilder::new();
    for &id in ids {
        let info = world.components().get_info(id)?;
        let erased_component = world.get_by_id(entity, id)?;
        builder.push(info, erased_component);
    }
    remove_by_ids(world, entity, ids);
    Some(builder.build())
}
fn remove_by_ids(world: &mut World, entity: Entity, ids: &[ComponentId]) {}
unsafe impl Bundle for MovedErasedBundle {
    fn component_ids(
        components: &mut bevy::ecs::component::Components,
        storages: &mut bevy::ecs::storage::Storages,
        ids: &mut impl FnMut(ComponentId),
    ) {
        todo!()
    }

    unsafe fn from_components<T, F>(ctx: &mut T, func: &mut F) -> Self
    where
        // Ensure that the `OwningPtr` is used correctly
        F: for<'a> FnMut(&'a mut T) -> OwningPtr<'a>,
        Self: Sized,
    {
        todo!()
    }
}

impl DynamicBundle for MovedErasedBundle {
    fn get_components(self, func: &mut impl FnMut(StorageType, OwningPtr<'_>)) {
        for (offset, id) in self.0.iter() {
            // SAFETY: This is unsafe, yet sound right now:
            // this relies on `func` not calling any unsafe methods of OnwingPtr,
            // which is currently the case in bevy. It only calls `as_ptr`
            let owning = unsafe { ptr.assert_unique().promote() };
            func(StorageType::Table, owning);
        }
    }
}
