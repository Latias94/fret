use cosmic_text::{
    Attrs, AttrsList, CacheKey, CacheKeyFlags, Family, FontSystem, Metrics, ShapeBuffer, ShapeLine,
    Shaping, SwashCache, Weight,
};
use cosmic_text::SwashContent;
use fret_core::{
    CaretAffinity, HitTestResult, Point, Rect, Size, TextBlobId, TextConstraints, TextMetrics,
    TextOverflow, TextStyle, TextWrap, geometry::Px,
};
use slotmap::SlotMap;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

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

fn font_stack_cache_key(locale: &str, db: &cosmic_text::fontdb::Database) -> u64 {
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

    hasher.finish()
}

#[derive(Debug, Clone)]
pub struct GlyphQuad {
    /// Logical-space rect relative to the text baseline origin.
    pub rect: [f32; 4],
    /// Normalized UV rect in the atlas: (u0, v0, u1, v1).
    pub uv: [f32; 4],
    pub kind: GlyphQuadKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphQuadKind {
    Mask,
    Color,
}

#[derive(Debug, Clone)]
pub struct TextBlob {
    pub glyphs: Vec<GlyphQuad>,
    pub metrics: TextMetrics,
    pub lines: Vec<TextLine>,
    pub caret_stops: Vec<(usize, Px)>,
    ref_count: u32,
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
    font: fret_core::FontId,
    font_stack_key: u64,
    size_bits: u32,
    weight: u16,
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
            font: style.font.clone(),
            font_stack_key,
            size_bits: style.size.0.to_bits(),
            weight: style.weight.0,
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
struct GlyphAtlasEntry {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
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

#[derive(Debug)]
struct GlyphAtlas {
    width: u32,
    height: u32,

    pen_x: u32,
    pen_y: u32,
    row_h: u32,

    glyphs: HashMap<CacheKey, GlyphAtlasEntry>,
    pending: Vec<PendingUpload>,
}

impl GlyphAtlas {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pen_x: 1,
            pen_y: 1,
            row_h: 0,
            glyphs: HashMap::new(),
            pending: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.pen_x = 1;
        self.pen_y = 1;
        self.row_h = 0;
        self.glyphs.clear();
        self.pending.clear();
    }

    fn allocate(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        let w = w.saturating_add(2);
        let h = h.saturating_add(2);

        if w > self.width || h > self.height {
            return None;
        }

        if self.pen_x.saturating_add(w) > self.width {
            self.pen_x = 1;
            self.pen_y = self.pen_y.saturating_add(self.row_h).saturating_add(1);
            self.row_h = 0;
        }

        if self.pen_y.saturating_add(h) > self.height {
            return None;
        }

        let x = self.pen_x;
        let y = self.pen_y;
        self.pen_x = self.pen_x.saturating_add(w).saturating_add(1);
        self.row_h = self.row_h.max(h);
        Some((x, y))
    }

    fn get_or_insert(
        &mut self,
        key: CacheKey,
        w: u32,
        h: u32,
        bytes_per_pixel: u32,
        data: Vec<u8>,
    ) -> Option<&GlyphAtlasEntry> {
        if self.glyphs.contains_key(&key) {
            return self.glyphs.get(&key);
        }

        let (x, y) = self.allocate(w, h)?;
        self.pending.push(PendingUpload {
            x,
            y,
            w,
            h,
            bytes_per_pixel,
            data,
        });
        self.glyphs.insert(key, GlyphAtlasEntry { x, y, w, h });
        self.glyphs.get(&key)
    }
}

fn subpixel_mask_to_alpha(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 4);
    for rgba in data.chunks_exact(4) {
        out.push(rgba[0].max(rgba[1]).max(rgba[2]));
    }
    out
}

pub struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
    scratch: ShapeBuffer,
    font_stack_key: u64,

    blobs: SlotMap<TextBlobId, TextBlob>,
    blob_cache: HashMap<TextBlobKey, TextBlobId>,
    blob_key_by_id: HashMap<TextBlobId, TextBlobKey>,

    mask_atlas: GlyphAtlas,
    color_atlas: GlyphAtlas,
    mask_atlas_texture: wgpu::Texture,
    color_atlas_texture: wgpu::Texture,
    atlas_bind_group_layout: wgpu::BindGroupLayout,
    mask_atlas_bind_group: wgpu::BindGroup,
    color_atlas_bind_group: wgpu::BindGroup,
}

fn family_for_font_id(font: &fret_core::FontId) -> Family<'_> {
    match font {
        fret_core::FontId::Ui => Family::SansSerif,
        fret_core::FontId::Serif => Family::Serif,
        fret_core::FontId::Monospace => Family::Monospace,
        fret_core::FontId::Family(name) => Family::Name(name.as_str()),
    }
}

/// Overrides for the default font family selection.
///
/// This configures the three generic families used by `TextStyle.font`
/// (`SansSerif`/`Serif`/`Monospace`). Full per-script fallback customization is tracked
/// separately (ADR 0029).
#[derive(Debug, Clone, Default)]
pub struct TextFontFamilyConfig {
    pub ui_sans: Vec<String>,
    pub ui_serif: Vec<String>,
    pub ui_mono: Vec<String>,
}

impl TextSystem {
    pub fn new(device: &wgpu::Device) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;

        let mask_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret glyph mask atlas"),
            size: wgpu::Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let color_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret glyph color atlas"),
            size: wgpu::Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mask_atlas_view =
            mask_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let color_atlas_view =
            color_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
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

        let mask_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret glyph mask atlas bind group"),
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&mask_atlas_view),
                },
            ],
        });

        let color_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret glyph color atlas bind group"),
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&color_atlas_view),
                },
            ],
        });

        let (locale, mut db) = FontSystem::new().into_locale_and_db();
        let installed = build_installed_family_set(&db);

        if let Some(sans) = first_installed_family(&installed, default_sans_candidates()) {
            db.set_sans_serif_family(sans);
        }
        if let Some(serif) = first_installed_family(&installed, default_serif_candidates()) {
            db.set_serif_family(serif);
        }
        if let Some(mono) = first_installed_family(&installed, default_monospace_candidates()) {
            db.set_monospace_family(mono);
        }

        let font_stack_key = font_stack_cache_key(&locale, &db);
        let font_system = FontSystem::new_with_locale_and_db_and_fallback(locale, db, FretFallback);

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            scratch: ShapeBuffer::default(),
            font_stack_key,

            blobs: SlotMap::with_key(),
            blob_cache: HashMap::new(),
            blob_key_by_id: HashMap::new(),

            mask_atlas: GlyphAtlas::new(atlas_width, atlas_height),
            color_atlas: GlyphAtlas::new(atlas_width, atlas_height),
            mask_atlas_texture,
            color_atlas_texture,
            atlas_bind_group_layout,
            mask_atlas_bind_group,
            color_atlas_bind_group,
        }
    }

    pub fn set_font_families(&mut self, config: &TextFontFamilyConfig) -> bool {
        let installed = build_installed_family_set(self.font_system.db());
        let old_key = self.font_stack_key;

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
            }
            if let Some(serif) = pick(&config.ui_serif, default_serif_candidates()) {
                db.set_serif_family(serif.as_ref());
            }
            if let Some(mono) = pick(&config.ui_mono, default_monospace_candidates()) {
                db.set_monospace_family(mono.as_ref());
            }
        }

        let new_key = font_stack_cache_key(self.font_system.locale(), self.font_system.db());
        if new_key == old_key {
            return false;
        }

        self.font_stack_key = new_key;
        self.blobs.clear();
        self.blob_cache.clear();
        self.blob_key_by_id.clear();
        self.mask_atlas.reset();
        self.color_atlas.reset();
        true
    }

    pub fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.atlas_bind_group_layout
    }

    pub fn mask_atlas_bind_group(&self) -> &wgpu::BindGroup {
        &self.mask_atlas_bind_group
    }

    pub fn color_atlas_bind_group(&self) -> &wgpu::BindGroup {
        &self.color_atlas_bind_group
    }

    pub fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        fn flush(pending: &mut Vec<PendingUpload>, texture: &wgpu::Texture, queue: &wgpu::Queue) {
            for upload in std::mem::take(pending) {
                if upload.w == 0 || upload.h == 0 {
                    continue;
                }

                let bytes_per_row = upload.w.saturating_mul(upload.bytes_per_pixel);
                if bytes_per_row == 0 {
                    continue;
                }

                let aligned_bytes_per_row =
                    bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
                let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);

                let expected_len = (bytes_per_row.saturating_mul(upload.h)) as usize;
                if upload.data.len() != expected_len {
                    continue;
                }

                let data = if aligned_bytes_per_row == bytes_per_row {
                    upload.data
                } else {
                    let mut padded = vec![0u8; (aligned_bytes_per_row * upload.h) as usize];
                    for row in 0..upload.h as usize {
                        let src0 = row * bytes_per_row as usize;
                        let src1 = src0 + bytes_per_row as usize;
                        let dst0 = row * aligned_bytes_per_row as usize;
                        let dst1 = dst0 + bytes_per_row as usize;
                        padded[dst0..dst1].copy_from_slice(&upload.data[src0..src1]);
                    }
                    padded
                };

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: upload.x,
                            y: upload.y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &data,
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

        if !self.mask_atlas.pending.is_empty() {
            flush(&mut self.mask_atlas.pending, &self.mask_atlas_texture, queue);
        }
        if !self.color_atlas.pending.is_empty() {
            flush(
                &mut self.color_atlas.pending,
                &self.color_atlas_texture,
                queue,
            );
        }
    }

    pub fn blob(&self, id: TextBlobId) -> Option<&TextBlob> {
        self.blobs.get(id)
    }

    pub fn prepare(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let key = TextBlobKey::new(text, style, constraints, self.font_stack_key);
        if let Some(id) = self.blob_cache.get(&key).copied() {
            if let Some(blob) = self.blobs.get_mut(id) {
                blob.ref_count = blob.ref_count.saturating_add(1);
                return (id, blob.metrics);
            }
            // Stale cache entry (shouldn't happen, but keep it robust).
            self.blob_cache.remove(&key);
            self.blob_key_by_id.remove(&id);
        }

        let scale = constraints.scale_factor.max(1.0);
        let font_size_px = (style.size.0 * scale).max(1.0);

        let mut attrs = Attrs::new().family(family_for_font_id(&style.font));
        attrs = attrs.weight(Weight(style.weight.0));
        if let Some(letter_spacing_em) = style.letter_spacing_em
            && letter_spacing_em != 0.0
            && letter_spacing_em.is_finite()
        {
            attrs = attrs.letter_spacing(letter_spacing_em);
        }
        if let Some(line_height) = style.line_height {
            let line_height_px = (line_height.0 * scale).max(font_size_px);
            if line_height_px.is_finite() {
                attrs = attrs.metrics(Metrics::new(font_size_px, line_height_px));
            }
        }
        let (layout, line_starts) = layout_text(
            &mut self.font_system,
            &mut self.scratch,
            text,
            &attrs,
            font_size_px,
            constraints,
            scale,
        );

        let metrics = layout.metrics;
        let first_ascent_px = metrics.baseline.0 * scale;

        let mut glyphs: Vec<GlyphQuad> = Vec::new();
        let mut lines: Vec<TextLine> = Vec::with_capacity(layout.lines.len().max(1));

        for (i, l) in layout.lines.iter().enumerate() {
            let base_offset = line_starts[i];

            let line_height_px = l
                .line_height_opt
                .unwrap_or_else(|| (l.max_ascent + l.max_descent).max(0.0))
                .max(0.0);

            let y_top_px = layout.line_tops_px[i];

            let local_start = layout.local_starts[i];
            let local_end = layout.local_ends[i];

            let mut boundaries_local: Vec<usize> =
                utf8_char_boundaries(&text[base_offset..layout.paragraph_ends[i]])
                    .into_iter()
                    .filter(|b| *b >= local_start && *b <= local_end)
                    .collect();
            boundaries_local.push(local_start);
            boundaries_local.push(local_end);
            boundaries_local.sort_unstable();
            boundaries_local.dedup();

            let caret_stops = build_line_caret_stops(
                base_offset,
                &boundaries_local,
                l.glyphs.as_slice(),
                local_start,
                local_end,
                l.w,
                scale,
            );

            lines.push(TextLine {
                start: base_offset + local_start,
                end: base_offset + local_end,
                width: Px(l.w / scale),
                y_top: Px(y_top_px / scale),
                height: Px(line_height_px / scale),
                caret_stops,
            });

            for g in &l.glyphs {
                if g.glyph_id == 0 {
                    continue;
                }

                let (cache_key, _, _) = CacheKey::new(
                    g.font_id,
                    g.glyph_id,
                    font_size_px,
                    (0.0, 0.0),
                    CacheKeyFlags::empty(),
                );

                let Some(image) = self
                    .swash_cache
                    .get_image(&mut self.font_system, cache_key)
                    .clone()
                else {
                    continue;
                };

                if image.placement.width == 0 || image.placement.height == 0 {
                    continue;
                }

                let (kind, bytes_per_pixel, data) = match image.content {
                    SwashContent::Mask => (GlyphQuadKind::Mask, 1, image.data),
                    SwashContent::Color => (GlyphQuadKind::Color, 4, image.data),
                    SwashContent::SubpixelMask => {
                        (GlyphQuadKind::Mask, 1, subpixel_mask_to_alpha(&image.data))
                    }
                };

                let (atlas_w, atlas_h, ex, ey, ew, eh) = match kind {
                    GlyphQuadKind::Mask => {
                        let (atlas_w, atlas_h) = (
                            self.mask_atlas.width as f32,
                            self.mask_atlas.height as f32,
                        );
                        let Some(e) = self.mask_atlas.get_or_insert(
                            cache_key,
                            image.placement.width,
                            image.placement.height,
                            bytes_per_pixel,
                            data,
                        ) else {
                            continue;
                        };
                        (atlas_w, atlas_h, e.x, e.y, e.w, e.h)
                    }
                    GlyphQuadKind::Color => {
                        let (atlas_w, atlas_h) = (
                            self.color_atlas.width as f32,
                            self.color_atlas.height as f32,
                        );
                        let Some(e) = self.color_atlas.get_or_insert(
                            cache_key,
                            image.placement.width,
                            image.placement.height,
                            bytes_per_pixel,
                            data,
                        ) else {
                            continue;
                        };
                        (atlas_w, atlas_h, e.x, e.y, e.w, e.h)
                    }
                };

                let line_baseline_px = y_top_px + l.max_ascent.max(0.0);
                let line_offset_px = line_baseline_px - first_ascent_px;
                let x0_px = g.x + image.placement.left as f32;
                let y0_px = (line_offset_px + g.y) - image.placement.top as f32;
                let w_px = image.placement.width as f32;
                let h_px = image.placement.height as f32;

                let u0 = ex as f32 / atlas_w;
                let v0 = ey as f32 / atlas_h;
                let u1 = (ex + ew) as f32 / atlas_w;
                let v1 = (ey + eh) as f32 / atlas_h;

                glyphs.push(GlyphQuad {
                    rect: [x0_px / scale, y0_px / scale, w_px / scale, h_px / scale],
                    uv: [u0, v0, u1, v1],
                    kind,
                });
            }
        }

        let caret_stops = lines
            .first()
            .map(|l| l.caret_stops.clone())
            .unwrap_or_else(|| vec![(0, Px(0.0))]);

        let id = self.blobs.insert(TextBlob {
            glyphs,
            metrics,
            lines,
            caret_stops,
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
        let scale = constraints.scale_factor.max(1.0);
        let font_size_px = (style.size.0 * scale).max(1.0);

        let mut attrs = Attrs::new().family(family_for_font_id(&style.font));
        attrs = attrs.weight(Weight(style.weight.0));
        if let Some(letter_spacing_em) = style.letter_spacing_em
            && letter_spacing_em != 0.0
            && letter_spacing_em.is_finite()
        {
            attrs = attrs.letter_spacing(letter_spacing_em);
        }
        if let Some(line_height) = style.line_height {
            let line_height_px = (line_height.0 * scale).max(font_size_px);
            if line_height_px.is_finite() {
                attrs = attrs.metrics(Metrics::new(font_size_px, line_height_px));
            }
        }
        layout_text(
            &mut self.font_system,
            &mut self.scratch,
            text,
            &attrs,
            font_size_px,
            constraints,
            scale,
        )
        .0
        .metrics
    }

    pub fn caret_x(&self, blob: TextBlobId, index: usize) -> Option<Px> {
        let blob_id = blob;
        let blob = self.blobs.get(blob_id)?;
        if blob.lines.len() > 1 {
            return Some(
                self.caret_rect(blob_id, index, CaretAffinity::Downstream)?
                    .origin
                    .x,
            );
        }
        let stops = blob.caret_stops.as_slice();
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
        if blob.lines.len() > 1 {
            return Some(self.hit_test_point(blob_id, Point::new(x, Px(0.0)))?.index);
        }
        let stops = blob.caret_stops.as_slice();
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
        Some(self.blobs.get(blob)?.caret_stops.as_slice())
    }

    pub fn caret_rect(
        &self,
        blob: TextBlobId,
        index: usize,
        affinity: CaretAffinity,
    ) -> Option<Rect> {
        let blob = self.blobs.get(blob)?;
        caret_rect_from_lines(&blob.lines, index, affinity)
    }

    pub fn hit_test_point(&self, blob: TextBlobId, point: Point) -> Option<HitTestResult> {
        let blob = self.blobs.get(blob)?;
        hit_test_point_from_lines(&blob.lines, point)
    }

    pub fn selection_rects(
        &self,
        blob: TextBlobId,
        range: (usize, usize),
        out: &mut Vec<Rect>,
    ) -> Option<()> {
        let blob = self.blobs.get(blob)?;
        selection_rects_from_lines(&blob.lines, range, out);
        Some(())
    }

    pub fn release(&mut self, blob: TextBlobId) {
        let should_remove = match self.blobs.get_mut(blob) {
            Some(b) => {
                if b.ref_count > 1 {
                    b.ref_count = b.ref_count.saturating_sub(1);
                    false
                } else {
                    true
                }
            }
            None => return,
        };

        if !should_remove {
            return;
        }

        if let Some(key) = self.blob_key_by_id.remove(&blob) {
            self.blob_cache.remove(&key);
        }
        let _ = self.blobs.remove(blob);
    }
}

#[derive(Debug, Clone)]
struct PreparedLayout {
    metrics: TextMetrics,
    lines: Vec<cosmic_text::LayoutLine>,
    line_tops_px: Vec<f32>,
    local_starts: Vec<usize>,
    local_ends: Vec<usize>,
    paragraph_ends: Vec<usize>,
}

fn layout_text(
    font_system: &mut FontSystem,
    scratch: &mut ShapeBuffer,
    text: &str,
    attrs: &Attrs,
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

    let mut push_slice = |base_offset: usize, slice: &str, paragraph_end: usize| {
        let mut attrs_list = AttrsList::new(attrs);
        attrs_list.add_span(0..slice.len(), attrs);

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
        );

        let mut ellipsis_local_end: Option<usize> = None;
        if want_ellipsis
            && layout_lines.len() == 1
            && let Some(max_w) = max_width_px
            && let Some(line) = layout_lines.get_mut(0)
            && line.w > max_w
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
                if g.glyph_id == 0 {
                    continue;
                }
                let right = (g.x + g.w).max(0.0);
                if right <= available_w {
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
            let height_px = ll
                .line_height_opt
                .unwrap_or(ascent_px + descent_px)
                .max(0.0);

            first_ascent_px.get_or_insert(ascent_px);
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
    use super::{TextBlobKey, layout_text, subpixel_mask_to_alpha};
    use cosmic_text::{Attrs, Family};
    use fret_core::{FontWeight, Px, TextConstraints, TextOverflow, TextStyle, TextWrap};

    #[test]
    fn subpixel_mask_to_alpha_uses_channel_max() {
        let data = vec![
            10u8, 3u8, 4u8, 0u8, //
            1u8, 200u8, 2u8, 0u8,
        ];
        assert_eq!(subpixel_mask_to_alpha(&data), vec![10u8, 200u8]);
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
    fn font_id_maps_to_cosmic_text_family() {
        assert_eq!(
            super::family_for_font_id(&fret_core::FontId::ui()),
            Family::SansSerif
        );
        assert_eq!(
            super::family_for_font_id(&fret_core::FontId::serif()),
            Family::Serif
        );
        assert_eq!(
            super::family_for_font_id(&fret_core::FontId::monospace()),
            Family::Monospace
        );
    }

    #[test]
    fn ellipsis_overflow_truncates_single_line_layout() {
        let mut font_system = cosmic_text::FontSystem::new();
        let mut scratch = cosmic_text::ShapeBuffer::default();

        let mut attrs = Attrs::new().family(Family::SansSerif);
        attrs = attrs.weight(cosmic_text::Weight(FontWeight::NORMAL.0));

        let text = "This is a long line that should truncate";
        let constraints = TextConstraints {
            max_width: Some(Px(80.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            scale_factor: 1.0,
        };

        let (layout, _) = layout_text(
            &mut font_system,
            &mut scratch,
            text,
            &attrs,
            13.0,
            constraints,
            1.0,
        );

        assert_eq!(layout.lines.len(), 1);
        assert!(layout.local_ends[0] < text.len());
        assert!(layout.lines[0].w <= 80.0 + 0.01);
    }
}
