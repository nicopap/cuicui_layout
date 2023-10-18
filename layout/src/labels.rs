use bevy::prelude::SystemSet;

/// Mark [`compute_layout`] as added by [`Plugin`].
///
/// Consider using [`ComputeLayoutSet`] instead. `ComputeLayout` marks
/// `compute_layout` only, while `ComputeLayoutSet` also includes the
/// content-sized node's computation.
///
/// [`Plugin`]: crate::Plugin
/// [`compute_layout`]: crate::compute_layout
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub struct ComputeLayout;

/// [`compute_layout`] and content-sized systems.
///
/// This first runs the systems updating the size of content-dependent nodes
/// then run the global layouting system.
///
/// [`compute_layout`]: crate::compute_layout
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub struct ComputeLayoutSet;

/// Systems updating the [`ContentSized`] component.
///
/// It is part of [`ComputeLayoutSet`], but this happens just
/// before [`compute_layout`], setting the content-sized
/// informations.
///
/// [`ContentSized`]: crate::ContentSized
/// [`compute_layout`]: crate::compute_layout
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub struct ContentSizedSet;
