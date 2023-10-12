//! Trait to load assets from a `LoadContext`.
//!
//! Implemented for:
//! - `Image` (using `ImageTextureLoader`)
//! - `Chirp` (using `ChirpLoader`)
//! - `Font` (using `FontLoader`)

use std::path::Path;

use anyhow::Result;
use bevy::asset::LoadContext;
#[cfg(feature = "load_image")]
use bevy::render::texture::{CompressedImageFormats, Image, ImageType};
#[cfg(feature = "load_font")]
use bevy::text::Font;

use crate::Chirp;

/// Synchronously load file at `path` with provided `bytes` content.
///
/// Used in [`crate::parse_dsl::args`] to support [`bevy::asset::Handle`] in `.chirp` files.
///
/// Implementations are provided for:
/// - `bevy::render::texture::Image` with the `load_image` feature flag.
/// - `bevy::text::Font` with the `load_font` feature flag.
/// - `crate::Chirp`
pub trait LoadAsset: Sized {
    /// Synchronously load file at `path` with provided `bytes` content.
    #[allow(clippy::missing_errors_doc)] // False positive
    fn load(path: &Path, bytes: &[u8], load_context: &LoadContext) -> Result<Self>;
}
#[cfg(feature = "load_image")]
impl LoadAsset for Image {
    fn load(path: &Path, bytes: &[u8], _: &LoadContext) -> Result<Self> {
        // use the file extension for the image type
        let ext = path.extension().unwrap().to_str().unwrap();

        let image_type = ImageType::Extension(ext);
        let formats = CompressedImageFormats::empty();

        Ok(Self::from_buffer(bytes, image_type, formats, true)?)
    }
}
#[cfg(feature = "load_font")]
impl LoadAsset for Font {
    fn load(_: &Path, bytes: &[u8], _: &LoadContext) -> Result<Self> {
        Ok(Self::try_from_bytes(bytes.into())?)
    }
}
impl LoadAsset for Chirp {
    fn load(_: &Path, _: &[u8], _: &LoadContext) -> Result<Self> {
        todo!()
    }
}
