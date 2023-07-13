use std::fmt;
use std::marker::PhantomData;

use bevy::prelude::SystemSet;

/// The layouting systems, [`compute_layout`], just this system as added by [`Plugin`].
///
/// Consider using [`ComputeLayoutSet`] instead. `ComputeLayout` marks
/// `compute_layout` only, while `ComputeLayoutSet` also includes the
/// content-sized node's computation.
///
/// [`Plugin`]: crate::Plugin
/// [`compute_layout`]: crate::compute_layout
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub struct ComputeLayout;

/// The layouting systems, [`compute_layout`] and systems added by [`add_content_sized`].
///
/// [`add_content_sized`]: crate::AppContentSizeExt::add_content_sized
/// [`compute_layout`]: crate::compute_layout
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, SystemSet)]
pub struct ComputeLayoutSet;

/// When [`ComputeContentSize::compute_content`]  is evaulated.
/// [`add_content_sized`] automatically adds the relevant systems to this set.
///
/// It is part of [`ComputeLayoutSet`], but this happens just
/// before [`compute_layout`], setting the content-sized
/// informations.
///
/// [`add_content_sized`]: crate::AppContentSizeExt::add_content_sized
/// [`compute_layout`]: crate::compute_layout
/// [`ComputeContentSize::compute_content`]: crate::ComputeContentSize::compute_content
#[derive(SystemSet)]
pub struct ContentSizedComputeSystem<S>(PhantomData<fn(S)>);
impl<S> PartialEq for ContentSizedComputeSystem<S> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl<S> std::hash::Hash for ContentSizedComputeSystem<S> {
    fn hash<H>(&self, _: &mut H) {}
}
impl<S> fmt::Debug for ContentSizedComputeSystem<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(std::any::type_name::<Self>())
    }
}
impl<S> Eq for ContentSizedComputeSystem<S> {}
impl<S> Clone for ContentSizedComputeSystem<S> {
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl<S> Copy for ContentSizedComputeSystem<S> {}
impl<S> Default for ContentSizedComputeSystem<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}