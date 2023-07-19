#![allow(clippy::needless_pass_by_value)]

use bevy::ecs::{component::Tick, prelude::*, system::SystemChangeTick};
use bevy::prelude::{debug, Children, Name, Parent};
#[cfg(feature = "reflect")]
use bevy::prelude::{Reflect, ReflectComponent};
use bevy_mod_sysfail::sysfail;

use crate::{
    error::Computed, layout::Layout, layout::NodeQuery, ComputeLayoutError, LayoutRect, Node, Root,
    Size,
};

/// A [`Node`] that can't have children.
#[derive(Component, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
pub struct LeafNode;

/// Use this camera's logical size as the root container size.
///
/// Note that it is an error to have more than a single camera with this
/// component.
#[derive(Component, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
pub struct LayoutRootCamera;

/// Set this [`Root`] to track the [`LayoutRootCamera`]'s size.
#[derive(Component, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
pub struct ScreenRoot;

/// Stores the tick of the last time [`compute_layout`] ran.
#[derive(Resource, Default)]
pub struct LastLayoutChange {
    tick: Option<Tick>,
}
impl LastLayoutChange {
    /// The last time [`compute_layout`] ran.
    #[must_use]
    pub const fn tick(&self) -> Option<Tick> {
        self.tick
    }
}

type LayoutRef = (
    Option<Ref<'static, Node>>,
    Option<Ref<'static, Root>>,
    Option<Ref<'static, Children>>,
    Option<Ref<'static, Parent>>,
);

/// A run condition to tell whether it's necessary to recompute layout.
#[allow(clippy::needless_pass_by_value, clippy::must_use_candidate)]
pub fn require_layout_recompute(
    nodes: Query<NodeQuery>,
    anything_changed: Query<LayoutRef, Or<(With<Node>, With<Root>)>>,
    last_layout_change: Res<LastLayoutChange>,
    system_tick: SystemChangeTick,
    mut children_removed: RemovedComponents<Children>,
    mut parent_removed: RemovedComponents<Parent>,
) -> bool {
    let Some(tick) = last_layout_change.tick else {
        return true;
    };
    let this_tick = system_tick.this_run();
    let anything_changed = anything_changed.iter().any(|q| {
        matches!(q.0, Some(r) if r.last_changed().is_newer_than(tick, this_tick))
            || matches!(q.1, Some(r) if r.last_changed().is_newer_than(tick, this_tick))
            || matches!(q.2, Some(r) if r.last_changed().is_newer_than(tick, this_tick))
            || matches!(q.3, Some(r) if r.last_changed().is_newer_than(tick, this_tick))
    });
    let mut children_removed = || children_removed.iter().any(|e| nodes.contains(e));
    let mut parent_removed = || parent_removed.iter().any(|e| nodes.contains(e));

    anything_changed || children_removed() || parent_removed()
}

/// Run the layout algorithm.
#[sysfail(log(level = "error"))]
pub fn compute_layout(
    mut to_update: Query<&'static mut LayoutRect>,
    nodes: Query<NodeQuery>,
    names: Query<&'static Name>,
    roots: Query<(Entity, &'static Root, &'static Children)>,
    mut last_layout_change: ResMut<LastLayoutChange>,
    system_tick: SystemChangeTick,
) -> Result<(), ComputeLayoutError> {
    debug!("Computing layout");
    last_layout_change.tick = Some(system_tick.this_run());
    for (entity, root, children) in &roots {
        let root_container = *root.get();
        let bounds = root.get_size(entity, &names)?;
        if let Ok(mut to_update) = to_update.get_mut(entity) {
            to_update.size = bounds;
        }
        let mut layout = Layout::new(entity, &mut to_update, &nodes, &names);
        let mut bounds: Size<Computed> = bounds.into();
        bounds.set_margin(root_container.margin, &layout)?;
        layout.container(root_container, children, bounds)?;
    }
    Ok(())
}

/// Whether a [`apply_deferred`] needs to run after the last run of [`update_leaf_nodes`].
///
/// [`apply_deferred`]: bevy::prelude::apply_deferred
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
pub struct LeafNodeInsertWitness {
    needs_apply: bool,
}

impl LeafNodeInsertWitness {
    /// Create a new [`LeafNodeInsertWitness`].
    #[must_use]
    pub const fn new(needs_apply: bool) -> Self {
        Self { needs_apply }
    }
}

/// Add/remove [`LeafNode`] component according to the current [`Node`] state.
///
/// Note that the change won't be visible untill the next flush.
pub fn update_leaf_nodes(
    mut leaf_nodes: ResMut<LeafNodeInsertWitness>,
    mut cmds: Commands,
    was_leaf_node: Query<(Entity, &Node), (Changed<Node>, With<LeafNode>)>,
    wasnt_leaf_node: Query<(Entity, &Node), (Changed<Node>, Without<LeafNode>)>,
    no_node: Query<Entity, (Without<Node>, With<LeafNode>)>,
) {
    leaf_nodes.needs_apply = false;
    for entity in &no_node {
        cmds.entity(entity).remove::<LeafNode>();
    }
    for (entity, node) in &was_leaf_node {
        if matches!(node, Node::Container(_)) {
            leaf_nodes.needs_apply = true;
            cmds.entity(entity).remove::<LeafNode>();
        }
    }
    for (entity, node) in &wasnt_leaf_node {
        if matches!(node, Node::Axis(_) | Node::Box(_)) {
            leaf_nodes.needs_apply = true;
            cmds.entity(entity).insert(LeafNode);
        }
    }
}
