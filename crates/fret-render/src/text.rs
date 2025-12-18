use cosmic_text::{
    Attrs, AttrsList, CacheKey, CacheKeyFlags, Family, FontSystem, ShapeBuffer, ShapeLine, Shaping,
    SwashCache,
};
use fret_core::{
    Size, TextBlobId, TextConstraints, TextMetrics, TextStyle, TextWrap, geometry::Px,
};
use slotmap::SlotMap;
use std::{collections::HashMap, hash::Hash, sync::Arc};

#[derive(Debug, Clone)]
pub struct GlyphQuad {
    /// Logical-space rect relative to the text baseline origin.
    pub rect: [f32; 4],
    /// Normalized UV rect in the atlas: (u0, v0, u1, v1).
    pub uv: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct TextBlob {
    pub glyphs: Vec<GlyphQuad>,
    pub metrics: TextMetrics,
    pub caret_stops: Vec<(usize, Px)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextBlobKey {
    text: Arc<str>,
    font: fret_core::FontId,
    size_bits: u32,
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    scale_bits: u32,
}

impl TextBlobKey {
    fn new(text: &str, style: TextStyle, constraints: TextConstraints) -> Self {
        let max_width_bits = constraints.max_width.map(|w| w.0.to_bits());
        Self {
            text: Arc::<str>::from(text),
            font: style.font,
            size_bits: style.size.0.to_bits(),
            max_width_bits,
            wrap: constraints.wrap,
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
        data: Vec<u8>,
    ) -> Option<&GlyphAtlasEntry> {
        if self.glyphs.contains_key(&key) {
            return self.glyphs.get(&key);
        }

        let (x, y) = self.allocate(w, h)?;
        self.pending.push(PendingUpload { x, y, w, h, data });
        self.glyphs.insert(key, GlyphAtlasEntry { x, y, w, h });
        self.glyphs.get(&key)
    }
}

pub struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
    scratch: ShapeBuffer,

    blobs: SlotMap<TextBlobId, TextBlob>,
    blob_cache: HashMap<TextBlobKey, TextBlobId>,
    blob_key_by_id: HashMap<TextBlobId, TextBlobKey>,

    atlas: GlyphAtlas,
    atlas_texture: wgpu::Texture,
    atlas_bind_group_layout: wgpu::BindGroupLayout,
    atlas_bind_group: wgpu::BindGroup,
}

impl TextSystem {
    pub fn new(device: &wgpu::Device) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret glyph atlas"),
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

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
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

        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret glyph atlas bind group"),
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
            ],
        });

        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            scratch: ShapeBuffer::default(),

            blobs: SlotMap::with_key(),
            blob_cache: HashMap::new(),
            blob_key_by_id: HashMap::new(),

            atlas: GlyphAtlas::new(atlas_width, atlas_height),
            atlas_texture,
            atlas_bind_group_layout,
            atlas_bind_group,
        }
    }

    pub fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.atlas_bind_group_layout
    }

    pub fn atlas_bind_group(&self) -> &wgpu::BindGroup {
        &self.atlas_bind_group
    }

    pub fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        if self.atlas.pending.is_empty() {
            return;
        }

        for upload in std::mem::take(&mut self.atlas.pending) {
            if upload.w == 0 || upload.h == 0 {
                continue;
            }

            // WebGPU requires `bytes_per_row` to be aligned; pad each row as needed.
            let bytes_per_row = upload.w;
            let aligned_bytes_per_row = ((bytes_per_row + wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1)
                / wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);

            let data = if aligned_bytes_per_row == bytes_per_row {
                upload.data
            } else {
                let mut padded = vec![0u8; (aligned_bytes_per_row * upload.h) as usize];
                for row in 0..upload.h as usize {
                    let src0 = row * upload.w as usize;
                    let src1 = src0 + upload.w as usize;
                    let dst0 = row * aligned_bytes_per_row as usize;
                    let dst1 = dst0 + upload.w as usize;
                    padded[dst0..dst1].copy_from_slice(&upload.data[src0..src1]);
                }
                padded
            };

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
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

    pub fn blob(&self, id: TextBlobId) -> Option<&TextBlob> {
        self.blobs.get(id)
    }

    pub fn prepare(
        &mut self,
        text: &str,
        style: TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let key = TextBlobKey::new(text, style, constraints);
        if let Some(id) = self.blob_cache.get(&key).copied() {
            if let Some(blob) = self.blobs.get(id) {
                return (id, blob.metrics);
            }
        }

        let scale = constraints.scale_factor.max(1.0);
        let font_size_px = (style.size.0 * scale).max(1.0);

        let attrs = Attrs::new().family(Family::SansSerif);
        let mut attrs_list = AttrsList::new(&attrs);
        attrs_list.add_span(0..text.len(), &attrs);

        let line = ShapeLine::new(
            &mut self.font_system,
            text,
            &attrs_list,
            Shaping::Advanced,
            4,
        );

        let mut layout_lines = Vec::with_capacity(1);
        let max_width = constraints.max_width.map(|w| w.0 * scale);
        let wrap = match constraints.wrap {
            TextWrap::None => cosmic_text::Wrap::None,
            TextWrap::Word => cosmic_text::Wrap::Word,
        };

        line.layout_to_buffer(
            &mut self.scratch,
            font_size_px,
            max_width,
            wrap,
            None,
            &mut layout_lines,
            None,
        );

        let layout = layout_lines.first();
        let (w_px, ascent_px, descent_px, glyphs_src) = match layout {
            Some(layout) => (
                layout.w,
                layout.max_ascent,
                layout.max_descent,
                layout.glyphs.as_slice(),
            ),
            None => (0.0, 0.0, 0.0, &[][..]),
        };

        let metrics = TextMetrics {
            size: Size::new(Px(w_px / scale), Px((ascent_px + descent_px) / scale)),
            baseline: Px(ascent_px / scale),
        };

        let caret_stops = build_single_line_caret_stops(text, glyphs_src, w_px, scale);

        let mut glyphs: Vec<GlyphQuad> = Vec::new();

        for g in glyphs_src {
            if g.glyph_id == 0 {
                continue;
            }

            let (cache_key, _, _) = CacheKey::new(
                g.font_id,
                g.glyph_id as u16,
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

            let (atlas_w, atlas_h) = (self.atlas.width as f32, self.atlas.height as f32);
            let (ex, ey, ew, eh) = match self.atlas.get_or_insert(
                cache_key,
                image.placement.width,
                image.placement.height,
                image.data,
            ) {
                Some(e) => (e.x, e.y, e.w, e.h),
                None => continue,
            };

            let x0_px = g.x as f32 + image.placement.left as f32;
            let y0_px = g.y as f32 - image.placement.top as f32;
            let w_px = image.placement.width as f32;
            let h_px = image.placement.height as f32;

            let u0 = ex as f32 / atlas_w;
            let v0 = ey as f32 / atlas_h;
            let u1 = (ex + ew) as f32 / atlas_w;
            let v1 = (ey + eh) as f32 / atlas_h;

            glyphs.push(GlyphQuad {
                rect: [x0_px / scale, y0_px / scale, w_px / scale, h_px / scale],
                uv: [u0, v0, u1, v1],
            });
        }

        let id = self.blobs.insert(TextBlob {
            glyphs,
            metrics,
            caret_stops,
        });
        self.blob_cache.insert(key.clone(), id);
        self.blob_key_by_id.insert(id, key);
        (id, metrics)
    }

    pub fn measure(
        &mut self,
        text: &str,
        style: TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        let scale = constraints.scale_factor.max(1.0);
        let font_size_px = (style.size.0 * scale).max(1.0);

        let attrs = Attrs::new().family(Family::SansSerif);
        let mut attrs_list = AttrsList::new(&attrs);
        attrs_list.add_span(0..text.len(), &attrs);

        let line = ShapeLine::new(
            &mut self.font_system,
            text,
            &attrs_list,
            Shaping::Advanced,
            4,
        );

        let mut layout_lines = Vec::with_capacity(1);
        let max_width = constraints.max_width.map(|w| w.0 * scale);
        let wrap = match constraints.wrap {
            TextWrap::None => cosmic_text::Wrap::None,
            TextWrap::Word => cosmic_text::Wrap::Word,
        };

        line.layout_to_buffer(
            &mut self.scratch,
            font_size_px,
            max_width,
            wrap,
            None,
            &mut layout_lines,
            None,
        );

        let layout = layout_lines.first();
        let (w_px, ascent_px, descent_px) = match layout {
            Some(layout) => (layout.w, layout.max_ascent, layout.max_descent),
            None => (0.0, 0.0, 0.0),
        };

        TextMetrics {
            size: Size::new(Px(w_px / scale), Px((ascent_px + descent_px) / scale)),
            baseline: Px(ascent_px / scale),
        }
    }

    pub fn caret_x(&self, blob: TextBlobId, index: usize) -> Option<Px> {
        let blob = self.blobs.get(blob)?;
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
        let blob = self.blobs.get(blob)?;
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

    pub fn release(&mut self, blob: TextBlobId) {
        let Some(key) = self.blob_key_by_id.remove(&blob) else {
            return;
        };
        self.blob_cache.remove(&key);
        self.blobs.remove(blob);
    }
}

fn build_single_line_caret_stops(
    text: &str,
    glyphs: &[cosmic_text::LayoutGlyph],
    line_w_px: f32,
    scale: f32,
) -> Vec<(usize, Px)> {
    let mut boundaries: Vec<usize> = Vec::with_capacity(text.chars().count().saturating_add(2));
    boundaries.push(0);
    for (i, _) in text.char_indices() {
        if i != 0 {
            boundaries.push(i);
        }
    }
    boundaries.push(text.len());
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut out: Vec<(usize, Px)> = Vec::with_capacity(boundaries.len());
    for idx in boundaries {
        if idx == 0 {
            out.push((0, Px(0.0)));
            continue;
        }
        if idx >= text.len() {
            out.push((text.len(), Px(line_w_px / scale)));
            continue;
        }

        let mut x_end = 0.0_f32;
        for g in glyphs {
            if g.end <= idx {
                x_end = x_end.max(g.x + g.w);
            }
        }
        out.push((idx, Px(x_end / scale)));
    }
    out
}
