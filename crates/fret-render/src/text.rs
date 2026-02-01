use cosmic_text::{Family, FontSystem};
use fret_core::scene::{Scene, SceneOp};
use fret_core::{
    AttributedText, CaretAffinity, HitTestResult, Point, Rect, Size, TextBlobId, TextConstraints,
    TextInputRef, TextMetrics, TextOverflow, TextSlant, TextSpan, TextStyle, TextWrap,
    geometry::Px,
};
use slotmap::SlotMap;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
    sync::Arc,
};

use parley::fontique::GenericFamily as ParleyGenericFamily;

struct FretFallback;

impl cosmic_text::Fallback for FretFallback {
    fn common_fallback(&self) -> &[&'static str] {
        #[cfg(target_os = "windows")]
        {
            &[
                // UI
                "Segoe UI",
                "Tahoma",
                // CJK
                "Microsoft YaHei UI",
                "Microsoft YaHei",
                "Yu Gothic UI",
                "Meiryo UI",
                "Meiryo",
                "Nirmala UI",
                // Emoji
                "Segoe UI Emoji",
                "Segoe UI Symbol",
            ]
        }
        #[cfg(target_os = "macos")]
        {
            &[
                // UI (attempt a couple of common names; fontdb will skip missing families)
                "SF Pro Text",
                ".SF NS Text",
                "Helvetica Neue",
                // CJK
                "PingFang SC",
                "PingFang TC",
                "Hiragino Sans",
                // Emoji
                "Apple Color Emoji",
            ]
        }
        #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
        {
            &[
                // UI
                "Noto Sans",
                "DejaVu Sans",
                "Liberation Sans",
                // CJK
                "Noto Sans CJK SC",
                "Noto Sans CJK JP",
                "Noto Sans CJK TC",
                // Emoji
                "Noto Color Emoji",
            ]
        }
        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            all(unix, not(any(target_os = "macos", target_os = "android")))
        )))]
        {
            &[]
        }
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        <cosmic_text::PlatformFallback as cosmic_text::Fallback>::forbidden_fallback(
            &cosmic_text::PlatformFallback,
        )
    }

    fn script_fallback(&self, script: unicode_script::Script, locale: &str) -> &[&'static str] {
        <cosmic_text::PlatformFallback as cosmic_text::Fallback>::script_fallback(
            &cosmic_text::PlatformFallback,
            script,
            locale,
        )
    }
}

fn build_installed_family_set(db: &cosmic_text::fontdb::Database) -> HashSet<String> {
    let mut set = HashSet::new();
    for face in db.faces() {
        for (family, _lang) in &face.families {
            set.insert(family.to_ascii_lowercase());
        }
    }
    set
}

fn first_installed_family<'a>(
    installed: &HashSet<String>,
    candidates: &'a [&'a str],
) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|name| installed.contains(&name.to_ascii_lowercase()))
}

fn default_sans_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["Segoe UI", "Tahoma", "Arial"]
    }
    #[cfg(target_os = "macos")]
    {
        &["SF Pro Text", ".SF NS Text", "Helvetica Neue", "Helvetica"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["Noto Sans", "DejaVu Sans", "Liberation Sans"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        &[]
    }
}

fn default_monospace_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["Cascadia Mono", "Consolas", "Courier New"]
    }
    #[cfg(target_os = "macos")]
    {
        &["SF Mono", "Menlo", "Monaco"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["Noto Sans Mono", "DejaVu Sans Mono", "Liberation Mono"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        &[]
    }
}

fn default_serif_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["Times New Roman", "Georgia"]
    }
    #[cfg(target_os = "macos")]
    {
        &["New York", "Times New Roman", "Times"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["DejaVu Serif", "Noto Serif", "Liberation Serif"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        &[]
    }
}

fn font_stack_cache_key(locale: &str, db: &cosmic_text::fontdb::Database, db_revision: u64) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    locale.hash(&mut hasher);

    db.family_name(&Family::SansSerif).hash(&mut hasher);
    db.family_name(&Family::Serif).hash(&mut hasher);
    db.family_name(&Family::Monospace).hash(&mut hasher);

    // Include the framework-level fallback policy so changing it can't reuse stale blobs.
    <FretFallback as cosmic_text::Fallback>::common_fallback(&FretFallback).hash(&mut hasher);
    <cosmic_text::PlatformFallback as cosmic_text::Fallback>::forbidden_fallback(
        &cosmic_text::PlatformFallback,
    )
    .hash(&mut hasher);

    // Ensure font-db mutations (user font loading, web font injection, etc.) participate in the
    // cache key even when generic family names are unchanged.
    db_revision.hash(&mut hasher);

    hasher.finish()
}

#[derive(Debug, Clone)]
pub struct GlyphInstance {
    /// Logical-space rect relative to the text baseline origin.
    pub rect: [f32; 4],
    pub paint_span: Option<u16>,
    key: GlyphKey,
}

impl GlyphInstance {
    pub fn kind(&self) -> GlyphQuadKind {
        self.key.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphQuadKind {
    Mask,
    Color,
    Subpixel,
}

#[derive(Debug, Clone)]
pub struct TextBlob {
    pub shape: Arc<TextShape>,
    pub paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
    ref_count: u32,
}

#[derive(Debug, Clone)]
pub struct TextShape {
    pub glyphs: Arc<[GlyphInstance]>,
    pub metrics: TextMetrics,
    pub lines: Arc<[TextLine]>,
    pub caret_stops: Arc<[(usize, Px)]>,
}

#[derive(Debug, Clone)]
pub struct TextLine {
    pub start: usize,
    pub end: usize,
    pub width: Px,
    pub y_top: Px,
    pub height: Px,
    pub caret_stops: Vec<(usize, Px)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextBlobKey {
    text: Arc<str>,
    spans_shaping_key: u64,
    spans_paint_key: u64,
    backend: u8,
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_bits: Option<u32>,
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    overflow: TextOverflow,
    scale_bits: u32,
}

impl TextBlobKey {
    fn new(
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        font_stack_key: u64,
    ) -> Self {
        let max_width_bits = constraints.max_width.map(|w| w.0.to_bits());
        Self {
            text: Arc::<str>::from(text),
            spans_shaping_key: 0,
            spans_paint_key: 0,
            backend: 0,
            font: style.font.clone(),
            font_stack_key,
            size_bits: style.size.0.to_bits(),
            weight: style.weight.0,
            slant: match style.slant {
                TextSlant::Normal => 0,
                TextSlant::Italic => 1,
                TextSlant::Oblique => 2,
            },
            line_height_bits: style.line_height.map(|px| px.0.to_bits()),
            letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
            max_width_bits,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            scale_bits: constraints.scale_factor.to_bits(),
        }
    }

    fn new_attributed(
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
        font_stack_key: u64,
    ) -> Self {
        let mut out = Self::new(rich.text.as_ref(), base_style, constraints, font_stack_key);
        out.spans_shaping_key = spans_shaping_fingerprint(&rich.spans);
        out.spans_paint_key = spans_paint_fingerprint(&rich.spans);
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextShapeKey {
    text: Arc<str>,
    spans_shaping_key: u64,
    backend: u8,
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_bits: Option<u32>,
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    overflow: TextOverflow,
    scale_bits: u32,
}

impl TextShapeKey {
    fn from_blob_key(key: &TextBlobKey) -> Self {
        Self {
            text: key.text.clone(),
            spans_shaping_key: key.spans_shaping_key,
            backend: key.backend,
            font: key.font.clone(),
            font_stack_key: key.font_stack_key,
            size_bits: key.size_bits,
            weight: key.weight,
            slant: key.slant,
            line_height_bits: key.line_height_bits,
            letter_spacing_bits: key.letter_spacing_bits,
            max_width_bits: key.max_width_bits,
            wrap: key.wrap,
            overflow: key.overflow,
            scale_bits: key.scale_bits,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FontFaceKey {
    blob_id: u64,
    face_index: u32,
    variation_key: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    font: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    kind: GlyphQuadKind,
}

fn stable_font_blob_id(bytes: &[u8]) -> u64 {
    // Stable, dependency-free fingerprint for font bytes.
    // This intentionally avoids `DefaultHasher` to stay deterministic across Rust versions.
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    fn fnv1a64_update(mut hash: u64, input: &[u8]) -> u64 {
        for b in input {
            hash ^= u64::from(*b);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv1a64_update(hash, b"fret.text.font_blob_id.v1\0");
    hash = fnv1a64_update(hash, &(bytes.len() as u64).to_le_bytes());
    fnv1a64_update(hash, bytes)
}

fn subpixel_bin_q4(pos: f32) -> (i32, u8) {
    // Keep behavior aligned with cosmic-text's `SubpixelBin::new`.
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

fn subpixel_bin_as_float(bin: u8) -> f32 {
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

fn subpixel_bin_y(pos: f32) -> (i32, u8) {
    let (y, bin) = subpixel_bin_q4(pos);
    if SUBPIXEL_VARIANTS_Y <= 1 {
        (y, 0)
    } else {
        (y, bin)
    }
}

const TEXT_ATLAS_MAX_PAGES: usize = 2;

#[derive(Debug, Clone, Copy)]
struct GlyphAtlasEntry {
    page: u16,
    alloc_id: etagere::AllocId,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    live_refs: u32,
    last_used_epoch: u64,
}

#[derive(Debug)]
struct PendingUpload {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
}

struct GlyphAtlasPage {
    allocator: etagere::BucketedAtlasAllocator,
    pending: Vec<PendingUpload>,
    live_glyph_refs: u32,
    last_used_epoch: u64,
    bind_group: wgpu::BindGroup,
    _texture: wgpu::Texture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlyphAtlasInsertError {
    OutOfSpace,
    TooLarge,
}

struct GlyphAtlas {
    width: u32,
    height: u32,
    padding_px: u32,
    pages: Vec<GlyphAtlasPage>,
    glyphs: HashMap<GlyphKey, GlyphAtlasEntry>,
    revision: u64,
}

impl GlyphAtlas {
    fn new(
        device: &wgpu::Device,
        atlas_bind_group_layout: &wgpu::BindGroupLayout,
        atlas_sampler: &wgpu::Sampler,
        label_prefix: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        page_count: usize,
    ) -> Self {
        let padding_px = 1;
        let mut pages: Vec<GlyphAtlasPage> = Vec::with_capacity(page_count.max(1));

        for i in 0..page_count.max(1) {
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

            pages.push(GlyphAtlasPage {
                allocator: etagere::BucketedAtlasAllocator::new(etagere::Size::new(
                    width as i32,
                    height as i32,
                )),
                pending: Vec::new(),
                live_glyph_refs: 0,
                last_used_epoch: 0,
                bind_group,
                _texture: texture,
            });
        }

        Self {
            width,
            height,
            padding_px,
            pages,
            glyphs: HashMap::new(),
            revision: 0,
        }
    }

    fn reset(&mut self) {
        self.revision = self.revision.saturating_add(1);
        self.glyphs.clear();
        for page in &mut self.pages {
            page.allocator = etagere::BucketedAtlasAllocator::new(etagere::Size::new(
                self.width as i32,
                self.height as i32,
            ));
            page.pending.clear();
            page.live_glyph_refs = 0;
            page.last_used_epoch = 0;
        }
    }

    fn bind_group(&self, page: u16) -> &wgpu::BindGroup {
        let idx = (page as usize).min(self.pages.len().saturating_sub(1));
        &self.pages[idx].bind_group
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn entry(&self, key: GlyphKey) -> Option<GlyphAtlasEntry> {
        self.glyphs.get(&key).copied()
    }

    fn get(&mut self, key: GlyphKey, epoch: u64) -> Option<GlyphAtlasEntry> {
        let hit = self.glyphs.get_mut(&key)?;
        hit.last_used_epoch = epoch;
        let idx = (hit.page as usize).min(self.pages.len().saturating_sub(1));
        self.pages[idx].last_used_epoch = epoch;
        Some(*hit)
    }

    fn inc_live_ref(&mut self, key: GlyphKey) {
        let Some(entry) = self.glyphs.get_mut(&key) else {
            return;
        };
        if entry.live_refs == 0 {
            let idx = (entry.page as usize).min(self.pages.len().saturating_sub(1));
            self.pages[idx].live_glyph_refs = self.pages[idx].live_glyph_refs.saturating_add(1);
        }
        entry.live_refs = entry.live_refs.saturating_add(1);
    }

    fn dec_live_ref(&mut self, key: GlyphKey) {
        let Some(entry) = self.glyphs.get_mut(&key) else {
            return;
        };
        if entry.live_refs == 0 {
            return;
        }
        entry.live_refs -= 1;
        if entry.live_refs == 0 {
            let idx = (entry.page as usize).min(self.pages.len().saturating_sub(1));
            if self.pages[idx].live_glyph_refs > 0 {
                self.pages[idx].live_glyph_refs -= 1;
            }
        }
    }

    fn inc_live_refs(&mut self, keys: &[GlyphKey]) {
        for &k in keys {
            self.inc_live_ref(k);
        }
    }

    fn dec_live_refs(&mut self, keys: &[GlyphKey]) {
        for &k in keys {
            self.dec_live_ref(k);
        }
    }

    fn evict_lru_unreferenced_glyph(&mut self) -> bool {
        let mut victim: Option<(GlyphKey, GlyphAtlasEntry)> = None;
        for (&k, &e) in &self.glyphs {
            if e.live_refs > 0 {
                continue;
            }
            let pick = match victim {
                None => true,
                Some((_, prev)) => e.last_used_epoch < prev.last_used_epoch,
            };
            if pick {
                victim = Some((k, e));
            }
        }

        let Some((victim_key, victim_entry)) = victim else {
            return false;
        };

        let page_idx = (victim_entry.page as usize).min(self.pages.len().saturating_sub(1));
        self.pages[page_idx]
            .allocator
            .deallocate(victim_entry.alloc_id);
        self.glyphs.remove(&victim_key);
        self.revision = self.revision.saturating_add(1);
        true
    }

    fn evict_lru_unreferenced_page(&mut self) -> bool {
        let mut victim: Option<usize> = None;
        for (idx, page) in self.pages.iter().enumerate() {
            if page.live_glyph_refs > 0 {
                continue;
            }
            let pick = match victim {
                None => true,
                Some(prev) => page.last_used_epoch < self.pages[prev].last_used_epoch,
            };
            if pick {
                victim = Some(idx);
            }
        }

        let Some(victim) = victim else {
            return false;
        };

        self.pages[victim].allocator = etagere::BucketedAtlasAllocator::new(etagere::Size::new(
            self.width as i32,
            self.height as i32,
        ));
        self.pages[victim].pending.clear();
        self.pages[victim].last_used_epoch = 0;
        self.pages[victim].live_glyph_refs = 0;

        let victim_page = victim as u16;
        let keys_to_remove: Vec<GlyphKey> = self
            .glyphs
            .iter()
            .filter_map(|(k, e)| (e.page == victim_page).then_some(*k))
            .collect();
        for k in keys_to_remove {
            self.glyphs.remove(&k);
        }

        self.revision = self.revision.saturating_add(1);
        true
    }

    fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        for page in &mut self.pages {
            for upload in std::mem::take(&mut page.pending) {
                if upload.w == 0 || upload.h == 0 {
                    continue;
                }

                let bytes_per_row = upload.w.saturating_mul(upload.bytes_per_pixel);
                if bytes_per_row == 0 {
                    continue;
                }

                let expected_len = (bytes_per_row as usize).saturating_mul(upload.h as usize);
                debug_assert_eq!(upload.data.len(), expected_len);
                if upload.data.len() != expected_len {
                    continue;
                }

                let aligned_bytes_per_row = bytes_per_row
                    .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                    * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
                let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);

                let mut owned: Vec<u8> = Vec::new();
                let bytes: &[u8] = if aligned_bytes_per_row == bytes_per_row {
                    &upload.data
                } else {
                    owned.resize(
                        (aligned_bytes_per_row as usize).saturating_mul(upload.h as usize),
                        0,
                    );
                    for row in 0..upload.h as usize {
                        let src0 = row.saturating_mul(bytes_per_row as usize);
                        let src1 = src0.saturating_add(bytes_per_row as usize);
                        let dst0 = row.saturating_mul(aligned_bytes_per_row as usize);
                        let dst1 = dst0.saturating_add(bytes_per_row as usize);
                        owned[dst0..dst1].copy_from_slice(&upload.data[src0..src1]);
                    }
                    &owned
                };

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &page._texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: upload.x,
                            y: upload.y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    bytes,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(aligned_bytes_per_row),
                        rows_per_image: Some(upload.h),
                    },
                    wgpu::Extent3d {
                        width: upload.w,
                        height: upload.h,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    fn get_or_insert(
        &mut self,
        key: GlyphKey,
        w: u32,
        h: u32,
        bytes_per_pixel: u32,
        data: Vec<u8>,
        epoch: u64,
    ) -> Result<GlyphAtlasEntry, GlyphAtlasInsertError> {
        if let Some(hit) = self.glyphs.get_mut(&key) {
            hit.last_used_epoch = epoch;
            let idx = (hit.page as usize).min(self.pages.len().saturating_sub(1));
            self.pages[idx].last_used_epoch = epoch;
            return Ok(*hit);
        }

        let pad = self.padding_px;
        let w_pad = w.saturating_add(pad.saturating_mul(2));
        let h_pad = h.saturating_add(pad.saturating_mul(2));
        if w == 0 || h == 0 || w_pad == 0 || h_pad == 0 || w_pad > self.width || h_pad > self.height
        {
            return Err(GlyphAtlasInsertError::TooLarge);
        }

        let size = etagere::Size::new(w_pad as i32, h_pad as i32);

        for _ in 0..=self.pages.len() {
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

                let x = base_x.saturating_add(pad);
                let y = base_y.saturating_add(pad);

                page.pending.push(PendingUpload {
                    x,
                    y,
                    w,
                    h,
                    bytes_per_pixel,
                    data,
                });
                page.last_used_epoch = epoch;

                let entry = GlyphAtlasEntry {
                    page: page_index as u16,
                    alloc_id: allocation.id,
                    x,
                    y,
                    w,
                    h,
                    live_refs: 0,
                    last_used_epoch: epoch,
                };
                self.glyphs.insert(key, entry);
                self.revision = self.revision.saturating_add(1);
                return Ok(entry);
            }

            if self.evict_lru_unreferenced_glyph() {
                continue;
            }
            if self.evict_lru_unreferenced_page() {
                continue;
            }
            return Err(GlyphAtlasInsertError::OutOfSpace);
        }

        Err(GlyphAtlasInsertError::OutOfSpace)
    }
}

fn subpixel_mask_to_alpha(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 4);
    for rgba in data.chunks_exact(4) {
        out.push(rgba[0].max(rgba[1]).max(rgba[2]));
    }
    out
}

fn collect_font_names(db: &cosmic_text::fontdb::Database) -> Vec<String> {
    let mut by_lower: HashMap<String, String> = HashMap::new();

    for face in db.faces() {
        for (family, _lang) in &face.families {
            let key = family.to_ascii_lowercase();
            by_lower.entry(key).or_insert_with(|| family.clone());
        }
    }

    for family in [
        db.family_name(&Family::SansSerif),
        db.family_name(&Family::Serif),
        db.family_name(&Family::Monospace),
    ] {
        let key = family.to_ascii_lowercase();
        by_lower.entry(key).or_insert_with(|| family.to_string());
    }

    let mut names: Vec<String> = by_lower.into_values().collect();
    names.sort_unstable_by(|a, b| {
        a.to_ascii_lowercase()
            .cmp(&b.to_ascii_lowercase())
            .then(a.cmp(b))
    });
    names
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextMeasureKey {
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_bits: Option<u32>,
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    overflow: TextOverflow,
    scale_bits: u32,
}

impl TextMeasureKey {
    fn new(style: &TextStyle, constraints: TextConstraints, font_stack_key: u64) -> Self {
        let max_width_bits = constraints.max_width.map(|w| w.0.to_bits());
        Self {
            font: style.font.clone(),
            font_stack_key,
            size_bits: style.size.0.to_bits(),
            weight: style.weight.0,
            slant: match style.slant {
                TextSlant::Normal => 0,
                TextSlant::Italic => 1,
                TextSlant::Oblique => 2,
            },
            line_height_bits: style.line_height.map(|px| px.0.to_bits()),
            letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
            max_width_bits,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            scale_bits: constraints.scale_factor.to_bits(),
        }
    }
}

#[derive(Debug, Clone)]
struct TextMeasureEntry {
    text_hash: u64,
    spans_hash: u64,
    text: Arc<str>,
    spans: Option<Arc<[TextSpan]>>,
    metrics: TextMetrics,
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn spans_shaping_fingerprint(spans: &[TextSpan]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "fret.text.spans.shaping.v0".hash(&mut hasher);
    for s in spans {
        s.len.hash(&mut hasher);
        s.shaping.font.hash(&mut hasher);
        s.shaping.weight.hash(&mut hasher);
        s.shaping.slant.hash(&mut hasher);
        s.shaping
            .letter_spacing_em
            .map(|v| v.to_bits())
            .hash(&mut hasher);
    }
    hasher.finish()
}

fn spans_paint_fingerprint(spans: &[TextSpan]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "fret.text.spans.paint.v0".hash(&mut hasher);
    for s in spans {
        s.len.hash(&mut hasher);
        match s.paint.fg {
            None => 0u8.hash(&mut hasher),
            Some(c) => {
                1u8.hash(&mut hasher);
                c.r.to_bits().hash(&mut hasher);
                c.g.to_bits().hash(&mut hasher);
                c.b.to_bits().hash(&mut hasher);
                c.a.to_bits().hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

pub struct TextSystem {
    font_system: FontSystem,
    parley_shaper: crate::text_v2::parley_shaper::ParleyShaper,
    parley_scale: parley::swash::scale::ScaleContext,
    font_stack_key: u64,
    font_db_revision: u64,

    blobs: SlotMap<TextBlobId, TextBlob>,
    blob_cache: HashMap<TextBlobKey, TextBlobId>,
    blob_key_by_id: HashMap<TextBlobId, TextBlobKey>,
    shape_cache: HashMap<TextShapeKey, Arc<TextShape>>,
    measure_cache: HashMap<TextMeasureKey, VecDeque<TextMeasureEntry>>,

    mask_atlas: GlyphAtlas,
    color_atlas: GlyphAtlas,
    subpixel_atlas: GlyphAtlas,
    atlas_bind_group_layout: wgpu::BindGroupLayout,

    text_pin_mask: Vec<Vec<GlyphKey>>,
    text_pin_color: Vec<Vec<GlyphKey>>,
    text_pin_subpixel: Vec<Vec<GlyphKey>>,
    font_bytes_by_blob_id: HashMap<u64, Arc<[u8]>>,
    font_face_key_by_fontique: HashMap<(u64, u32), FontFaceKey>,
}

pub type TextFontFamilyConfig = fret_core::TextFontFamilyConfig;

fn metrics_from_wrapped_lines(
    lines: &[crate::text_v2::parley_shaper::ShapedLineLayout],
    scale: f32,
) -> TextMetrics {
    let first_baseline_px = lines.first().map(|l| l.baseline.max(0.0)).unwrap_or(0.0);

    let mut max_w_px = 0.0_f32;
    let mut total_h_px = 0.0_f32;
    for line in lines {
        max_w_px = max_w_px.max(line.width.max(0.0));
        total_h_px += line.line_height.max(0.0);
    }

    TextMetrics {
        size: Size::new(
            Px((max_w_px / scale).max(0.0)),
            Px((total_h_px / scale).max(0.0)),
        ),
        baseline: Px((first_baseline_px / scale).max(0.0)),
    }
}

impl TextSystem {
    /// Returns a sorted list of available font family names.
    ///
    /// This is intended for settings/UI pickers. The result is best-effort and platform-dependent.
    pub fn all_font_names(&self) -> Vec<String> {
        collect_font_names(self.font_system.db())
    }

    pub fn font_stack_key(&self) -> u64 {
        self.font_stack_key
    }

    /// Adds font bytes (TTF/OTF/TTC) to the font database.
    ///
    /// Returns the number of newly loaded faces. When this returns non-zero, all cached text blobs
    /// and atlas entries are cleared to avoid reusing stale shaping/rasterization results.
    pub fn add_fonts(&mut self, fonts: impl IntoIterator<Item = Vec<u8>>) -> usize {
        let fonts: Vec<Vec<u8>> = fonts.into_iter().collect();

        let before_faces = self.font_system.db().faces().count();
        for data in fonts.iter().cloned() {
            self.font_system.db_mut().load_font_data(data);
        }
        let after_faces = self.font_system.db().faces().count();
        let added = after_faces.saturating_sub(before_faces);
        let parley_added = self.parley_shaper.add_fonts(fonts.into_iter());

        if added > 0 || parley_added > 0 {
            self.font_db_revision = self.font_db_revision.saturating_add(1);
            self.font_stack_key = font_stack_cache_key(
                self.font_system.locale(),
                self.font_system.db(),
                self.font_db_revision,
            );
            self.blobs.clear();
            self.blob_cache.clear();
            self.blob_key_by_id.clear();
            self.shape_cache.clear();
            self.measure_cache.clear();
            self.mask_atlas.reset();
            self.color_atlas.reset();
            self.subpixel_atlas.reset();
            self.text_pin_mask.iter_mut().for_each(|v| v.clear());
            self.text_pin_color.iter_mut().for_each(|v| v.clear());
            self.text_pin_subpixel.iter_mut().for_each(|v| v.clear());
            self.font_bytes_by_blob_id.clear();
            self.font_face_key_by_fontique.clear();
        }

        added
    }

    pub fn new(device: &wgpu::Device) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret glyph atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret glyph atlas bind group layout"),
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
            });

        let mask_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph mask atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::R8Unorm,
            TEXT_ATLAS_MAX_PAGES,
        );
        let color_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph color atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            TEXT_ATLAS_MAX_PAGES,
        );
        let subpixel_atlas = GlyphAtlas::new(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph subpixel atlas",
            atlas_width,
            atlas_height,
            wgpu::TextureFormat::Rgba8Unorm,
            TEXT_ATLAS_MAX_PAGES,
        );

        let (locale, mut db) = FontSystem::new().into_locale_and_db();
        let installed = build_installed_family_set(&db);

        let sans = first_installed_family(&installed, default_sans_candidates());
        if let Some(sans) = sans {
            db.set_sans_serif_family(sans);
        }
        let serif = first_installed_family(&installed, default_serif_candidates());
        if let Some(serif) = serif {
            db.set_serif_family(serif);
        }
        let mono = first_installed_family(&installed, default_monospace_candidates());
        if let Some(mono) = mono {
            db.set_monospace_family(mono);
        }

        let font_db_revision = 0u64;
        let font_stack_key = font_stack_cache_key(&locale, &db, font_db_revision);
        let font_system = FontSystem::new_with_locale_and_db_and_fallback(locale, db, FretFallback);

        let mut parley_shaper = crate::text_v2::parley_shaper::ParleyShaper::new();
        if let Some(sans) = sans {
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::SansSerif, sans);
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::SystemUi, sans);
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::UiSansSerif, sans);
        }
        if let Some(serif) = serif {
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::Serif, serif);
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::UiSerif, serif);
        }
        if let Some(mono) = mono {
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::Monospace, mono);
            let _ = parley_shaper.set_generic_family_name(ParleyGenericFamily::UiMonospace, mono);
        }

        Self {
            font_system,
            parley_shaper,
            parley_scale: parley::swash::scale::ScaleContext::new(),
            font_stack_key,
            font_db_revision,

            blobs: SlotMap::with_key(),
            blob_cache: HashMap::new(),
            blob_key_by_id: HashMap::new(),
            shape_cache: HashMap::new(),
            measure_cache: HashMap::new(),

            mask_atlas,
            color_atlas,
            subpixel_atlas,
            atlas_bind_group_layout,

            text_pin_mask: vec![Vec::new(); 3],
            text_pin_color: vec![Vec::new(); 3],
            text_pin_subpixel: vec![Vec::new(); 3],
            font_bytes_by_blob_id: HashMap::new(),
            font_face_key_by_fontique: HashMap::new(),
        }
    }

    pub fn set_font_families(&mut self, config: &TextFontFamilyConfig) -> bool {
        let installed = build_installed_family_set(self.font_system.db());
        let old_key = self.font_stack_key;
        let mut parley_changed = false;

        let pick =
            |overrides: &[String], defaults: &'static [&'static str]| -> Option<Cow<'_, str>> {
                for candidate in overrides {
                    if installed.contains(&candidate.to_ascii_lowercase()) {
                        return Some(Cow::Owned(candidate.clone()));
                    }
                }
                for &candidate in defaults {
                    if installed.contains(&candidate.to_ascii_lowercase()) {
                        return Some(Cow::Borrowed(candidate));
                    }
                }
                None
            };

        {
            let db = self.font_system.db_mut();

            if let Some(sans) = pick(&config.ui_sans, default_sans_candidates()) {
                db.set_sans_serif_family(sans.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::SansSerif, sans.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::SystemUi, sans.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::UiSansSerif, sans.as_ref());
            }
            if let Some(serif) = pick(&config.ui_serif, default_serif_candidates()) {
                db.set_serif_family(serif.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::Serif, serif.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::UiSerif, serif.as_ref());
            }
            if let Some(mono) = pick(&config.ui_mono, default_monospace_candidates()) {
                db.set_monospace_family(mono.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::Monospace, mono.as_ref());
                parley_changed |= self
                    .parley_shaper
                    .set_generic_family_name(ParleyGenericFamily::UiMonospace, mono.as_ref());
            }
        }

        let mut new_key = font_stack_cache_key(
            self.font_system.locale(),
            self.font_system.db(),
            self.font_db_revision,
        );
        if new_key == old_key && parley_changed {
            // Fontique generic family changes do not participate in the cosmic-text key, so we
            // bump the revision to ensure caches cannot reuse stale Parley shaping results.
            self.font_db_revision = self.font_db_revision.saturating_add(1);
            new_key = font_stack_cache_key(
                self.font_system.locale(),
                self.font_system.db(),
                self.font_db_revision,
            );
        }
        if new_key == old_key {
            return false;
        }

        self.font_stack_key = new_key;
        self.blobs.clear();
        self.blob_cache.clear();
        self.blob_key_by_id.clear();
        self.shape_cache.clear();
        self.measure_cache.clear();
        self.mask_atlas.reset();
        self.color_atlas.reset();
        self.subpixel_atlas.reset();
        self.text_pin_mask.iter_mut().for_each(|v| v.clear());
        self.text_pin_color.iter_mut().for_each(|v| v.clear());
        self.text_pin_subpixel.iter_mut().for_each(|v| v.clear());
        self.font_bytes_by_blob_id.clear();
        self.font_face_key_by_fontique.clear();
        true
    }

    pub fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.atlas_bind_group_layout
    }

    pub fn mask_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.mask_atlas.bind_group(page)
    }

    pub fn color_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.color_atlas.bind_group(page)
    }

    pub fn subpixel_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.subpixel_atlas.bind_group(page)
    }

    pub fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        self.mask_atlas.flush_uploads(queue);
        self.color_atlas.flush_uploads(queue);
        self.subpixel_atlas.flush_uploads(queue);
    }

    pub(crate) fn atlas_revision(&self) -> u64 {
        self.mask_atlas
            .revision()
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ self.color_atlas.revision().rotate_left(1)
            ^ self.subpixel_atlas.revision().rotate_left(2)
    }

    pub(crate) fn glyph_uv_for_instance(&self, glyph: &GlyphInstance) -> Option<(u16, [f32; 4])> {
        let (atlas, w, h) = match glyph.kind() {
            GlyphQuadKind::Mask => (
                &self.mask_atlas,
                self.mask_atlas.width as f32,
                self.mask_atlas.height as f32,
            ),
            GlyphQuadKind::Color => (
                &self.color_atlas,
                self.color_atlas.width as f32,
                self.color_atlas.height as f32,
            ),
            GlyphQuadKind::Subpixel => (
                &self.subpixel_atlas,
                self.subpixel_atlas.width as f32,
                self.subpixel_atlas.height as f32,
            ),
        };

        let entry = atlas.entry(glyph.key)?;
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let u0 = entry.x as f32 / w;
        let v0 = entry.y as f32 / h;
        let u1 = (entry.x.saturating_add(entry.w) as f32) / w;
        let v1 = (entry.y.saturating_add(entry.h) as f32) / h;
        Some((entry.page, [u0, v0, u1, v1]))
    }

    pub fn prepare_for_scene(&mut self, scene: &Scene, frame_index: u64) {
        let ring_len = self
            .text_pin_mask
            .len()
            .min(self.text_pin_color.len())
            .min(self.text_pin_subpixel.len());
        if ring_len == 0 {
            return;
        }
        let bucket = (frame_index as usize) % ring_len;

        let old_mask = std::mem::take(&mut self.text_pin_mask[bucket]);
        let old_color = std::mem::take(&mut self.text_pin_color[bucket]);
        let old_subpixel = std::mem::take(&mut self.text_pin_subpixel[bucket]);
        self.mask_atlas.dec_live_refs(&old_mask);
        self.color_atlas.dec_live_refs(&old_color);
        self.subpixel_atlas.dec_live_refs(&old_subpixel);

        let mut mask_keys: HashSet<GlyphKey> = HashSet::new();
        let mut color_keys: HashSet<GlyphKey> = HashSet::new();
        let mut subpixel_keys: HashSet<GlyphKey> = HashSet::new();

        for op in scene.ops() {
            let SceneOp::Text { text, .. } = *op else {
                continue;
            };
            let Some(blob) = self.blobs.get(text) else {
                continue;
            };
            for glyph in blob.shape.glyphs.as_ref() {
                match glyph.kind() {
                    GlyphQuadKind::Mask => {
                        mask_keys.insert(glyph.key);
                    }
                    GlyphQuadKind::Color => {
                        color_keys.insert(glyph.key);
                    }
                    GlyphQuadKind::Subpixel => {
                        subpixel_keys.insert(glyph.key);
                    }
                }
            }
        }

        let epoch = frame_index;
        let mut new_mask: Vec<GlyphKey> = mask_keys.into_iter().collect();
        let mut new_color: Vec<GlyphKey> = color_keys.into_iter().collect();
        let mut new_subpixel: Vec<GlyphKey> = subpixel_keys.into_iter().collect();

        for &key in &new_mask {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_color {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_subpixel {
            self.ensure_glyph_in_atlas(key, epoch);
        }

        self.mask_atlas.inc_live_refs(&new_mask);
        self.color_atlas.inc_live_refs(&new_color);
        self.subpixel_atlas.inc_live_refs(&new_subpixel);

        self.text_pin_mask[bucket].append(&mut new_mask);
        self.text_pin_color[bucket].append(&mut new_color);
        self.text_pin_subpixel[bucket].append(&mut new_subpixel);
    }

    fn ensure_glyph_in_atlas(&mut self, key: GlyphKey, epoch: u64) {
        let already_present = match key.kind {
            GlyphQuadKind::Mask => self.mask_atlas.get(key, epoch).is_some(),
            GlyphQuadKind::Color => self.color_atlas.get(key, epoch).is_some(),
            GlyphQuadKind::Subpixel => self.subpixel_atlas.get(key, epoch).is_some(),
        };
        if already_present {
            return;
        }

        self.ensure_parley_glyph(key, epoch);
    }

    fn ensure_parley_glyph(&mut self, key: GlyphKey, epoch: u64) {
        let Some(font_bytes) = self.font_bytes_by_blob_id.get(&key.font.blob_id) else {
            return;
        };

        let Some(font_ref) =
            parley::swash::FontRef::from_index(font_bytes.as_ref(), key.font.face_index as usize)
        else {
            return;
        };
        let Ok(glyph_id) = u16::try_from(key.glyph_id) else {
            return;
        };

        let font_size = f32::from_bits(key.size_bits).max(1.0);
        let mut scaler = self
            .parley_scale
            .builder(font_ref)
            .size(font_size)
            .hint(false)
            .build();

        let offset_px = parley::swash::zeno::Vector::new(
            subpixel_bin_as_float(key.x_bin),
            subpixel_bin_as_float(key.y_bin),
        );
        let Some(image) = parley::swash::scale::Render::new(&[
            parley::swash::scale::Source::ColorOutline(0),
            parley::swash::scale::Source::ColorBitmap(parley::swash::scale::StrikeWith::BestFit),
            parley::swash::scale::Source::Outline,
        ])
        .offset(offset_px)
        .render(&mut scaler, glyph_id) else {
            return;
        };
        if image.placement.width == 0 || image.placement.height == 0 {
            return;
        }

        let (image_kind, bytes_per_pixel) = match image.content {
            parley::swash::scale::image::Content::Mask => (GlyphQuadKind::Mask, 1),
            parley::swash::scale::image::Content::Color => (GlyphQuadKind::Color, 4),
            parley::swash::scale::image::Content::SubpixelMask => (GlyphQuadKind::Subpixel, 4),
        };
        if image_kind != key.kind {
            return;
        }

        let data = image.data;

        match key.kind {
            GlyphQuadKind::Mask => {
                let _ = self.mask_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
            GlyphQuadKind::Color => {
                let _ = self.color_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
            GlyphQuadKind::Subpixel => {
                let _ = self.subpixel_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
        }
    }

    pub fn blob(&self, id: TextBlobId) -> Option<&TextBlob> {
        self.blobs.get(id)
    }

    pub fn prepare_input(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        match input {
            TextInputRef::Plain { text, style } => self.prepare(text, style, constraints),
            TextInputRef::Attributed { text, base, spans } => {
                let rich =
                    AttributedText::new(Arc::<str>::from(text), Arc::<[TextSpan]>::from(spans));
                self.prepare_attributed(&rich, base, constraints)
            }
        }
    }

    pub fn prepare(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let key = TextBlobKey::new(text, style, constraints, self.font_stack_key);
        self.prepare_with_key(key, style, None, constraints)
    }

    pub fn prepare_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let key = TextBlobKey::new_attributed(rich, base_style, constraints, self.font_stack_key);
        self.prepare_with_key(key, base_style, Some(rich.spans.as_ref()), constraints)
    }

    fn prepare_with_key(
        &mut self,
        mut key: TextBlobKey,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let text = key.text.clone();
        key.backend = 1;

        if let Some(id) = self.blob_cache.get(&key).copied() {
            if let Some(blob) = self.blobs.get_mut(id) {
                blob.ref_count = blob.ref_count.saturating_add(1);
                return (id, blob.shape.metrics);
            }
            // Stale cache entry (shouldn't happen, but keep it robust).
            self.blob_cache.remove(&key);
            self.blob_key_by_id.remove(&id);
        }

        let resolved_spans = spans.and_then(|spans| resolve_spans_for_text(text.as_ref(), spans));
        let paint_palette = resolved_spans.as_ref().map(|spans| {
            let mut palette: Vec<Option<fret_core::Color>> = Vec::with_capacity(spans.len());
            palette.extend(spans.iter().map(|s| s.fg));
            Arc::<[Option<fret_core::Color>]>::from(palette)
        });

        let shape_key = TextShapeKey::from_blob_key(&key);
        let shape = if let Some(shape) = self.shape_cache.get(&shape_key) {
            shape.clone()
        } else {
            let scale = constraints.scale_factor.max(1.0);
            let shape = {
                let input = match spans {
                    Some(spans) => TextInputRef::Attributed {
                        text: text.as_ref(),
                        base: style,
                        spans,
                    },
                    None => TextInputRef::Plain {
                        text: text.as_ref(),
                        style,
                    },
                };
                let wrapped = crate::text_v2::wrapper::wrap_with_constraints(
                    &mut self.parley_shaper,
                    input,
                    constraints,
                );
                let kept_end = wrapped.kept_end;

                let first_baseline_px = wrapped
                    .lines
                    .first()
                    .map(|l| l.baseline.max(0.0))
                    .unwrap_or(0.0);

                let metrics = metrics_from_wrapped_lines(&wrapped.lines, scale);

                let mut glyphs: Vec<GlyphInstance> = Vec::new();
                let mut lines: Vec<TextLine> = Vec::with_capacity(wrapped.lines.len().max(1));
                let mut first_line_caret_stops: Vec<(usize, Px)> = Vec::new();

                let mut line_top_px = 0.0_f32;

                for (i, (range, line)) in wrapped
                    .line_ranges
                    .iter()
                    .cloned()
                    .zip(wrapped.lines.into_iter())
                    .enumerate()
                {
                    let line_height_px = line.line_height.max(0.0);
                    let line_baseline_px = line.baseline.max(0.0);
                    let line_offset_px = (line_top_px + line_baseline_px) - first_baseline_px;

                    let slice = &text[range.clone()];
                    let caret_stops = caret_stops_for_slice(
                        slice,
                        range.start,
                        &line.clusters,
                        line.width.max(0.0),
                        scale,
                        kept_end,
                    );
                    if i == 0 {
                        first_line_caret_stops = caret_stops.clone();
                    }

                    lines.push(TextLine {
                        start: range.start,
                        end: range.end.min(kept_end),
                        width: Px((line.width / scale).max(0.0)),
                        y_top: Px((line_top_px / scale).max(0.0)),
                        height: Px((line_height_px / scale).max(0.0)),
                        caret_stops,
                    });

                    for g in line.glyphs {
                        let Ok(glyph_id) = u16::try_from(g.id) else {
                            continue;
                        };
                        let fontique_id = g.font.data.id();
                        let face_index = g.font.index;
                        let face_key = if let Some(hit) = self
                            .font_face_key_by_fontique
                            .get(&(fontique_id, face_index))
                            .copied()
                        {
                            hit
                        } else {
                            let bytes = g.font.data.data();
                            let blob_id = stable_font_blob_id(bytes);
                            self.font_bytes_by_blob_id
                                .entry(blob_id)
                                .or_insert_with(|| Arc::from(bytes.to_vec()));
                            let key = FontFaceKey {
                                blob_id,
                                face_index,
                                variation_key: 0,
                            };
                            self.font_face_key_by_fontique
                                .insert((fontique_id, face_index), key);
                            key
                        };
                        let Some(font_ref) = parley::swash::FontRef::from_index(
                            g.font.data.data(),
                            g.font.index as usize,
                        ) else {
                            continue;
                        };

                        let mut scaler = self
                            .parley_scale
                            .builder(font_ref)
                            .size(g.font_size.max(1.0))
                            .hint(false)
                            .build();

                        let pos_y = g.y + line_offset_px;
                        let (x, x_bin) = subpixel_bin_q4(g.x);
                        let (y, y_bin) = subpixel_bin_y(pos_y);
                        let offset_px = parley::swash::zeno::Vector::new(
                            subpixel_bin_as_float(x_bin),
                            subpixel_bin_as_float(y_bin),
                        );

                        let Some(image) = parley::swash::scale::Render::new(&[
                            parley::swash::scale::Source::ColorOutline(0),
                            parley::swash::scale::Source::ColorBitmap(
                                parley::swash::scale::StrikeWith::BestFit,
                            ),
                            parley::swash::scale::Source::Outline,
                        ])
                        .offset(offset_px)
                        .render(&mut scaler, glyph_id) else {
                            continue;
                        };

                        if image.placement.width == 0 || image.placement.height == 0 {
                            continue;
                        }

                        let kind = match image.content {
                            parley::swash::scale::image::Content::Mask => GlyphQuadKind::Mask,
                            parley::swash::scale::image::Content::Color => GlyphQuadKind::Color,
                            parley::swash::scale::image::Content::SubpixelMask => {
                                GlyphQuadKind::Subpixel
                            }
                        };

                        let glyph_key = GlyphKey {
                            font: face_key,
                            glyph_id: g.id,
                            size_bits: g.font_size.to_bits(),
                            x_bin,
                            y_bin,
                            kind,
                        };

                        let x0_px = x as f32 + image.placement.left as f32;
                        let y0_px = y as f32 - image.placement.top as f32;
                        let w_px = image.placement.width as f32;
                        let h_px = image.placement.height as f32;

                        let text_range =
                            (range.start + g.text_range.start)..(range.start + g.text_range.end);
                        let paint_span = resolved_spans
                            .as_deref()
                            .and_then(|spans| paint_span_for_text_range(spans, &text_range));

                        glyphs.push(GlyphInstance {
                            rect: [x0_px / scale, y0_px / scale, w_px / scale, h_px / scale],
                            paint_span,
                            key: glyph_key,
                        });
                    }

                    line_top_px += line_height_px;
                }

                Arc::new(TextShape {
                    glyphs: Arc::from(glyphs),
                    metrics,
                    lines: Arc::from(lines),
                    caret_stops: Arc::from(first_line_caret_stops),
                })
            };
            self.shape_cache.insert(shape_key.clone(), shape.clone());
            shape
        };

        let metrics = shape.metrics;
        let id = self.blobs.insert(TextBlob {
            shape,
            paint_palette,
            ref_count: 1,
        });
        self.blob_cache.insert(key.clone(), id);
        self.blob_key_by_id.insert(id, key);
        (id, metrics)
    }

    pub fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        const MEASURE_CACHE_PER_BUCKET_LIMIT: usize = 256;

        let key = TextMeasureKey::new(style, constraints, self.font_stack_key);
        let text_hash = hash_text(text);
        if let Some(bucket) = self.measure_cache.get_mut(&key)
            && let Some(hit) = bucket
                .iter()
                .find(|e| e.text_hash == text_hash && e.spans_hash == 0 && e.text.as_ref() == text)
        {
            return hit.metrics;
        }

        let scale = constraints.scale_factor.max(1.0);
        let wrapped = crate::text_v2::wrapper::wrap_with_constraints_measure_only(
            &mut self.parley_shaper,
            TextInputRef::plain(text, style),
            constraints,
        );
        let metrics = metrics_from_wrapped_lines(&wrapped.lines, scale);

        let bucket = self.measure_cache.entry(key).or_default();
        bucket.push_back(TextMeasureEntry {
            text_hash,
            spans_hash: 0,
            text: Arc::<str>::from(text),
            spans: None,
            metrics,
        });
        while bucket.len() > MEASURE_CACHE_PER_BUCKET_LIMIT {
            bucket.pop_front();
        }

        metrics
    }

    pub fn measure_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        const MEASURE_CACHE_PER_BUCKET_LIMIT: usize = 256;

        let key = TextMeasureKey::new(base_style, constraints, self.font_stack_key);
        let text_hash = hash_text(rich.text.as_ref());
        let spans_hash = spans_shaping_fingerprint(rich.spans.as_ref());

        if let Some(bucket) = self.measure_cache.get_mut(&key)
            && let Some(hit) = bucket.iter().find(|e| {
                e.text_hash == text_hash
                    && e.spans_hash == spans_hash
                    && e.text.as_ref() == rich.text.as_ref()
                    && e.spans.as_ref().is_some_and(|s| {
                        Arc::ptr_eq(s, &rich.spans) || s.as_ref() == rich.spans.as_ref()
                    })
            })
        {
            return hit.metrics;
        }

        let scale = constraints.scale_factor.max(1.0);
        let wrapped = crate::text_v2::wrapper::wrap_with_constraints_measure_only(
            &mut self.parley_shaper,
            TextInputRef::Attributed {
                text: rich.text.as_ref(),
                base: base_style,
                spans: rich.spans.as_ref(),
            },
            constraints,
        );
        let metrics = metrics_from_wrapped_lines(&wrapped.lines, scale);

        let bucket = self.measure_cache.entry(key).or_default();
        bucket.push_back(TextMeasureEntry {
            text_hash,
            spans_hash,
            text: rich.text.clone(),
            spans: Some(rich.spans.clone()),
            metrics,
        });
        while bucket.len() > MEASURE_CACHE_PER_BUCKET_LIMIT {
            bucket.pop_front();
        }

        metrics
    }

    pub fn caret_x(&self, blob: TextBlobId, index: usize) -> Option<Px> {
        let blob_id = blob;
        let blob = self.blobs.get(blob_id)?;
        if blob.shape.lines.len() > 1 {
            return Some(
                self.caret_rect(blob_id, index, CaretAffinity::Downstream)?
                    .origin
                    .x,
            );
        }
        let stops = blob.shape.caret_stops.as_ref();
        if stops.is_empty() {
            return Some(Px(0.0));
        }
        if let Some((_, x)) = stops.iter().find(|(i, _)| *i == index) {
            return Some(*x);
        }
        let mut last = Px(0.0);
        for (i, x) in stops {
            if *i > index {
                break;
            }
            last = *x;
        }
        Some(last)
    }

    pub fn hit_test_x(&self, blob: TextBlobId, x: Px) -> Option<usize> {
        let blob_id = blob;
        let blob = self.blobs.get(blob_id)?;
        if blob.shape.lines.len() > 1 {
            return Some(self.hit_test_point(blob_id, Point::new(x, Px(0.0)))?.index);
        }
        let stops = blob.shape.caret_stops.as_ref();
        if stops.is_empty() {
            return Some(0);
        }
        let mut best = stops[0].0;
        let mut best_dist = (stops[0].1.0 - x.0).abs();
        for (idx, px) in stops {
            let dist = (px.0 - x.0).abs();
            if dist < best_dist {
                best = *idx;
                best_dist = dist;
            }
        }
        Some(best)
    }

    pub fn caret_stops(&self, blob: TextBlobId) -> Option<&[(usize, Px)]> {
        Some(self.blobs.get(blob)?.shape.caret_stops.as_ref())
    }

    pub fn caret_rect(
        &self,
        blob: TextBlobId,
        index: usize,
        affinity: CaretAffinity,
    ) -> Option<Rect> {
        let blob = self.blobs.get(blob)?;
        caret_rect_from_lines(blob.shape.lines.as_ref(), index, affinity)
    }

    pub fn hit_test_point(&self, blob: TextBlobId, point: Point) -> Option<HitTestResult> {
        let blob = self.blobs.get(blob)?;
        hit_test_point_from_lines(blob.shape.lines.as_ref(), point)
    }

    pub fn selection_rects(
        &self,
        blob: TextBlobId,
        range: (usize, usize),
        out: &mut Vec<Rect>,
    ) -> Option<()> {
        let blob = self.blobs.get(blob)?;
        selection_rects_from_lines(blob.shape.lines.as_ref(), range, out);
        Some(())
    }

    pub fn release(&mut self, blob: TextBlobId) {
        let (should_remove, remove_shape) = match self.blobs.get_mut(blob) {
            Some(b) => {
                if b.ref_count > 1 {
                    b.ref_count = b.ref_count.saturating_sub(1);
                    (false, false)
                } else {
                    let remove_shape = Arc::strong_count(&b.shape) == 2;
                    (true, remove_shape)
                }
            }
            None => return,
        };

        if !should_remove {
            return;
        }

        if let Some(key) = self.blob_key_by_id.remove(&blob) {
            self.blob_cache.remove(&key);
            if remove_shape {
                let shape_key = TextShapeKey::from_blob_key(&key);
                self.shape_cache.remove(&shape_key);
            }
        }
        let _ = self.blobs.remove(blob);
    }
}

#[cfg(any())]
#[derive(Debug, Clone)]
struct PreparedLayout {
    metrics: TextMetrics,
    lines: Vec<cosmic_text::LayoutLine>,
    line_tops_px: Vec<f32>,
    local_starts: Vec<usize>,
    local_ends: Vec<usize>,
    paragraph_ends: Vec<usize>,
}

#[derive(Clone, Debug)]
struct ResolvedSpan {
    start: usize,
    end: usize,
    slot: u16,
    fg: Option<fret_core::Color>,
}

fn resolve_spans_for_text(text: &str, spans: &[TextSpan]) -> Option<Vec<ResolvedSpan>> {
    if spans.is_empty() {
        return None;
    }

    let mut out: Vec<ResolvedSpan> = Vec::with_capacity(spans.len());
    let mut offset: usize = 0;
    for span in spans {
        let end = offset.saturating_add(span.len);
        if end > text.len() {
            return None;
        }
        if !text.is_char_boundary(offset) || !text.is_char_boundary(end) {
            return None;
        }
        if span.len != 0 {
            let slot = u16::try_from(out.len()).ok()?;
            out.push(ResolvedSpan {
                start: offset,
                end,
                slot,
                fg: span.paint.fg,
            });
        }
        offset = end;
    }
    if offset != text.len() {
        return None;
    }

    Some(out)
}

#[cfg(any())]
fn paint_span_for_glyph(
    spans: &[ResolvedSpan],
    base_offset: usize,
    g: &cosmic_text::LayoutGlyph,
) -> Option<u16> {
    let mut global = base_offset.saturating_add(g.start);
    if g.start == g.end && global > 0 {
        global = global.saturating_sub(1);
    }
    spans
        .iter()
        .find(|s| global >= s.start && global < s.end)
        .map(|s| s.slot)
}

fn paint_span_for_text_range(
    spans: &[ResolvedSpan],
    range: &std::ops::Range<usize>,
) -> Option<u16> {
    let mut idx = range.start;
    if range.start == range.end && idx > 0 {
        idx = idx.saturating_sub(1);
    }
    spans
        .iter()
        .find(|s| idx >= s.start && idx < s.end)
        .map(|s| s.slot)
}

#[cfg(any())]
fn layout_text(
    font_system: &mut FontSystem,
    scratch: &mut ShapeBuffer,
    text: &str,
    attrs: &Attrs,
    spans: Option<&[TextSpan]>,
    font_size_px: f32,
    constraints: TextConstraints,
    scale: f32,
) -> (PreparedLayout, Vec<usize>) {
    let max_width_px = constraints.max_width.map(|w| w.0 * scale);
    let wrap = match constraints.wrap {
        TextWrap::None => cosmic_text::Wrap::None,
        TextWrap::Word => cosmic_text::Wrap::Word,
    };

    let want_ellipsis = matches!(constraints.overflow, TextOverflow::Ellipsis)
        && matches!(constraints.wrap, TextWrap::None)
        && max_width_px.is_some();

    let mut all_lines: Vec<cosmic_text::LayoutLine> = Vec::new();
    let mut line_tops_px: Vec<f32> = Vec::new();
    let mut local_starts: Vec<usize> = Vec::new();
    let mut local_ends: Vec<usize> = Vec::new();
    let mut paragraph_ends: Vec<usize> = Vec::new();
    let mut line_starts_global: Vec<usize> = Vec::new();

    let mut max_w_px = 0.0_f32;
    let mut total_h_px = 0.0_f32;
    let mut first_ascent_px: Option<f32> = None;

    let resolved_spans: Option<Vec<ResolvedSpan>> =
        spans.and_then(|spans| resolve_spans_for_text(text, spans));

    let mut push_slice = |base_offset: usize, slice: &str, paragraph_end: usize| {
        let mut attrs_list = AttrsList::new(attrs);
        attrs_list.add_span(0..slice.len(), attrs);

        if let Some(spans) = resolved_spans.as_ref() {
            for span in spans {
                if span.end <= base_offset || span.start >= paragraph_end {
                    continue;
                }

                let start = span.start.max(base_offset) - base_offset;
                let end = span.end.min(paragraph_end) - base_offset;
                if start >= end || end > slice.len() {
                    continue;
                }

                let mut span_attrs = attrs.clone();
                if let Some(font) = span.font.as_ref() {
                    span_attrs = span_attrs.family(family_for_font_id(font));
                }
                if let Some(weight) = span.weight {
                    span_attrs = span_attrs.weight(Weight(weight.0));
                }
                if let Some(slant) = span.slant {
                    span_attrs = match slant {
                        TextSlant::Normal => span_attrs.style(CosmicStyle::Normal),
                        TextSlant::Italic => span_attrs.style(CosmicStyle::Italic),
                        TextSlant::Oblique => span_attrs.style(CosmicStyle::Oblique),
                    };
                }
                if let Some(letter_spacing_em) = span.letter_spacing_em
                    && letter_spacing_em != 0.0
                    && letter_spacing_em.is_finite()
                {
                    span_attrs = span_attrs.letter_spacing(letter_spacing_em);
                }
                attrs_list.add_span(start..end, &span_attrs);
            }
        }

        let shape_line = ShapeLine::new(font_system, slice, &attrs_list, Shaping::Advanced, 4);
        let mut layout_lines: Vec<cosmic_text::LayoutLine> = Vec::new();
        shape_line.layout_to_buffer(
            scratch,
            font_size_px,
            max_width_px,
            wrap,
            None,
            &mut layout_lines,
            None,
            Hinting::Disabled,
        );

        let mut ellipsis_local_end: Option<usize> = None;
        if want_ellipsis
            && layout_lines.len() == 1
            && let Some(max_w) = max_width_px
            && let Some(line) = layout_lines.get_mut(0)
            // Avoid spurious ellipses caused by subpixel layout rounding (especially visible in
            // list rows where the remaining gap makes the truncation look "wrong").
            && line.w > max_w + 0.5
        {
            let ellipsis_text = "…";
            let (ellipsis_w, ellipsis_glyphs) = {
                let mut ellipsis_attrs_list = AttrsList::new(attrs);
                ellipsis_attrs_list.add_span(0..ellipsis_text.len(), attrs);
                let ellipsis_shape = ShapeLine::new(
                    font_system,
                    ellipsis_text,
                    &ellipsis_attrs_list,
                    Shaping::Advanced,
                    4,
                );
                let mut ellipsis_lines: Vec<cosmic_text::LayoutLine> = Vec::new();
                ellipsis_shape.layout_to_buffer(
                    scratch,
                    font_size_px,
                    None,
                    cosmic_text::Wrap::None,
                    None,
                    &mut ellipsis_lines,
                    None,
                    Hinting::Disabled,
                );
                let w = ellipsis_lines.first().map(|l| l.w).unwrap_or(0.0);
                let glyphs = ellipsis_lines
                    .first()
                    .map(|l| l.glyphs.clone())
                    .unwrap_or_default();
                (w, glyphs)
            };

            let available_w = (max_w - ellipsis_w).max(0.0);
            let mut cut_end = 0usize;
            for g in &line.glyphs {
                let right = (g.x + g.w).max(0.0);
                if right <= available_w + 0.5 {
                    cut_end = cut_end.max(g.end.min(slice.len()));
                }
            }
            while cut_end > 0
                && slice
                    .as_bytes()
                    .get(cut_end.saturating_sub(1))
                    .is_some_and(|b| b.is_ascii_whitespace())
            {
                cut_end = cut_end.saturating_sub(1);
            }

            let mut kept: Vec<cosmic_text::LayoutGlyph> = line
                .glyphs
                .iter()
                .filter(|&g| g.end <= cut_end)
                .cloned()
                .collect();

            let ellipsis_start_x = (max_w - ellipsis_w).max(0.0);
            for mut g in ellipsis_glyphs {
                g.start = cut_end;
                g.end = cut_end;
                g.x = (g.x + ellipsis_start_x).max(0.0);
                kept.push(g);
            }
            line.glyphs = kept;
            line.w = max_w;
            ellipsis_local_end = Some(cut_end);
        }

        if layout_lines.is_empty() {
            layout_lines.push(cosmic_text::LayoutLine {
                w: 0.0,
                max_ascent: 0.0,
                max_descent: 0.0,
                line_height_opt: None,
                glyphs: Vec::new(),
            });
        }

        let layout_count = layout_lines.len();
        let mut expected_start_local: usize = 0;

        for (idx, ll) in layout_lines.into_iter().enumerate() {
            let mut local_end = ll
                .glyphs
                .iter()
                .map(|g| g.end)
                .max()
                .unwrap_or(expected_start_local);
            if idx + 1 == layout_count {
                local_end = slice.len();
            }
            if idx + 1 == layout_count
                && let Some(end) = ellipsis_local_end
            {
                local_end = end.min(slice.len());
            }

            let local_start = expected_start_local;
            expected_start_local = local_end;

            let ascent_px = ll.max_ascent.max(0.0);
            let descent_px = ll.max_descent.max(0.0);
            let min_height_px = (ascent_px + descent_px).max(0.0);
            let height_px = ll
                .line_height_opt
                .unwrap_or(min_height_px)
                .max(min_height_px)
                .max(0.0);

            // Center the baseline within the line box when line-height exceeds the font's
            // ascent+descent. This avoids visible "text floats up" artifacts when swapping fonts
            // (e.g. Nerd Fonts with unusual metrics) while keeping behavior unchanged when the
            // line box is tight.
            let padding_top_px = ((height_px - ascent_px - descent_px) * 0.5).max(0.0);
            let baseline_offset_px = padding_top_px + ascent_px;
            first_ascent_px.get_or_insert(baseline_offset_px);
            max_w_px = max_w_px.max(ll.w);

            line_tops_px.push(total_h_px);
            local_starts.push(local_start);
            local_ends.push(local_end);
            paragraph_ends.push(paragraph_end);
            line_starts_global.push(base_offset);

            total_h_px += height_px;
            all_lines.push(ll);
        }
    };

    let mut slice_start = 0usize;
    for (i, ch) in text.char_indices() {
        if ch != '\n' {
            continue;
        }
        push_slice(slice_start, &text[slice_start..i], i);
        slice_start = i + 1;
    }
    push_slice(slice_start, &text[slice_start..text.len()], text.len());

    let first_ascent_px = first_ascent_px.unwrap_or(0.0);
    let metrics = TextMetrics {
        size: Size::new(Px(max_w_px / scale), Px(total_h_px / scale)),
        baseline: Px(first_ascent_px / scale),
    };

    (
        PreparedLayout {
            metrics,
            lines: all_lines,
            line_tops_px,
            local_starts,
            local_ends,
            paragraph_ends,
        },
        line_starts_global,
    )
}

fn utf8_char_boundaries(text: &str) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(text.chars().count().saturating_add(2));
    out.push(0);
    for (i, _) in text.char_indices() {
        out.push(i);
    }
    out.push(text.len());
    out.sort_unstable();
    out.dedup();
    out
}

fn caret_stops_for_slice(
    slice: &str,
    base_offset: usize,
    clusters: &[crate::text_v2::parley_shaper::ShapedCluster],
    line_width_px: f32,
    scale: f32,
    kept_end: usize,
) -> Vec<(usize, Px)> {
    let mut out: Vec<(usize, Px)> = Vec::new();
    let boundaries = utf8_char_boundaries(slice);

    if boundaries.is_empty() {
        return vec![(base_offset, Px(0.0))];
    }

    if clusters.is_empty() {
        for &b in &boundaries {
            let idx = base_offset + b;
            if idx > kept_end {
                continue;
            }
            let x = if b >= slice.len() {
                (line_width_px / scale).max(0.0)
            } else {
                0.0
            };
            out.push((idx, Px(x)));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
        out.dedup_by(|a, b| a.0 == b.0);
        return out;
    }

    let mut cluster_i = 0usize;
    for &b in &boundaries {
        let idx = base_offset + b;
        if idx > kept_end {
            continue;
        }

        while cluster_i + 1 < clusters.len() && clusters[cluster_i].text_range.end < b {
            cluster_i = cluster_i.saturating_add(1);
        }

        let x = if b <= clusters[0].text_range.start {
            0.0
        } else if cluster_i >= clusters.len() {
            line_width_px.max(0.0)
        } else {
            let c = &clusters[cluster_i];
            let start = c.text_range.start.min(slice.len());
            let end = c.text_range.end.min(slice.len());

            if start == end {
                c.x0.max(0.0)
            } else if b <= start {
                c.x0.max(0.0)
            } else if b >= end {
                c.x1.max(0.0)
            } else {
                let denom = (end - start) as f32;
                let mut t = ((b - start) as f32 / denom).clamp(0.0, 1.0);
                if c.is_rtl {
                    t = 1.0 - t;
                }
                let w = (c.x1 - c.x0).max(0.0);
                (c.x0 + w * t).max(0.0)
            }
        };

        out.push((idx, Px((x / scale).max(0.0))));
    }

    out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

#[cfg(any())]
fn build_line_caret_stops(
    base_offset: usize,
    boundaries_local: &[usize],
    glyphs: &[cosmic_text::LayoutGlyph],
    local_start: usize,
    local_end: usize,
    line_w_px: f32,
    scale: f32,
) -> Vec<(usize, Px)> {
    let mut out: Vec<(usize, Px)> = Vec::with_capacity(boundaries_local.len());
    for &idx_local in boundaries_local {
        let idx_global = base_offset + idx_local;
        if idx_local <= local_start {
            out.push((idx_global, Px(0.0)));
            continue;
        }
        if idx_local >= local_end {
            out.push((idx_global, Px(line_w_px / scale)));
            continue;
        }

        let mut x_end = 0.0_f32;
        for g in glyphs {
            if g.end <= idx_local {
                x_end = x_end.max(g.x + g.w);
            }
        }
        out.push((idx_global, Px(x_end / scale)));
    }
    out.sort_by_key(|(idx, _)| *idx);
    out.dedup_by_key(|(idx, _)| *idx);
    out
}

fn caret_x_from_stops(stops: &[(usize, Px)], index: usize) -> Px {
    if stops.is_empty() {
        return Px(0.0);
    }
    if let Ok(pos) = stops.binary_search_by_key(&index, |(idx, _)| *idx) {
        return stops[pos].1;
    }
    match stops.partition_point(|(idx, _)| *idx <= index) {
        0 => stops[0].1,
        n => stops[n.saturating_sub(1)].1,
    }
}

fn hit_test_x_from_stops(stops: &[(usize, Px)], x: Px) -> usize {
    if stops.is_empty() {
        return 0;
    }
    let mut best = stops[0].0;
    let mut best_dist = (stops[0].1.0 - x.0).abs();
    for (idx, px) in stops {
        let dist = (px.0 - x.0).abs();
        if dist < best_dist {
            best = *idx;
            best_dist = dist;
        }
    }
    best
}

fn caret_rect_from_lines(
    lines: &[TextLine],
    index: usize,
    affinity: CaretAffinity,
) -> Option<Rect> {
    if lines.is_empty() {
        return None;
    }

    let mut candidates: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if index >= line.start && index <= line.end {
            candidates.push(i);
        }
    }

    let line_idx = match candidates.as_slice() {
        [] => {
            if index <= lines[0].start {
                0
            } else {
                lines.len().saturating_sub(1)
            }
        }
        [only] => *only,
        many => match affinity {
            CaretAffinity::Upstream => many[0],
            CaretAffinity::Downstream => many[many.len().saturating_sub(1)],
        },
    };

    let line = &lines[line_idx];
    let x = caret_x_from_stops(&line.caret_stops, index);
    Some(Rect::new(
        Point::new(x, line.y_top),
        Size::new(Px(1.0), line.height),
    ))
}

fn hit_test_point_from_lines(lines: &[TextLine], point: Point) -> Option<HitTestResult> {
    if lines.is_empty() {
        return None;
    }

    let mut line_idx = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let y0 = line.y_top.0;
        let y1 = (line.y_top.0 + line.height.0).max(y0);
        if point.y.0 >= y0 && point.y.0 < y1 {
            line_idx = i;
            break;
        }
        if point.y.0 >= y1 {
            line_idx = i;
        }
    }

    let line = &lines[line_idx];
    let index = hit_test_x_from_stops(&line.caret_stops, point.x);

    let mut affinity = CaretAffinity::Downstream;
    if line_idx + 1 < lines.len() && index == line.end && lines[line_idx + 1].start == index {
        affinity = CaretAffinity::Upstream;
    }

    Some(HitTestResult { index, affinity })
}

fn selection_rects_from_lines(lines: &[TextLine], range: (usize, usize), out: &mut Vec<Rect>) {
    out.clear();
    if lines.is_empty() {
        return;
    }

    let (a, b) = (range.0.min(range.1), range.0.max(range.1));
    if a == b {
        return;
    }

    for line in lines {
        let start = a.max(line.start);
        let end = b.min(line.end);
        if start >= end {
            continue;
        }

        let x0 = if start <= line.start {
            Px(0.0)
        } else {
            caret_x_from_stops(&line.caret_stops, start)
        };
        let x1 = if end >= line.end {
            line.width
        } else {
            caret_x_from_stops(&line.caret_stops, end)
        };

        out.push(Rect::new(
            Point::new(x0, line.y_top),
            Size::new(Px((x1.0 - x0.0).max(0.0)), line.height),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TextBlobKey, TextShapeKey, collect_font_names, spans_paint_fingerprint,
        spans_shaping_fingerprint, subpixel_mask_to_alpha,
    };
    use cosmic_text::Family;
    use fret_core::{
        Color, FontWeight, Px, TextConstraints, TextInputRef, TextOverflow, TextSpan, TextStyle,
        TextWrap,
    };
    use std::sync::Arc;

    #[test]
    fn subpixel_mask_to_alpha_uses_channel_max() {
        let data = vec![
            10u8, 3u8, 4u8, 0u8, //
            1u8, 200u8, 2u8, 0u8,
        ];
        assert_eq!(subpixel_mask_to_alpha(&data), vec![10u8, 200u8]);
    }

    #[test]
    fn caret_stops_for_slice_interpolates_within_cluster_ltr() {
        let clusters = vec![crate::text_v2::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: false,
        }];

        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let x_at = |i: usize| stops.iter().find(|(idx, _)| *idx == i).unwrap().1.0;

        assert_eq!(x_at(0), 0.0);
        assert_eq!(x_at(1), 10.0);
        assert_eq!(x_at(2), 20.0);
        assert_eq!(x_at(3), 30.0);
        assert_eq!(x_at(4), 40.0);
    }

    #[test]
    fn caret_stops_for_slice_interpolates_within_cluster_rtl() {
        let clusters = vec![crate::text_v2::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: true,
        }];

        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let x_at = |i: usize| stops.iter().find(|(idx, _)| *idx == i).unwrap().1.0;

        assert_eq!(x_at(0), 0.0);
        assert_eq!(x_at(1), 30.0);
        assert_eq!(x_at(2), 20.0);
        assert_eq!(x_at(3), 10.0);
        assert_eq!(x_at(4), 40.0);
    }

    #[test]
    fn all_font_names_is_sorted_and_deduped() {
        // This is intentionally platform-dependent; we only assert structural invariants.
        let (locale, db) = cosmic_text::FontSystem::new().into_locale_and_db();
        let _ = locale;

        let names = collect_font_names(&db);

        assert!(
            names
                .iter()
                .any(|n| n == db.family_name(&Family::SansSerif)),
            "expected sans-serif generic family to be present"
        );
        assert!(
            names.iter().any(|n| n == db.family_name(&Family::Serif)),
            "expected serif generic family to be present"
        );
        assert!(
            names
                .iter()
                .any(|n| n == db.family_name(&Family::Monospace)),
            "expected monospace generic family to be present"
        );

        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for name in &names {
            assert!(
                seen.insert(name.to_ascii_lowercase()),
                "expected case-insensitive dedupe for {name:?}"
            );
        }

        for w in names.windows(2) {
            assert!(
                w[0].to_ascii_lowercase() <= w[1].to_ascii_lowercase(),
                "expected case-insensitive sort"
            );
        }
    }

    #[test]
    fn text_blob_key_includes_typography_fields() {
        let constraints = TextConstraints {
            max_width: Some(Px(120.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 2.0,
        };

        let base = TextStyle::default();
        let k0 = TextBlobKey::new("hello", &base, constraints, 1);

        let mut style = base.clone();
        style.weight = FontWeight::BOLD;
        let k_weight = TextBlobKey::new("hello", &style, constraints, 1);
        assert_ne!(k0, k_weight);

        let mut style = base.clone();
        style.line_height = Some(Px(18.0));
        let k_line_height = TextBlobKey::new("hello", &style, constraints, 1);
        assert_ne!(k0, k_line_height);

        let mut style = base.clone();
        style.letter_spacing_em = Some(0.05);
        let k_tracking = TextBlobKey::new("hello", &style, constraints, 1);
        assert_ne!(k0, k_tracking);
    }

    #[test]
    fn text_blob_key_includes_font_fallback_policy() {
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let base = TextStyle::default();
        let k0 = TextBlobKey::new("hello", &base, constraints, 1);
        let k1 = TextBlobKey::new("hello", &base, constraints, 2);
        assert_ne!(k0, k1);
    }

    #[test]
    fn ellipsis_overflow_truncates_single_line_layout() {
        let text = "This is a long line that should truncate";
        let constraints = TextConstraints {
            max_width: Some(Px(80.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            scale_factor: 1.0,
        };

        let mut shaper = crate::text_v2::parley_shaper::ParleyShaper::new();
        let base = TextStyle::default();
        let wrapped = crate::text_v2::wrapper::wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text, &base),
            constraints,
        );

        assert_eq!(wrapped.lines.len(), 1);
        assert!(wrapped.kept_end < text.len());
        assert!(wrapped.lines[0].width <= 80.0 + 0.5);
    }

    #[test]
    fn emoji_sequences_use_color_quads_when_color_font_is_available() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut text = super::TextSystem::new(&ctx.device);

        let fonts: Vec<Vec<u8>> = fret_fonts::bootstrap_fonts()
            .iter()
            .chain(fret_fonts::emoji_fonts().iter())
            .map(|b| b.to_vec())
            .collect();
        let added = text.add_fonts(fonts);
        assert!(added > 0, "expected bundled fonts to load");

        let style = TextStyle {
            font: fret_core::FontId::family("Noto Color Emoji"),
            size: Px(32.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let cases = [
            ("\u{1F600}", "single emoji"),
            ("\u{2708}\u{FE0F}", "vs16 emoji presentation"),
            ("1\u{FE0F}\u{20E3}", "keycap sequence"),
            ("\u{1F1FA}\u{1F1F8}", "flag sequence"),
            (
                "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
                "zwj family sequence",
            ),
            (
                "\u{1F3F3}\u{FE0F}\u{200D}\u{1F308}",
                "zwj rainbow flag sequence",
            ),
        ];

        for (text_str, label) in cases {
            let (blob_id, _metrics) = text.prepare(text_str, &style, constraints);
            let blob = text.blob(blob_id).expect("text blob");

            let mut color_glyphs: Vec<super::GlyphKey> = Vec::new();
            for g in blob.shape.glyphs.as_ref() {
                if matches!(g.kind(), super::GlyphQuadKind::Color) {
                    color_glyphs.push(g.key);
                }
            }

            assert!(
                !color_glyphs.is_empty(),
                "expected at least one color glyph quad for {label} when Noto Color Emoji is present"
            );

            let epoch = 1;
            for key in color_glyphs {
                text.ensure_glyph_in_atlas(key, epoch);
                assert!(
                    text.color_atlas.get(key, epoch).is_some(),
                    "expected color glyph to be present in color atlas after ensure ({label})"
                );
            }
        }
    }

    #[test]
    fn span_fingerprints_split_shaping_and_paint() {
        let constraints = TextConstraints {
            max_width: Some(Px(200.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };
        let base = TextStyle::default();
        let text = "hello";

        let mut spans_a = vec![TextSpan {
            len: text.len(),
            shaping: Default::default(),
            paint: Default::default(),
        }];
        spans_a[0].paint.fg = Some(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });
        let mut spans_b = spans_a.clone();
        spans_b[0].paint.fg = Some(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        });

        assert_eq!(
            spans_shaping_fingerprint(&spans_a),
            spans_shaping_fingerprint(&spans_b)
        );
        assert_ne!(
            spans_paint_fingerprint(&spans_a),
            spans_paint_fingerprint(&spans_b)
        );

        let rich_a = fret_core::AttributedText::new(
            Arc::<str>::from(text),
            Arc::<[TextSpan]>::from(spans_a),
        );
        let rich_b = fret_core::AttributedText::new(
            Arc::<str>::from(text),
            Arc::<[TextSpan]>::from(spans_b),
        );

        let k_a = TextBlobKey::new_attributed(&rich_a, &base, constraints, 7);
        let k_b = TextBlobKey::new_attributed(&rich_b, &base, constraints, 7);
        assert_ne!(k_a, k_b, "paint changes should affect blob cache keys");
        assert_eq!(
            TextShapeKey::from_blob_key(&k_a),
            TextShapeKey::from_blob_key(&k_b),
            "paint changes must not affect shape cache keys"
        );
    }
}
