use std::fmt;

use bevy::ecs::query::Has;
use bevy::ecs::{prelude::*, system::SystemParam};
use bevy::prelude::{
    Children, ComputedVisibility, DebugName, GlobalTransform, Name, Parent, Visibility,
};

use super::scene::ChirpInstance;

type HierarchyQuery = (
    DebugName,
    Option<&'static Children>,
    Option<&'static Parent>,
    (Has<ComputedVisibility>, Has<GlobalTransform>),
    (Has<bevy::ui::Style>, Has<bevy::ui::Node>),
);
#[derive(SystemParam)]
pub struct PrintHierarchy<'w, 's> {
    query: Query<'w, 's, HierarchyQuery>,
}
impl<'w, 's> PrintHierarchy<'w, 's> {
    /// Get a [`PrintEntityHierarchy`], which `Debug` impl displays the bevy
    /// hierarchy with provided `root`.
    pub const fn print<'a>(&'a self, root: Entity) -> PrintEntityHierarchy<'a, 'w, 's> {
        PrintEntityHierarchy(root, self)
    }
}
/// Printable bevy hierarchy.
pub struct PrintEntityHierarchy<'a, 'w, 's>(Entity, &'a PrintHierarchy<'w, 's>);
impl fmt::Debug for PrintEntityHierarchy<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // unwrap: This query always is Ok for a living Entity
        let (n, children, _, vg, sn) = self.1.query.get(self.0).unwrap();
        let vis = if vg.0 { " vis" } else { "" };
        let tran = if vg.1 { " tran" } else { "" };
        let style = if sn.0 && sn.1 { " ui" } else { "" };

        let name = n
            .name
            .map_or_else(|| format!("Entity({:?})", n.entity), Name::to_string);
        // let parent = parent.map_or_else(String::new, |p| format!(" {:?}", p.get()));
        let name = format!("[{name:?}{vis}{tran}{style}]");
        if let Some(children) = children.filter(|c| !c.is_empty()) {
            let mut s = f.debug_tuple(&name);
            for &entry in children {
                s.field(&PrintEntityHierarchy(entry, self.1));
            }
            s.finish()
        } else {
            f.write_str(&name)
        }
    }
}
#[allow(clippy::needless_pass_by_value)] // false positive, bevy systems
pub(super) fn show_spawned(q: Query<Entity, Added<ChirpInstance>>, print: PrintHierarchy) {
    for e in &q {
        eprintln!("{:#?}", print.print(e));
    }
}
