use fret_core::ImageId;
use fret_runtime::GlobalsHost;
use std::collections::HashMap;

/// Optional, policy-owned image metadata used by ecosystem components.
///
/// This is intentionally **not** a `UiServices` capability:
/// - the mechanism layer (`fret-ui`) must remain backend-agnostic, and
/// - ADR 0126 explicitly discourages implicit layout dependence on intrinsic image size.
///
/// Instead, apps/components that already know image dimensions (e.g. decoders, caches, streaming
/// sources) can record them here to enable ergonomic recipes like "aspect-ratio wrappers".
#[derive(Debug, Default, Clone)]
pub struct ImageMetadataStore {
    by_id: HashMap<ImageId, ImageMetadata>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ImageMetadata {
    pub intrinsic_size_px: Option<(u32, u32)>,
}

impl ImageMetadataStore {
    pub fn set_intrinsic_size_px(&mut self, image: ImageId, size_px: (u32, u32)) {
        self.by_id.entry(image).or_default().intrinsic_size_px = Some(size_px);
    }

    pub fn clear_intrinsic_size_px(&mut self, image: ImageId) {
        if let Some(entry) = self.by_id.get_mut(&image) {
            entry.intrinsic_size_px = None;
            if *entry == ImageMetadata::default() {
                self.by_id.remove(&image);
            }
        }
    }

    pub fn intrinsic_size_px(&self, image: ImageId) -> Option<(u32, u32)> {
        self.by_id.get(&image)?.intrinsic_size_px
    }

    pub fn aspect_ratio(&self, image: ImageId) -> Option<f32> {
        let (w, h) = self.intrinsic_size_px(image)?;
        if w == 0 || h == 0 {
            return None;
        }
        Some((w as f32) / (h as f32))
    }
}

/// Mutably accesses the global [`ImageMetadataStore`], creating it if needed.
///
/// This is a convenience wrapper around [`GlobalsHost::with_global_mut`], intended for apps and
/// policy layers that already know intrinsic image dimensions (e.g. decoders, caches, streaming
/// sources).
pub fn with_image_metadata_store_mut<H: GlobalsHost, R>(
    host: &mut H,
    f: impl FnOnce(&mut ImageMetadataStore) -> R,
) -> R {
    host.with_global_mut(ImageMetadataStore::default, |store, _host| f(store))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_metadata_store_reports_aspect_ratio() {
        let mut store = ImageMetadataStore::default();
        let img = ImageId::default();
        store.set_intrinsic_size_px(img, (1920, 1080));
        assert_eq!(store.aspect_ratio(img), Some(1920.0 / 1080.0));
    }

    #[test]
    fn image_metadata_store_ignores_zero_dimensions() {
        let mut store = ImageMetadataStore::default();
        let img = ImageId::default();
        store.set_intrinsic_size_px(img, (0, 1080));
        assert_eq!(store.aspect_ratio(img), None);
        store.set_intrinsic_size_px(img, (1920, 0));
        assert_eq!(store.aspect_ratio(img), None);
    }
}
