use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use fret_core::{ImageId, UvRect};

use crate::Renderer;
use crate::images::{ImageColorSpace, ImageDescriptor};
use crate::svg::{
    SMOOTH_SVG_SCALE_FACTOR, SvgAlphaMask, SvgRenderer, SvgRgbaImage, upload_alpha_mask,
    upload_rgba_image,
};
use fret_core::AlphaMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvgRasterKind {
    AlphaMask,
    Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SvgCacheKey {
    bytes_hash: u64,
    target_w: u32,
    target_h: u32,
    smooth_scale_bits: u32,
    kind: SvgRasterKind,
}

#[derive(Debug, Clone, Copy)]
pub struct CachedSvgImage {
    pub image: ImageId,
    pub uv: UvRect,
    pub size_px: (u32, u32),
}

struct SvgCacheEntry {
    image: ImageId,
    size_px: (u32, u32),
    _texture: wgpu::Texture,
}

/// App-owned cache for *manual* SVG rasterization results.
///
/// Prefer the newer `SvgId` + `SceneOp::SvgMaskIcon/SvgImage` pipeline when possible: it keeps SVG
/// rasters as renderer-internal resources (so the renderer can safely apply LRU/byte budgets).
///
/// This cache stores `ImageId`s that are visible to application code, so it intentionally avoids
/// automatic eviction: if other code holds `ImageId`s beyond the cache's awareness, evicting would
/// cause icons/images to disappear. Callers can explicitly `remove_*` or `clear`.
pub struct SvgImageCache {
    renderer: SvgRenderer,
    entries: HashMap<SvgCacheKey, SvgCacheEntry>,
}

impl Default for SvgImageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SvgImageCache {
    pub fn new() -> Self {
        Self {
            renderer: SvgRenderer::new(),
            entries: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self, renderer: &mut Renderer) {
        for e in self.entries.values() {
            let _ = renderer.unregister_image(e.image);
        }
        self.entries.clear();
    }

    /// Removes a cached entry if present.
    ///
    /// This mirrors GPUI's `PlatformAtlas::remove(key)` style: the cache is stable by default, and
    /// callers explicitly invalidate when assets change or when they want to reclaim memory.
    pub fn remove(
        &mut self,
        renderer: &mut Renderer,
        svg_bytes: &[u8],
        target_box_px: (u32, u32),
        kind: SvgRasterKind,
        smooth_scale_factor: f32,
    ) -> bool {
        let key = SvgCacheKey {
            bytes_hash: hash_bytes(svg_bytes),
            target_w: target_box_px.0,
            target_h: target_box_px.1,
            smooth_scale_bits: smooth_scale_factor.to_bits(),
            kind,
        };
        let Some(entry) = self.entries.remove(&key) else {
            return false;
        };
        let _ = renderer.unregister_image(entry.image);
        true
    }

    pub fn remove_alpha_mask(
        &mut self,
        renderer: &mut Renderer,
        svg_bytes: &[u8],
        target_box_px: (u32, u32),
    ) -> bool {
        self.remove(
            renderer,
            svg_bytes,
            target_box_px,
            SvgRasterKind::AlphaMask,
            SMOOTH_SVG_SCALE_FACTOR,
        )
    }

    pub fn remove_rgba(
        &mut self,
        renderer: &mut Renderer,
        svg_bytes: &[u8],
        target_box_px: (u32, u32),
    ) -> bool {
        self.remove(
            renderer,
            svg_bytes,
            target_box_px,
            SvgRasterKind::Rgba,
            SMOOTH_SVG_SCALE_FACTOR,
        )
    }

    pub fn get_or_create_alpha_mask(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut Renderer,
        svg_bytes: &[u8],
        target_box_px: (u32, u32),
    ) -> Result<CachedSvgImage, usvg::Error> {
        self.get_or_create_alpha_mask_fit(
            device,
            queue,
            renderer,
            svg_bytes,
            target_box_px,
            SMOOTH_SVG_SCALE_FACTOR,
        )
    }

    pub fn get_or_create_alpha_mask_fit(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut Renderer,
        svg_bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
    ) -> Result<CachedSvgImage, usvg::Error> {
        let key = SvgCacheKey {
            bytes_hash: hash_bytes(svg_bytes),
            target_w: target_box_px.0,
            target_h: target_box_px.1,
            smooth_scale_bits: smooth_scale_factor.to_bits(),
            kind: SvgRasterKind::AlphaMask,
        };

        if let Some(e) = self.entries.get(&key) {
            return Ok(CachedSvgImage {
                image: e.image,
                uv: full_uv(),
                size_px: e.size_px,
            });
        }

        let mask: SvgAlphaMask =
            self.renderer
                .render_alpha_mask_fit(svg_bytes, target_box_px, smooth_scale_factor)?;
        let uploaded = upload_alpha_mask(device, queue, &mask);
        let image = renderer.register_image(ImageDescriptor {
            view: uploaded.view.clone(),
            size: uploaded.size_px,
            format: wgpu::TextureFormat::R8Unorm,
            color_space: ImageColorSpace::Linear,
            alpha_mode: AlphaMode::Straight,
        });

        self.entries.insert(
            key,
            SvgCacheEntry {
                image,
                size_px: uploaded.size_px,
                _texture: uploaded.texture,
            },
        );

        Ok(CachedSvgImage {
            image,
            uv: full_uv(),
            size_px: uploaded.size_px,
        })
    }

    pub fn get_or_create_rgba(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut Renderer,
        svg_bytes: &[u8],
        target_box_px: (u32, u32),
    ) -> Result<CachedSvgImage, usvg::Error> {
        self.get_or_create_rgba_fit(
            device,
            queue,
            renderer,
            svg_bytes,
            target_box_px,
            SMOOTH_SVG_SCALE_FACTOR,
        )
    }

    pub fn get_or_create_rgba_fit(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut Renderer,
        svg_bytes: &[u8],
        target_box_px: (u32, u32),
        smooth_scale_factor: f32,
    ) -> Result<CachedSvgImage, usvg::Error> {
        let key = SvgCacheKey {
            bytes_hash: hash_bytes(svg_bytes),
            target_w: target_box_px.0,
            target_h: target_box_px.1,
            smooth_scale_bits: smooth_scale_factor.to_bits(),
            kind: SvgRasterKind::Rgba,
        };

        if let Some(e) = self.entries.get(&key) {
            return Ok(CachedSvgImage {
                image: e.image,
                uv: full_uv(),
                size_px: e.size_px,
            });
        }

        let rgba: SvgRgbaImage =
            self.renderer
                .render_rgba_fit(svg_bytes, target_box_px, smooth_scale_factor)?;
        let uploaded = upload_rgba_image(device, queue, &rgba);
        let image = renderer.register_image(ImageDescriptor {
            view: uploaded.view.clone(),
            size: uploaded.size_px,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            color_space: ImageColorSpace::Srgb,
            alpha_mode: AlphaMode::Straight,
        });

        self.entries.insert(
            key,
            SvgCacheEntry {
                image,
                size_px: uploaded.size_px,
                _texture: uploaded.texture,
            },
        );

        Ok(CachedSvgImage {
            image,
            uv: full_uv(),
            size_px: uploaded.size_px,
        })
    }
}

fn full_uv() -> UvRect {
    UvRect {
        u0: 0.0,
        v0: 0.0,
        u1: 1.0,
        v1: 1.0,
    }
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}
