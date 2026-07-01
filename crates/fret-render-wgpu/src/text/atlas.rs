use fret_core::RendererGlyphAtlasPerfSnapshot;
use fret_render_text::FontFaceKey;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[cfg(not(target_arch = "wasm32"))]
mod debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum GlyphQuadKind {
    Mask,
    Color,
    Subpixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct GlyphKey {
    pub(super) font: FontFaceKey,
    pub(super) glyph_id: u32,
    pub(super) size_bits: u32,
    pub(super) x_bin: u8,
    pub(super) y_bin: u8,
    kind: GlyphQuadKind,
}

const GLYPH_KEY_LOOKUP_KIND_ORDER: [GlyphQuadKind; 3] = [
    GlyphQuadKind::Color,
    GlyphQuadKind::Subpixel,
    GlyphQuadKind::Mask,
];

impl GlyphKey {
    fn new(
        font: FontFaceKey,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        kind: GlyphQuadKind,
    ) -> Self {
        Self {
            font,
            glyph_id,
            size_bits,
            x_bin,
            y_bin,
            kind,
        }
    }

    pub(super) fn from_image_content(
        font: FontFaceKey,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        content: parley::swash::scale::image::Content,
    ) -> (Self, u32) {
        let (kind, bytes_per_pixel) = glyph_image_content_metadata(content);
        (
            Self::new(font, glyph_id, size_bits, x_bin, y_bin, kind),
            bytes_per_pixel,
        )
    }

    #[cfg(test)]
    pub(super) fn lookup_keys(
        font: FontFaceKey,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
    ) -> [Self; 3] {
        GLYPH_KEY_LOOKUP_KIND_ORDER
            .map(|kind| Self::new(font, glyph_id, size_bits, x_bin, y_bin, kind))
    }

    pub(super) fn lookup_keys_with_hint(
        font: FontFaceKey,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        hint: Option<GlyphQuadKind>,
    ) -> [Self; 3] {
        glyph_key_lookup_kind_order_with_hint(hint)
            .map(|kind| Self::new(font, glyph_id, size_bits, x_bin, y_bin, kind))
    }

    pub(super) fn quad_kind(self) -> GlyphQuadKind {
        self.kind
    }

    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(super) fn is_mask(self) -> bool {
        matches!(self.kind, GlyphQuadKind::Mask)
    }

    pub(super) fn is_color(self) -> bool {
        matches!(self.kind, GlyphQuadKind::Color)
    }

    pub(super) fn is_subpixel(self) -> bool {
        matches!(self.kind, GlyphQuadKind::Subpixel)
    }

    pub(super) fn bytes_per_pixel_for_image_content(
        self,
        content: parley::swash::scale::image::Content,
    ) -> Option<u32> {
        let (kind, bytes_per_pixel) = glyph_image_content_metadata(content);
        (kind == self.kind).then_some(bytes_per_pixel)
    }
}

#[derive(Default)]
pub(super) struct GlyphKeyBuckets {
    mask: HashSet<GlyphKey>,
    color: HashSet<GlyphKey>,
    subpixel: HashSet<GlyphKey>,
}

impl GlyphKeyBuckets {
    #[cfg(test)]
    pub(super) fn with_capacities(mask: usize, color: usize, subpixel: usize) -> Self {
        Self {
            mask: HashSet::with_capacity(mask),
            color: HashSet::with_capacity(color),
            subpixel: HashSet::with_capacity(subpixel),
        }
    }

    pub(super) fn insert(&mut self, key: GlyphKey) {
        if key.is_color() {
            self.color.insert(key);
        } else if key.is_subpixel() {
            self.subpixel.insert(key);
        } else {
            self.mask.insert(key);
        }
    }

    #[cfg(test)]
    pub(super) fn extend_pin_keys(&mut self, keys: &GlyphPinKeys) {
        self.mask.extend(keys.mask.iter().copied());
        self.color.extend(keys.color.iter().copied());
        self.subpixel.extend(keys.subpixel.iter().copied());
    }

    pub(super) fn into_pin_bucket(self) -> (Vec<GlyphKey>, Vec<GlyphKey>, Vec<GlyphKey>) {
        (
            self.mask.into_iter().collect(),
            self.color.into_iter().collect(),
            self.subpixel.into_iter().collect(),
        )
    }
}

pub(super) struct GlyphPinBucketDelta {
    pub(super) retained_len: usize,
    pub(super) added: (Vec<GlyphKey>, Vec<GlyphKey>, Vec<GlyphKey>),
    pub(super) removed: (Vec<GlyphKey>, Vec<GlyphKey>, Vec<GlyphKey>),
}

#[derive(Debug, Clone, Default)]
pub(super) struct GlyphPinKeys {
    mask: Arc<[GlyphKey]>,
    color: Arc<[GlyphKey]>,
    subpixel: Arc<[GlyphKey]>,
}

impl GlyphPinKeys {
    pub(super) fn from_keys(keys: impl IntoIterator<Item = GlyphKey>) -> Self {
        let mut buckets = GlyphKeyBuckets::default();
        for key in keys {
            buckets.insert(key);
        }
        let (mask, color, subpixel) = buckets.into_pin_bucket();
        Self {
            mask: Arc::from(mask),
            color: Arc::from(color),
            subpixel: Arc::from(subpixel),
        }
    }

    pub(super) fn heap_bytes_estimate(&self) -> u64 {
        let bytes = (self.mask.len() as u128)
            .saturating_add(self.color.len() as u128)
            .saturating_add(self.subpixel.len() as u128)
            .saturating_mul(std::mem::size_of::<GlyphKey>() as u128);
        bytes.min(u64::MAX as u128) as u64
    }

    pub(super) fn mask_keys(&self) -> &[GlyphKey] {
        &self.mask
    }

    pub(super) fn color_keys(&self) -> &[GlyphKey] {
        &self.color
    }

    pub(super) fn subpixel_keys(&self) -> &[GlyphKey] {
        &self.subpixel
    }
}

fn glyph_image_content_metadata(
    content: parley::swash::scale::image::Content,
) -> (GlyphQuadKind, u32) {
    match content {
        parley::swash::scale::image::Content::Mask => (GlyphQuadKind::Mask, 1),
        parley::swash::scale::image::Content::Color => (GlyphQuadKind::Color, 4),
        parley::swash::scale::image::Content::SubpixelMask => (GlyphQuadKind::Subpixel, 4),
    }
}

fn glyph_key_lookup_kind_order_with_hint(hint: Option<GlyphQuadKind>) -> [GlyphQuadKind; 3] {
    let Some(hint) = hint else {
        return GLYPH_KEY_LOOKUP_KIND_ORDER;
    };

    match hint {
        GlyphQuadKind::Color => [
            GlyphQuadKind::Color,
            GlyphQuadKind::Subpixel,
            GlyphQuadKind::Mask,
        ],
        GlyphQuadKind::Subpixel => [
            GlyphQuadKind::Subpixel,
            GlyphQuadKind::Color,
            GlyphQuadKind::Mask,
        ],
        GlyphQuadKind::Mask => [
            GlyphQuadKind::Mask,
            GlyphQuadKind::Color,
            GlyphQuadKind::Subpixel,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face() -> FontFaceKey {
        FontFaceKey::new(1, 0, 0, false, 0)
    }

    fn alloc_id(id: u32) -> etagere::AllocId {
        etagere::AllocId::deserialize(id)
    }

    fn test_atlas_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("test glyph atlas bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        })
    }

    fn test_atlas_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("test glyph atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        })
    }

    fn test_glyph_atlas(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        max_pages: usize,
    ) -> GlyphAtlas {
        let layout = test_atlas_bind_group_layout(device);
        let sampler = test_atlas_sampler(device);
        GlyphAtlas::new(
            device,
            &layout,
            &sampler,
            "test glyph atlas",
            width,
            height,
            wgpu::TextureFormat::R8Unorm,
            0,
            max_pages,
        )
    }

    fn mask_key(glyph_id: u32) -> GlyphKey {
        let [_color, _subpixel, mask] =
            GlyphKey::lookup_keys(face(), glyph_id, 16.0f32.to_bits(), 0, 0);
        mask
    }

    #[test]
    fn glyph_pin_keys_deduplicate_by_bucket() {
        let [color, subpixel, mask] = GlyphKey::lookup_keys(face(), 42, 16.0f32.to_bits(), 0, 0);

        let keys = GlyphPinKeys::from_keys([mask, color, mask, subpixel, color]);

        assert_eq!(keys.mask.as_ref(), &[mask]);
        assert_eq!(keys.color.as_ref(), &[color]);
        assert_eq!(keys.subpixel.as_ref(), &[subpixel]);

        let mut buckets = GlyphKeyBuckets::default();
        buckets.extend_pin_keys(&keys);
        buckets.extend_pin_keys(&keys);
        let (mask, color, subpixel) = buckets.into_pin_bucket();

        assert_eq!(mask.len(), 1);
        assert_eq!(color.len(), 1);
        assert_eq!(subpixel.len(), 1);
    }

    #[test]
    fn glyph_key_buckets_with_capacities_deduplicate_by_bucket() {
        let [color, subpixel, mask] = GlyphKey::lookup_keys(face(), 42, 16.0f32.to_bits(), 0, 0);

        let mut buckets = GlyphKeyBuckets::with_capacities(8, 8, 8);
        buckets.insert(mask);
        buckets.insert(mask);
        buckets.insert(color);
        buckets.insert(color);
        buckets.insert(subpixel);
        buckets.insert(subpixel);
        let (mask, color, subpixel) = buckets.into_pin_bucket();

        assert_eq!(mask.len(), 1);
        assert_eq!(color.len(), 1);
        assert_eq!(subpixel.len(), 1);
    }

    #[test]
    fn glyph_lookup_kind_hint_only_reorders_full_fallback_set() {
        let fallback = glyph_key_lookup_kind_order_with_hint(None);
        assert_eq!(fallback, GLYPH_KEY_LOOKUP_KIND_ORDER);

        for hint in [
            GlyphQuadKind::Mask,
            GlyphQuadKind::Color,
            GlyphQuadKind::Subpixel,
        ] {
            let order = glyph_key_lookup_kind_order_with_hint(Some(hint));
            assert_eq!(order[0], hint);
            for kind in [
                GlyphQuadKind::Mask,
                GlyphQuadKind::Color,
                GlyphQuadKind::Subpixel,
            ] {
                assert!(
                    order.contains(&kind),
                    "lookup hint must not remove the {kind:?} fallback"
                );
            }
        }
    }

    #[test]
    fn glyph_atlas_diagnostics_report_page_budget_separately_from_live_pages() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut atlas = test_glyph_atlas(&ctx.device, 16, 16, 3);

        let cold = atlas.diagnostics_snapshot();
        assert_eq!(cold.pages, 0);
        assert_eq!(cold.max_pages, 3);
        assert_eq!(cold.capacity_px, 0);

        let key = mask_key(7);
        atlas
            .get_or_insert(key, 10, 10, 0, 0, 1, vec![255; 100], 1)
            .expect("glyph insert");

        let snap = atlas.diagnostics_snapshot();
        assert_eq!(snap.pages, 1);
        assert_eq!(snap.max_pages, 3);
        assert_eq!(snap.capacity_px, 16 * 16);
        assert_eq!(snap.entries, 1);
    }

    #[test]
    fn glyph_atlas_page_budget_blocks_growth_when_live_page_is_full() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut atlas = test_glyph_atlas(&ctx.device, 16, 16, 1);
        let a = mask_key(11);
        let b = mask_key(12);

        atlas
            .get_or_insert(a, 12, 12, 0, 0, 1, vec![1; 144], 1)
            .expect("first glyph insert");
        atlas.inc_live_refs(&[a]);

        let err = atlas
            .get_or_insert(b, 12, 12, 0, 0, 1, vec![2; 144], 2)
            .expect_err("live full page should not grow beyond the configured budget");

        let snap = atlas.diagnostics_snapshot();
        assert_eq!(err, GlyphAtlasInsertError::OutOfSpace);
        assert_eq!(snap.pages, 1);
        assert_eq!(snap.max_pages, 1);
        assert_eq!(snap.entries, 1);
        assert_eq!(snap.frame_out_of_space, 1);
    }

    #[test]
    fn pending_texture_uploads_merge_contiguous_padded_slots() {
        let uploads = prepare_texture_uploads(vec![
            PendingUpload {
                alloc_id: alloc_id(1),
                x: 1,
                y: 1,
                w: 2,
                h: 2,
                padded_x: 0,
                padded_y: 0,
                padded_w: 4,
                padded_h: 4,
                bytes_per_pixel: 1,
                data: vec![1, 2, 3, 4],
            },
            PendingUpload {
                alloc_id: alloc_id(2),
                x: 5,
                y: 1,
                w: 2,
                h: 2,
                padded_x: 4,
                padded_y: 0,
                padded_w: 4,
                padded_h: 4,
                bytes_per_pixel: 1,
                data: vec![5, 6, 7, 8],
            },
        ]);

        assert_eq!(uploads.len(), 1);
        let upload = &uploads[0];
        assert_eq!(upload.origin.x, 0);
        assert_eq!(upload.origin.y, 0);
        assert_eq!(upload.extent.width, 8);
        assert_eq!(upload.extent.height, 4);
        assert_eq!(upload.bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

        let stride = upload.bytes_per_row as usize;
        assert_eq!(&upload.bytes[stride + 1..stride + 3], &[1, 2]);
        assert_eq!(&upload.bytes[stride + 5..stride + 7], &[5, 6]);
        assert_eq!(&upload.bytes[stride * 2 + 1..stride * 2 + 3], &[3, 4]);
        assert_eq!(&upload.bytes[stride * 2 + 5..stride * 2 + 7], &[7, 8]);
    }

    #[test]
    fn pending_texture_uploads_keep_distinct_padded_rows_separate() {
        let uploads = prepare_texture_uploads(vec![
            PendingUpload {
                alloc_id: alloc_id(1),
                x: 1,
                y: 1,
                w: 2,
                h: 2,
                padded_x: 0,
                padded_y: 0,
                padded_w: 4,
                padded_h: 4,
                bytes_per_pixel: 1,
                data: vec![1, 2, 3, 4],
            },
            PendingUpload {
                alloc_id: alloc_id(2),
                x: 1,
                y: 5,
                w: 2,
                h: 2,
                padded_x: 0,
                padded_y: 4,
                padded_w: 4,
                padded_h: 4,
                bytes_per_pixel: 1,
                data: vec![5, 6, 7, 8],
            },
        ]);

        assert_eq!(uploads.len(), 2);
        assert_eq!(uploads[0].origin.y, 1);
        assert_eq!(uploads[1].origin.y, 5);
    }

    #[test]
    fn pending_texture_uploads_remove_evicted_allocation_before_merge() {
        let evicted = alloc_id(1);
        let mut pending = vec![
            PendingUpload {
                alloc_id: evicted,
                x: 1,
                y: 1,
                w: 2,
                h: 2,
                padded_x: 0,
                padded_y: 0,
                padded_w: 4,
                padded_h: 4,
                bytes_per_pixel: 1,
                data: vec![9, 9, 9, 9],
            },
            PendingUpload {
                alloc_id: alloc_id(2),
                x: 1,
                y: 1,
                w: 2,
                h: 2,
                padded_x: 0,
                padded_y: 0,
                padded_w: 4,
                padded_h: 4,
                bytes_per_pixel: 1,
                data: vec![1, 2, 3, 4],
            },
            PendingUpload {
                alloc_id: alloc_id(3),
                x: 5,
                y: 1,
                w: 2,
                h: 2,
                padded_x: 4,
                padded_y: 0,
                padded_w: 4,
                padded_h: 4,
                bytes_per_pixel: 1,
                data: vec![5, 6, 7, 8],
            },
        ];

        assert!(remove_pending_upload_for_alloc(&mut pending, evicted));
        let uploads = prepare_texture_uploads(pending);

        assert_eq!(uploads.len(), 1);
        let upload = &uploads[0];
        let stride = upload.bytes_per_row as usize;
        assert_eq!(&upload.bytes[stride + 1..stride + 3], &[1, 2]);
        assert_eq!(&upload.bytes[stride * 2 + 1..stride * 2 + 3], &[3, 4]);
        assert!(
            !upload.bytes.contains(&9),
            "evicted pending upload data must not be merged into a later atlas write"
        );
    }
}

pub(super) fn subpixel_bin_q4(pos: f32) -> (i32, u8) {
    // Keep behavior aligned with the legacy 4-way subpixel binning policy.
    let trunc = pos as i32;
    let fract = pos - trunc as f32;

    if pos.is_sign_negative() {
        if fract > -0.125 {
            (trunc, 0)
        } else if fract > -0.375 {
            (trunc - 1, 3)
        } else if fract > -0.625 {
            (trunc - 1, 2)
        } else if fract > -0.875 {
            (trunc - 1, 1)
        } else {
            (trunc - 1, 0)
        }
    } else {
        #[allow(clippy::collapsible_else_if)]
        if fract < 0.125 {
            (trunc, 0)
        } else if fract < 0.375 {
            (trunc, 1)
        } else if fract < 0.625 {
            (trunc, 2)
        } else if fract < 0.875 {
            (trunc, 3)
        } else {
            (trunc + 1, 0)
        }
    }
}

pub(super) fn subpixel_bin_as_float(bin: u8) -> f32 {
    match bin {
        0 => 0.0,
        1 => 0.25,
        2 => 0.5,
        3 => 0.75,
        _ => 0.0,
    }
}

const SUBPIXEL_VARIANTS_X: u8 = 4;
const SUBPIXEL_VARIANTS_Y: u8 = if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
    1
} else {
    SUBPIXEL_VARIANTS_X
};

pub(super) fn subpixel_bin_y(pos: f32) -> (i32, u8) {
    let (y, bin) = subpixel_bin_q4(pos);
    if SUBPIXEL_VARIANTS_Y <= 1 {
        (y, 0)
    } else {
        (y, bin)
    }
}

const GLYPH_ATLAS_INSERT_GUARD_LIMIT: u32 = 128;

fn atlas_allocator_size(width: u32, height: u32) -> etagere::Size {
    etagere::Size::new(width as i32, height as i32)
}

#[derive(Debug, Default, Clone, Copy)]
struct GlyphAtlasFramePerf {
    hits: u64,
    misses: u64,
    inserts: u64,
    evict_glyphs: u64,
    evict_pages: u64,
    out_of_space: u64,
    too_large: u64,
    pending_uploads: u64,
    pending_upload_bytes: u64,
    upload_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GlyphAtlasEntry {
    pub(super) page: u16,
    alloc_id: etagere::AllocId,
    pub(super) x: u32,
    pub(super) y: u32,
    pub(super) w: u32,
    pub(super) h: u32,
    pub(super) placement_left: i32,
    pub(super) placement_top: i32,
    live_refs: u32,
    last_used_epoch: u64,
}

#[derive(Debug)]
struct PendingUpload {
    alloc_id: etagere::AllocId,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    padded_x: u32,
    padded_y: u32,
    padded_w: u32,
    padded_h: u32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
}

#[derive(Debug)]
struct PreparedTextureUpload {
    origin: wgpu::Origin3d,
    extent: wgpu::Extent3d,
    bytes_per_row: u32,
    rows_per_image: u32,
    bytes: Vec<u8>,
}

impl PendingUpload {
    fn into_texture_upload(self) -> Option<PreparedTextureUpload> {
        if self.w == 0 || self.h == 0 {
            return None;
        }

        let bytes_per_row = self.w.saturating_mul(self.bytes_per_pixel);
        if bytes_per_row == 0 {
            return None;
        }

        let expected_len = (bytes_per_row as usize).saturating_mul(self.h as usize);
        debug_assert_eq!(self.data.len(), expected_len);
        if self.data.len() != expected_len {
            return None;
        }

        let aligned_bytes_per_row = align_upload_bytes_per_row(bytes_per_row);
        let bytes = if aligned_bytes_per_row == bytes_per_row {
            self.data
        } else {
            align_upload_bytes(&self.data, bytes_per_row, aligned_bytes_per_row, self.h)
        };

        Some(PreparedTextureUpload {
            origin: wgpu::Origin3d {
                x: self.x,
                y: self.y,
                z: 0,
            },
            extent: wgpu::Extent3d {
                width: self.w,
                height: self.h,
                depth_or_array_layers: 1,
            },
            bytes_per_row: aligned_bytes_per_row,
            rows_per_image: self.h,
            bytes,
        })
    }
}

fn remove_pending_upload_for_alloc(
    pending: &mut Vec<PendingUpload>,
    alloc_id: etagere::AllocId,
) -> bool {
    let before = pending.len();
    pending.retain(|upload| upload.alloc_id != alloc_id);
    pending.len() != before
}

fn pending_upload_data_len(upload: &PendingUpload) -> Option<usize> {
    let bytes_per_row = upload.w.checked_mul(upload.bytes_per_pixel)?;
    let len = (bytes_per_row as usize).checked_mul(upload.h as usize)?;
    (len == upload.data.len()).then_some(len)
}

fn pending_upload_can_merge_with_run(run: &[PendingUpload], upload: &PendingUpload) -> bool {
    let Some(last) = run.last() else {
        return false;
    };

    last.bytes_per_pixel == upload.bytes_per_pixel
        && last.padded_y == upload.padded_y
        && last.padded_h == upload.padded_h
        && run
            .last()
            .is_some_and(|last| last.padded_x.saturating_add(last.padded_w) == upload.padded_x)
}

fn prepare_texture_uploads(mut pending: Vec<PendingUpload>) -> Vec<PreparedTextureUpload> {
    if pending.is_empty() {
        return Vec::new();
    }

    // Each pending upload belongs to a freshly allocated glyph slot in a single atlas page. Merging
    // only contiguous padded slots keeps writes inside newly owned texture regions.
    pending.retain(|upload| pending_upload_data_len(upload).is_some());
    pending.sort_by_key(|upload| {
        (
            upload.bytes_per_pixel,
            upload.padded_y,
            upload.padded_h,
            upload.padded_x,
            upload.padded_w,
            upload.y,
            upload.x,
        )
    });

    let mut out = Vec::with_capacity(pending.len());
    let mut run: Vec<PendingUpload> = Vec::new();
    for upload in pending {
        if !pending_upload_can_merge_with_run(&run, &upload) {
            flush_pending_upload_run(&mut out, &mut run);
        }
        run.push(upload);
    }
    flush_pending_upload_run(&mut out, &mut run);
    out
}

fn flush_pending_upload_run(out: &mut Vec<PreparedTextureUpload>, run: &mut Vec<PendingUpload>) {
    if run.is_empty() {
        return;
    }

    if run.len() == 1 {
        if let Some(upload) = run.pop().and_then(PendingUpload::into_texture_upload) {
            out.push(upload);
        }
        return;
    }

    let Some(upload) = merged_pending_upload(run.as_slice()) else {
        for pending in run.drain(..) {
            if let Some(upload) = pending.into_texture_upload() {
                out.push(upload);
            }
        }
        return;
    };
    run.clear();
    out.push(upload);
}

fn merged_pending_upload(run: &[PendingUpload]) -> Option<PreparedTextureUpload> {
    let first = run.first()?;
    let run_x = first.padded_x;
    let run_y = first.padded_y;
    let run_h = first.padded_h;
    let bytes_per_pixel = first.bytes_per_pixel;
    let run_end_x = run.last()?.padded_x.checked_add(run.last()?.padded_w)?;
    let run_w = run_end_x.checked_sub(run_x)?;
    let bytes_per_row = run_w.checked_mul(bytes_per_pixel)?;
    let len = (bytes_per_row as usize).checked_mul(run_h as usize)?;
    let mut data = vec![0; len];

    for upload in run {
        if upload.bytes_per_pixel != bytes_per_pixel
            || upload.padded_y != run_y
            || upload.padded_h != run_h
            || upload.x < run_x
            || upload.y < run_y
            || upload.x.checked_add(upload.w)? > run_end_x
            || upload.y.checked_add(upload.h)? > run_y.checked_add(run_h)?
        {
            return None;
        }

        let src_row_bytes = upload.w.checked_mul(bytes_per_pixel)? as usize;
        let dst_x_bytes = upload.x.checked_sub(run_x)?.checked_mul(bytes_per_pixel)? as usize;
        let dst_y = upload.y.checked_sub(run_y)? as usize;
        let dst_row_bytes = bytes_per_row as usize;
        for row in 0..upload.h as usize {
            let src0 = row.checked_mul(src_row_bytes)?;
            let src1 = src0.checked_add(src_row_bytes)?;
            let dst0 = dst_y
                .checked_add(row)?
                .checked_mul(dst_row_bytes)?
                .checked_add(dst_x_bytes)?;
            let dst1 = dst0.checked_add(src_row_bytes)?;
            data.get_mut(dst0..dst1)?
                .copy_from_slice(upload.data.get(src0..src1)?);
        }
    }

    PendingUpload {
        alloc_id: first.alloc_id,
        x: run_x,
        y: run_y,
        w: run_w,
        h: run_h,
        padded_x: run_x,
        padded_y: run_y,
        padded_w: run_w,
        padded_h: run_h,
        bytes_per_pixel,
        data,
    }
    .into_texture_upload()
}

fn align_upload_bytes_per_row(bytes_per_row: u32) -> u32 {
    let aligned_bytes_per_row = bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    aligned_bytes_per_row.max(bytes_per_row)
}

fn align_upload_bytes(
    data: &[u8],
    bytes_per_row: u32,
    aligned_bytes_per_row: u32,
    height: u32,
) -> Vec<u8> {
    let mut owned = vec![0; (aligned_bytes_per_row as usize).saturating_mul(height as usize)];
    for row in 0..height as usize {
        let src0 = row.saturating_mul(bytes_per_row as usize);
        let src1 = src0.saturating_add(bytes_per_row as usize);
        let dst0 = row.saturating_mul(aligned_bytes_per_row as usize);
        let dst1 = dst0.saturating_add(bytes_per_row as usize);
        owned[dst0..dst1].copy_from_slice(&data[src0..src1]);
    }
    owned
}

#[derive(Debug, Clone, Copy)]
struct PaddedGlyphSize {
    w_pad: u32,
    h_pad: u32,
    size: etagere::Size,
}

#[derive(Debug, Clone, Copy)]
struct AllocatedAtlasSlot {
    page_index: usize,
    alloc_id: etagere::AllocId,
    x: u32,
    y: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct GlyphAtlasPerfSnapshot {
    pub(super) uploads: u64,
    pub(super) upload_bytes: u64,
    pub(super) evicted_glyphs: u64,
    pub(super) evicted_pages: u64,
    pub(super) evicted_page_glyphs: u64,
    pub(super) resets: u64,
}

#[derive(Debug, Default)]
struct GlyphAtlasPerfStats {
    uploads: u64,
    upload_bytes: u64,
    evicted_glyphs: u64,
    evicted_pages: u64,
    evicted_page_glyphs: u64,
    resets: u64,
}

struct GlyphAtlasPage {
    allocator: etagere::BucketedAtlasAllocator,
    pending: Vec<PendingUpload>,
    live_glyph_refs: u32,
    last_used_epoch: u64,
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GlyphAtlasInsertError {
    OutOfSpace,
    TooLarge,
}

pub(super) struct GlyphAtlas {
    device: wgpu::Device,
    atlas_bind_group_layout: wgpu::BindGroupLayout,
    atlas_sampler: wgpu::Sampler,
    label_prefix: String,
    width: u32,
    height: u32,
    padding_px: u32,
    format: wgpu::TextureFormat,
    max_pages: usize,
    pages: Vec<GlyphAtlasPage>,
    glyphs: HashMap<GlyphKey, GlyphAtlasEntry>,
    revision: u64,
    used_px: u64,
    perf_frame: GlyphAtlasFramePerf,
    perf: GlyphAtlasPerfStats,
}

impl GlyphAtlas {
    pub(super) fn new(
        device: &wgpu::Device,
        atlas_bind_group_layout: &wgpu::BindGroupLayout,
        atlas_sampler: &wgpu::Sampler,
        label_prefix: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        initial_pages: usize,
        max_pages: usize,
    ) -> Self {
        let padding_px = 1;
        let max_pages = max_pages.max(1);
        let initial_pages = initial_pages.min(max_pages);
        let mut pages: Vec<GlyphAtlasPage> = Vec::with_capacity(max_pages);

        for i in 0..initial_pages {
            pages.push(Self::create_page(
                device,
                atlas_bind_group_layout,
                atlas_sampler,
                label_prefix,
                width,
                height,
                format,
                i,
            ));
        }

        Self {
            device: device.clone(),
            atlas_bind_group_layout: atlas_bind_group_layout.clone(),
            atlas_sampler: atlas_sampler.clone(),
            label_prefix: label_prefix.to_string(),
            width,
            height,
            padding_px,
            format,
            max_pages,
            pages,
            glyphs: HashMap::new(),
            revision: 0,
            used_px: 0,
            perf_frame: GlyphAtlasFramePerf::default(),
            perf: GlyphAtlasPerfStats::default(),
        }
    }

    fn create_page(
        device: &wgpu::Device,
        atlas_bind_group_layout: &wgpu::BindGroupLayout,
        atlas_sampler: &wgpu::Sampler,
        label_prefix: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        i: usize,
    ) -> GlyphAtlasPage {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{label_prefix} page {i}")),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{label_prefix} bind group page {i}")),
            layout: atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });

        GlyphAtlasPage {
            allocator: etagere::BucketedAtlasAllocator::new(atlas_allocator_size(width, height)),
            pending: Vec::new(),
            live_glyph_refs: 0,
            last_used_epoch: 0,
            bind_group,
            texture,
        }
    }

    fn try_grow_pages(&mut self) -> bool {
        if self.pages.len() >= self.max_pages {
            return false;
        }
        let i = self.pages.len();
        self.pages.push(Self::create_page(
            &self.device,
            &self.atlas_bind_group_layout,
            &self.atlas_sampler,
            &self.label_prefix,
            self.width,
            self.height,
            self.format,
            i,
        ));
        true
    }

    pub(super) fn begin_frame_diagnostics(&mut self) {
        self.perf_frame = GlyphAtlasFramePerf::default();
    }

    pub(super) fn diagnostics_snapshot(&self) -> RendererGlyphAtlasPerfSnapshot {
        let pages = self.pages.len() as u32;
        let max_pages = self.max_pages.min(u32::MAX as usize) as u32;
        let capacity_px = u64::from(self.width)
            .saturating_mul(u64::from(self.height))
            .saturating_mul(u64::from(pages));
        RendererGlyphAtlasPerfSnapshot {
            width: self.width,
            height: self.height,
            pages,
            max_pages,
            entries: self.glyphs.len() as u64,
            used_px: self.used_px,
            capacity_px,
            frame_hits: self.perf_frame.hits,
            frame_misses: self.perf_frame.misses,
            frame_inserts: self.perf_frame.inserts,
            frame_evict_glyphs: self.perf_frame.evict_glyphs,
            frame_evict_pages: self.perf_frame.evict_pages,
            frame_out_of_space: self.perf_frame.out_of_space,
            frame_too_large: self.perf_frame.too_large,
            frame_pending_uploads: self.perf_frame.pending_uploads,
            frame_pending_upload_bytes: self.perf_frame.pending_upload_bytes,
            frame_upload_bytes: self.perf_frame.upload_bytes,
        }
    }

    pub(super) fn take_perf_snapshot(&mut self) -> GlyphAtlasPerfSnapshot {
        let snap = GlyphAtlasPerfSnapshot {
            uploads: self.perf.uploads,
            upload_bytes: self.perf.upload_bytes,
            evicted_glyphs: self.perf.evicted_glyphs,
            evicted_pages: self.perf.evicted_pages,
            evicted_page_glyphs: self.perf.evicted_page_glyphs,
            resets: self.perf.resets,
        };
        self.perf = GlyphAtlasPerfStats::default();
        snap
    }

    pub(super) fn reset(&mut self) {
        self.perf.resets = self.perf.resets.saturating_add(1);
        self.revision = self.revision.saturating_add(1);
        self.glyphs.clear();
        self.used_px = 0;
        for page_index in 0..self.pages.len() {
            self.reset_page_storage(page_index);
        }
    }

    pub(super) fn bind_group(&self, page: u16) -> &wgpu::BindGroup {
        assert!(
            !self.pages.is_empty(),
            "glyph atlas has no pages; bind_group() is only valid after at least one insert"
        );
        let idx = self.page_index(page);
        &self.pages[idx].bind_group
    }

    pub(super) fn revision(&self) -> u64 {
        self.revision
    }

    pub(super) fn touch_bounds_for_key(
        &mut self,
        key: GlyphKey,
        x: i32,
        y: i32,
        epoch: u64,
    ) -> Option<(f32, f32, f32, f32)> {
        let entry = self.get(key, epoch)?;
        Some((
            x as f32 + entry.placement_left as f32,
            y as f32 - entry.placement_top as f32,
            entry.w as f32,
            entry.h as f32,
        ))
    }

    pub(super) fn touch_if_present(&mut self, key: GlyphKey, epoch: u64) -> bool {
        self.get(key, epoch).is_some()
    }

    pub(super) fn uv_for_key(&self, key: GlyphKey) -> Option<(u16, [f32; 4])> {
        let entry = self.glyphs.get(&key).copied()?;
        let w = self.width as f32;
        let h = self.height as f32;
        if w == 0.0 || h == 0.0 {
            return None;
        }
        let u0 = entry.x as f32 / w;
        let v0 = entry.y as f32 / h;
        let u1 = (entry.x.saturating_add(entry.w) as f32) / w;
        let v1 = (entry.y.saturating_add(entry.h) as f32) / h;
        Some((entry.page, [u0, v0, u1, v1]))
    }

    pub(super) fn contains_key(&self, key: GlyphKey) -> bool {
        self.glyphs.contains_key(&key)
    }

    #[cfg(test)]
    pub(super) fn pending_upload_bytes_for_key(&self, key: GlyphKey) -> Option<Vec<u8>> {
        let entry = self.glyphs.get(&key).copied()?;
        let page_idx = entry.page as usize;
        self.pages.get(page_idx).and_then(|page| {
            page.pending
                .iter()
                .find(|pending| {
                    pending.x == entry.x
                        && pending.y == entry.y
                        && pending.w == entry.w
                        && pending.h == entry.h
                })
                .map(|pending| pending.data.clone())
        })
    }

    pub(super) fn get(&mut self, key: GlyphKey, epoch: u64) -> Option<GlyphAtlasEntry> {
        let Some(hit) = self.touch_existing(key, epoch) else {
            self.record_miss();
            return None;
        };
        Some(hit)
    }

    fn inc_live_ref(&mut self, key: GlyphKey) {
        let mut page_to_increment = None;
        {
            let Some(entry) = self.glyphs.get_mut(&key) else {
                return;
            };
            if entry.live_refs == 0 {
                page_to_increment = Some(entry.page);
            }
            entry.live_refs = entry.live_refs.saturating_add(1);
        }
        if let Some(page) = page_to_increment {
            let idx = self.page_index(page);
            self.pages[idx].live_glyph_refs = self.pages[idx].live_glyph_refs.saturating_add(1);
        }
    }

    fn dec_live_ref(&mut self, key: GlyphKey) {
        let mut page_to_decrement = None;
        {
            let Some(entry) = self.glyphs.get_mut(&key) else {
                return;
            };
            if entry.live_refs == 0 {
                return;
            }
            entry.live_refs -= 1;
            if entry.live_refs == 0 {
                page_to_decrement = Some(entry.page);
            }
        }
        if let Some(page) = page_to_decrement {
            let idx = self.page_index(page);
            if self.pages[idx].live_glyph_refs > 0 {
                self.pages[idx].live_glyph_refs -= 1;
            }
        }
    }

    pub(super) fn inc_live_refs(&mut self, keys: &[GlyphKey]) {
        for &k in keys {
            self.inc_live_ref(k);
        }
    }

    pub(super) fn dec_live_refs(&mut self, keys: &[GlyphKey]) {
        for &k in keys {
            self.dec_live_ref(k);
        }
    }

    fn evict_lru_unreferenced_glyph(&mut self) -> bool {
        let Some((victim_key, victim_entry)) = self.find_lru_unreferenced_glyph() else {
            return false;
        };

        let page_idx = self.page_index(victim_entry.page);
        self.pages[page_idx]
            .allocator
            .deallocate(victim_entry.alloc_id);
        let _ = self.remove_glyph_entry(victim_key);
        self.perf.evicted_glyphs = self.perf.evicted_glyphs.saturating_add(1);
        self.revision = self.revision.saturating_add(1);
        self.perf_frame.evict_glyphs = self.perf_frame.evict_glyphs.saturating_add(1);
        true
    }

    fn evict_lru_unreferenced_page(&mut self) -> bool {
        let Some(victim) = self.find_lru_unreferenced_page() else {
            return false;
        };

        self.reset_page_storage(victim);

        let victim_page = victim as u16;
        let keys_to_remove: Vec<GlyphKey> = self
            .glyphs
            .iter()
            .filter_map(|(k, e)| (e.page == victim_page).then_some(*k))
            .collect();
        self.perf.evicted_pages = self.perf.evicted_pages.saturating_add(1);
        self.perf.evicted_page_glyphs = self
            .perf
            .evicted_page_glyphs
            .saturating_add(keys_to_remove.len() as u64);
        for k in keys_to_remove {
            let _ = self.remove_glyph_entry(k);
        }

        self.revision = self.revision.saturating_add(1);
        self.perf_frame.evict_pages = self.perf_frame.evict_pages.saturating_add(1);
        true
    }

    pub(super) fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        for page in &mut self.pages {
            for upload in prepare_texture_uploads(std::mem::take(&mut page.pending)) {
                self.perf.uploads = self.perf.uploads.saturating_add(1);
                self.perf.upload_bytes = self
                    .perf
                    .upload_bytes
                    .saturating_add(upload.bytes.len() as u64);
                self.perf_frame.upload_bytes = self
                    .perf_frame
                    .upload_bytes
                    .saturating_add(upload.bytes.len() as u64);

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &page.texture,
                        mip_level: 0,
                        origin: upload.origin,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &upload.bytes,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(upload.bytes_per_row),
                        rows_per_image: Some(upload.rows_per_image),
                    },
                    upload.extent,
                );
            }
        }
    }

    pub(super) fn get_or_insert(
        &mut self,
        key: GlyphKey,
        w: u32,
        h: u32,
        placement_left: i32,
        placement_top: i32,
        bytes_per_pixel: u32,
        data: Vec<u8>,
        epoch: u64,
    ) -> Result<GlyphAtlasEntry, GlyphAtlasInsertError> {
        if let Some(hit) = self.touch_existing(key, epoch) {
            return Ok(hit);
        }
        self.record_miss();

        let padded = self.padded_size_for_insert(w, h)?;
        self.ensure_page_available()?;
        let slot = self.allocate_slot_with_recovery(padded.size)?;
        Ok(self.insert_allocated_glyph(
            key,
            slot,
            padded,
            w,
            h,
            placement_left,
            placement_top,
            bytes_per_pixel,
            data,
            epoch,
        ))
    }

    fn page_index(&self, page: u16) -> usize {
        debug_assert!(!self.pages.is_empty());
        (page as usize).min(self.pages.len().saturating_sub(1))
    }

    fn touch_existing(&mut self, key: GlyphKey, epoch: u64) -> Option<GlyphAtlasEntry> {
        let hit = self.glyphs.get_mut(&key)?;
        self.perf_frame.hits = self.perf_frame.hits.saturating_add(1);
        hit.last_used_epoch = epoch;
        let page = hit.page;
        let entry = *hit;
        let idx = self.page_index(page);
        self.pages[idx].last_used_epoch = epoch;
        Some(entry)
    }

    fn record_miss(&mut self) {
        self.perf_frame.misses = self.perf_frame.misses.saturating_add(1);
    }

    fn too_large_error(&mut self) -> GlyphAtlasInsertError {
        self.perf_frame.too_large = self.perf_frame.too_large.saturating_add(1);
        GlyphAtlasInsertError::TooLarge
    }

    fn out_of_space_error(&mut self) -> GlyphAtlasInsertError {
        self.perf_frame.out_of_space = self.perf_frame.out_of_space.saturating_add(1);
        GlyphAtlasInsertError::OutOfSpace
    }

    fn padded_size_for_insert(
        &mut self,
        w: u32,
        h: u32,
    ) -> Result<PaddedGlyphSize, GlyphAtlasInsertError> {
        let pad = self.padding_px;
        let w_pad = w.saturating_add(pad.saturating_mul(2));
        let h_pad = h.saturating_add(pad.saturating_mul(2));
        if w == 0 || h == 0 || w_pad == 0 || h_pad == 0 || w_pad > self.width || h_pad > self.height
        {
            return Err(self.too_large_error());
        }

        Ok(PaddedGlyphSize {
            w_pad,
            h_pad,
            size: etagere::Size::new(w_pad as i32, h_pad as i32),
        })
    }

    fn ensure_page_available(&mut self) -> Result<(), GlyphAtlasInsertError> {
        if self.pages.is_empty() && !self.try_grow_pages() {
            return Err(self.out_of_space_error());
        }
        Ok(())
    }

    fn allocate_slot_with_recovery(
        &mut self,
        size: etagere::Size,
    ) -> Result<AllocatedAtlasSlot, GlyphAtlasInsertError> {
        let mut guard = 0_u32;
        loop {
            guard = guard.saturating_add(1);
            if guard >= GLYPH_ATLAS_INSERT_GUARD_LIMIT {
                return Err(self.out_of_space_error());
            }

            if let Some(slot) = self.try_allocate_slot(size) {
                return Ok(slot);
            }

            if !self.recover_space_for_insert() {
                return Err(self.out_of_space_error());
            }
        }
    }

    fn recover_space_for_insert(&mut self) -> bool {
        self.try_grow_pages()
            || self.evict_lru_unreferenced_glyph()
            || self.evict_lru_unreferenced_page()
    }

    fn reset_page_storage(&mut self, page_index: usize) {
        let allocator_size = atlas_allocator_size(self.width, self.height);
        let page = &mut self.pages[page_index];
        page.allocator = etagere::BucketedAtlasAllocator::new(allocator_size);
        page.pending.clear();
        page.live_glyph_refs = 0;
        page.last_used_epoch = 0;
    }

    fn find_lru_unreferenced_glyph(&self) -> Option<(GlyphKey, GlyphAtlasEntry)> {
        let mut victim: Option<(GlyphKey, GlyphAtlasEntry)> = None;
        for (&key, &entry) in &self.glyphs {
            if entry.live_refs > 0 {
                continue;
            }
            let pick = match victim {
                None => true,
                Some((_, prev)) => entry.last_used_epoch < prev.last_used_epoch,
            };
            if pick {
                victim = Some((key, entry));
            }
        }
        victim
    }

    fn find_lru_unreferenced_page(&self) -> Option<usize> {
        let mut victim: Option<usize> = None;
        for (page_index, page) in self.pages.iter().enumerate() {
            if page.live_glyph_refs > 0 {
                continue;
            }
            let pick = match victim {
                None => true,
                Some(prev) => page.last_used_epoch < self.pages[prev].last_used_epoch,
            };
            if pick {
                victim = Some(page_index);
            }
        }
        victim
    }

    fn remove_glyph_entry(&mut self, key: GlyphKey) -> Option<GlyphAtlasEntry> {
        let entry = self.glyphs.remove(&key)?;
        self.remove_pending_upload_for_entry(entry);
        self.release_entry_area(entry);
        Some(entry)
    }

    fn remove_pending_upload_for_entry(&mut self, entry: GlyphAtlasEntry) {
        let page_idx = self.page_index(entry.page);
        if let Some(page) = self.pages.get_mut(page_idx) {
            remove_pending_upload_for_alloc(&mut page.pending, entry.alloc_id);
        }
    }

    fn release_entry_area(&mut self, entry: GlyphAtlasEntry) {
        self.used_px = self
            .used_px
            .saturating_sub(self.padded_entry_area_px(entry));
    }

    fn padded_entry_area_px(&self, entry: GlyphAtlasEntry) -> u64 {
        let pad = self.padding_px;
        let w_pad = entry.w.saturating_add(pad.saturating_mul(2));
        let h_pad = entry.h.saturating_add(pad.saturating_mul(2));
        u64::from(w_pad).saturating_mul(u64::from(h_pad))
    }

    fn try_allocate_slot(&mut self, size: etagere::Size) -> Option<AllocatedAtlasSlot> {
        let pad = self.padding_px;
        for (page_index, page) in self.pages.iter_mut().enumerate() {
            let Some(allocation) = page.allocator.allocate(size) else {
                continue;
            };

            let Ok(base_x) = u32::try_from(allocation.rectangle.min.x) else {
                page.allocator.deallocate(allocation.id);
                continue;
            };
            let Ok(base_y) = u32::try_from(allocation.rectangle.min.y) else {
                page.allocator.deallocate(allocation.id);
                continue;
            };

            return Some(AllocatedAtlasSlot {
                page_index,
                alloc_id: allocation.id,
                x: base_x.saturating_add(pad),
                y: base_y.saturating_add(pad),
            });
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_allocated_glyph(
        &mut self,
        key: GlyphKey,
        slot: AllocatedAtlasSlot,
        padded: PaddedGlyphSize,
        w: u32,
        h: u32,
        placement_left: i32,
        placement_top: i32,
        bytes_per_pixel: u32,
        data: Vec<u8>,
        epoch: u64,
    ) -> GlyphAtlasEntry {
        let page = &mut self.pages[slot.page_index];
        let padded_x = slot.x.saturating_sub(self.padding_px);
        let padded_y = slot.y.saturating_sub(self.padding_px);
        page.pending.push(PendingUpload {
            alloc_id: slot.alloc_id,
            x: slot.x,
            y: slot.y,
            w,
            h,
            padded_x,
            padded_y,
            padded_w: padded.w_pad,
            padded_h: padded.h_pad,
            bytes_per_pixel,
            data,
        });
        page.last_used_epoch = epoch;

        self.perf_frame.inserts = self.perf_frame.inserts.saturating_add(1);
        self.perf_frame.pending_uploads = self.perf_frame.pending_uploads.saturating_add(1);
        self.perf_frame.pending_upload_bytes = self.perf_frame.pending_upload_bytes.saturating_add(
            u64::from(w)
                .saturating_mul(u64::from(h))
                .saturating_mul(u64::from(bytes_per_pixel)),
        );
        self.used_px = self
            .used_px
            .saturating_add(u64::from(padded.w_pad).saturating_mul(u64::from(padded.h_pad)));

        let entry = GlyphAtlasEntry {
            page: slot.page_index as u16,
            alloc_id: slot.alloc_id,
            x: slot.x,
            y: slot.y,
            w,
            h,
            placement_left,
            placement_top,
            live_refs: 0,
            last_used_epoch: epoch,
        };
        self.glyphs.insert(key, entry);
        self.revision = self.revision.saturating_add(1);
        entry
    }
}
