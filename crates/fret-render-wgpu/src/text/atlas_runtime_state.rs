use super::atlas::{GlyphAtlas, GlyphKey, TEXT_ATLAS_MAX_PAGES};
use super::{DebugGlyphAtlasLookup, TextAtlasPerfSnapshot};

const TEXT_ATLAS_WIDTH: u32 = 2048;
const TEXT_ATLAS_HEIGHT: u32 = 2048;

pub(crate) struct TextAtlasRuntimeState {
    mask_atlas: GlyphAtlas,
    color_atlas: GlyphAtlas,
    subpixel_atlas: GlyphAtlas,
    atlas_bind_group_layout: wgpu::BindGroupLayout,
}

impl TextAtlasRuntimeState {
    pub(super) fn bootstrap(device: &wgpu::Device) -> Self {
        let atlas_sampler = create_atlas_sampler(device);
        let atlas_bind_group_layout = create_atlas_bind_group_layout(device);

        let mask_atlas = create_text_atlas(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph mask atlas",
            wgpu::TextureFormat::R8Unorm,
        );
        let color_atlas = create_text_atlas(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph color atlas",
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        let subpixel_atlas = create_text_atlas(
            device,
            &atlas_bind_group_layout,
            &atlas_sampler,
            "fret glyph subpixel atlas",
            wgpu::TextureFormat::Rgba8Unorm,
        );

        Self::new(
            mask_atlas,
            color_atlas,
            subpixel_atlas,
            atlas_bind_group_layout,
        )
    }

    fn new(
        mask_atlas: GlyphAtlas,
        color_atlas: GlyphAtlas,
        subpixel_atlas: GlyphAtlas,
        atlas_bind_group_layout: wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            mask_atlas,
            color_atlas,
            subpixel_atlas,
            atlas_bind_group_layout,
        }
    }

    pub(super) fn reset(&mut self) {
        self.mask_atlas.reset();
        self.color_atlas.reset();
        self.subpixel_atlas.reset();
    }

    pub(super) fn begin_frame_diagnostics(&mut self) {
        self.mask_atlas.begin_frame_diagnostics();
        self.color_atlas.begin_frame_diagnostics();
        self.subpixel_atlas.begin_frame_diagnostics();
    }

    pub(super) fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.atlas_bind_group_layout
    }

    pub(super) fn mask_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.mask_atlas.bind_group(page)
    }

    pub(super) fn color_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.color_atlas.bind_group(page)
    }

    pub(super) fn subpixel_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.subpixel_atlas.bind_group(page)
    }

    pub(super) fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        self.mask_atlas.flush_uploads(queue);
        self.color_atlas.flush_uploads(queue);
        self.subpixel_atlas.flush_uploads(queue);
    }

    pub(super) fn diagnostics_snapshots(
        &self,
    ) -> (
        fret_core::RendererGlyphAtlasPerfSnapshot,
        fret_core::RendererGlyphAtlasPerfSnapshot,
        fret_core::RendererGlyphAtlasPerfSnapshot,
    ) {
        (
            self.mask_atlas.diagnostics_snapshot(),
            self.color_atlas.diagnostics_snapshot(),
            self.subpixel_atlas.diagnostics_snapshot(),
        )
    }

    pub(super) fn take_perf_snapshot(&mut self) -> TextAtlasPerfSnapshot {
        let mask = self.mask_atlas.take_perf_snapshot();
        let color = self.color_atlas.take_perf_snapshot();
        let subpixel = self.subpixel_atlas.take_perf_snapshot();

        TextAtlasPerfSnapshot {
            uploads: mask.uploads + color.uploads + subpixel.uploads,
            upload_bytes: mask.upload_bytes + color.upload_bytes + subpixel.upload_bytes,
            evicted_glyphs: mask.evicted_glyphs + color.evicted_glyphs + subpixel.evicted_glyphs,
            evicted_pages: mask.evicted_pages + color.evicted_pages + subpixel.evicted_pages,
            evicted_page_glyphs: mask.evicted_page_glyphs
                + color.evicted_page_glyphs
                + subpixel.evicted_page_glyphs,
            resets: mask.resets + color.resets + subpixel.resets,
        }
    }

    pub(super) fn combined_revision(&self) -> u64 {
        self.mask_atlas
            .revision()
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ self.color_atlas.revision().rotate_left(1)
            ^ self.subpixel_atlas.revision().rotate_left(2)
    }

    pub(super) fn mask_dimensions(&self) -> (u32, u32) {
        self.mask_atlas.dimensions()
    }

    pub(super) fn color_dimensions(&self) -> (u32, u32) {
        self.color_atlas.dimensions()
    }

    pub(super) fn subpixel_dimensions(&self) -> (u32, u32) {
        self.subpixel_atlas.dimensions()
    }

    fn atlas_for_key(&self, key: GlyphKey) -> &GlyphAtlas {
        if key.is_color() {
            &self.color_atlas
        } else if key.is_subpixel() {
            &self.subpixel_atlas
        } else {
            &self.mask_atlas
        }
    }

    fn atlas_mut_for_key(&mut self, key: GlyphKey) -> &mut GlyphAtlas {
        if key.is_color() {
            &mut self.color_atlas
        } else if key.is_subpixel() {
            &mut self.subpixel_atlas
        } else {
            &mut self.mask_atlas
        }
    }

    pub(super) fn prepared_bounds_for_key(
        &mut self,
        key: GlyphKey,
        x: i32,
        y: i32,
        epoch: u64,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        let (x0, y0, w, h) = self
            .atlas_mut_for_key(key)
            .touch_bounds_for_key(key, x, y, epoch)?;
        Some((key, x0, y0, w, h))
    }

    pub(super) fn touch_if_present(&mut self, key: GlyphKey, epoch: u64) -> bool {
        self.atlas_mut_for_key(key).touch_if_present(key, epoch)
    }

    #[cfg(test)]
    pub(super) fn contains_key(&self, key: GlyphKey) -> bool {
        self.atlas_for_key(key).contains_key(key)
    }

    pub(super) fn uv_for_key(&self, key: GlyphKey) -> Option<(u16, [f32; 4])> {
        self.atlas_for_key(key).uv_for_key(key)
    }

    pub(super) fn dec_pin_bucket(
        &mut self,
        mask: &[GlyphKey],
        color: &[GlyphKey],
        subpixel: &[GlyphKey],
    ) {
        self.mask_atlas.dec_live_refs(mask);
        self.color_atlas.dec_live_refs(color);
        self.subpixel_atlas.dec_live_refs(subpixel);
    }

    pub(super) fn inc_pin_bucket(
        &mut self,
        mask: &[GlyphKey],
        color: &[GlyphKey],
        subpixel: &[GlyphKey],
    ) {
        self.mask_atlas.inc_live_refs(mask);
        self.color_atlas.inc_live_refs(color);
        self.subpixel_atlas.inc_live_refs(subpixel);
    }

    pub(super) fn cache_glyph(
        &mut self,
        key: GlyphKey,
        w: u32,
        h: u32,
        placement_left: i32,
        placement_top: i32,
        bytes_per_pixel: u32,
        data: Vec<u8>,
        epoch: u64,
    ) {
        let _ = self.atlas_mut_for_key(key).get_or_insert(
            key,
            w,
            h,
            placement_left,
            placement_top,
            bytes_per_pixel,
            data,
            epoch,
        );
    }

    #[cfg(test)]
    pub(super) fn pending_upload_bytes_for_key(&self, key: GlyphKey) -> Option<Vec<u8>> {
        self.atlas_for_key(key).pending_upload_bytes_for_key(key)
    }

    pub(super) fn debug_lookup_mask_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry_in_atlas(&self.mask_atlas, page, x, y, w, h)
    }

    pub(super) fn debug_lookup_color_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry_in_atlas(&self.color_atlas, page, x, y, w, h)
    }

    pub(super) fn debug_lookup_subpixel_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry_in_atlas(&self.subpixel_atlas, page, x, y, w, h)
    }

    fn debug_lookup_entry_in_atlas(
        &self,
        atlas: &GlyphAtlas,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        let k = atlas.find_key_for_bounds(page, x, y, w, h)?;

        Some(DebugGlyphAtlasLookup::new(
            k.font.font_data_id(),
            k.font.face_index(),
            k.font.variation_key(),
            k.font.synthesis_embolden(),
            k.font.synthesis_skew_degrees(),
            k.glyph_id,
            k.size_bits,
            k.x_bin,
            k.y_bin,
            k.kind_label(),
        ))
    }
}

fn create_atlas_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fret glyph atlas sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

fn create_atlas_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
    })
}

fn create_text_atlas(
    device: &wgpu::Device,
    atlas_bind_group_layout: &wgpu::BindGroupLayout,
    atlas_sampler: &wgpu::Sampler,
    label_prefix: &str,
    format: wgpu::TextureFormat,
) -> GlyphAtlas {
    GlyphAtlas::new(
        device,
        atlas_bind_group_layout,
        atlas_sampler,
        label_prefix,
        TEXT_ATLAS_WIDTH,
        TEXT_ATLAS_HEIGHT,
        format,
        0,
        TEXT_ATLAS_MAX_PAGES,
    )
}
