use super::{GlyphQuadKind, TextSystem};
use fret_core::RendererGlyphAtlasPerfSnapshot;
use fret_core::scene::{Scene, SceneOp};
use fret_render_text::font_instance_key::FontFaceKey;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct GlyphKey {
    pub(super) font: FontFaceKey,
    pub(super) glyph_id: u32,
    pub(super) size_bits: u32,
    pub(super) x_bin: u8,
    pub(super) y_bin: u8,
    pub(super) kind: GlyphQuadKind,
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

pub(super) const TEXT_ATLAS_MAX_PAGES: usize = 2;

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
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
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
            allocator: etagere::BucketedAtlasAllocator::new(etagere::Size::new(
                width as i32,
                height as i32,
            )),
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
        let capacity_px = u64::from(self.width)
            .saturating_mul(u64::from(self.height))
            .saturating_mul(u64::from(pages));
        RendererGlyphAtlasPerfSnapshot {
            width: self.width,
            height: self.height,
            pages,
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

    pub(super) fn bind_group(&self, page: u16) -> &wgpu::BindGroup {
        assert!(
            !self.pages.is_empty(),
            "glyph atlas has no pages; bind_group() is only valid after at least one insert"
        );
        let idx = (page as usize).min(self.pages.len().saturating_sub(1));
        &self.pages[idx].bind_group
    }

    pub(super) fn revision(&self) -> u64 {
        self.revision
    }

    pub(super) fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(super) fn entry(&self, key: GlyphKey) -> Option<GlyphAtlasEntry> {
        self.glyphs.get(&key).copied()
    }

    pub(super) fn find_key_for_bounds(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<GlyphKey> {
        self.glyphs.iter().find_map(|(key, entry)| {
            (entry.page == page && entry.x == x && entry.y == y && entry.w == w && entry.h == h)
                .then_some(*key)
        })
    }

    #[cfg(test)]
    pub(super) fn pending_upload_bytes_for_entry(&self, entry: GlyphAtlasEntry) -> Option<Vec<u8>> {
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
        let Some(hit) = self.glyphs.get_mut(&key) else {
            self.perf_frame.misses = self.perf_frame.misses.saturating_add(1);
            return None;
        };
        self.perf_frame.hits = self.perf_frame.hits.saturating_add(1);
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

        let pad = self.padding_px;
        let w_pad = victim_entry.w.saturating_add(pad.saturating_mul(2));
        let h_pad = victim_entry.h.saturating_add(pad.saturating_mul(2));
        self.used_px = self
            .used_px
            .saturating_sub(u64::from(w_pad).saturating_mul(u64::from(h_pad)));

        let page_idx = (victim_entry.page as usize).min(self.pages.len().saturating_sub(1));
        self.pages[page_idx]
            .allocator
            .deallocate(victim_entry.alloc_id);
        self.glyphs.remove(&victim_key);
        self.perf.evicted_glyphs = self.perf.evicted_glyphs.saturating_add(1);
        self.revision = self.revision.saturating_add(1);
        self.perf_frame.evict_glyphs = self.perf_frame.evict_glyphs.saturating_add(1);
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
        let pad = self.padding_px;
        self.perf.evicted_pages = self.perf.evicted_pages.saturating_add(1);
        self.perf.evicted_page_glyphs = self
            .perf
            .evicted_page_glyphs
            .saturating_add(keys_to_remove.len() as u64);
        for k in keys_to_remove {
            if let Some(entry) = self.glyphs.remove(&k) {
                let w_pad = entry.w.saturating_add(pad.saturating_mul(2));
                let h_pad = entry.h.saturating_add(pad.saturating_mul(2));
                self.used_px = self
                    .used_px
                    .saturating_sub(u64::from(w_pad).saturating_mul(u64::from(h_pad)));
            }
        }

        self.revision = self.revision.saturating_add(1);
        self.perf_frame.evict_pages = self.perf_frame.evict_pages.saturating_add(1);
        true
    }

    pub(super) fn flush_uploads(&mut self, queue: &wgpu::Queue) {
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

                self.perf.uploads = self.perf.uploads.saturating_add(1);
                self.perf.upload_bytes = self.perf.upload_bytes.saturating_add(bytes.len() as u64);
                self.perf_frame.upload_bytes = self
                    .perf_frame
                    .upload_bytes
                    .saturating_add(bytes.len() as u64);

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &page.texture,
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
        if let Some(hit) = self.glyphs.get_mut(&key) {
            self.perf_frame.hits = self.perf_frame.hits.saturating_add(1);
            hit.last_used_epoch = epoch;
            let idx = (hit.page as usize).min(self.pages.len().saturating_sub(1));
            self.pages[idx].last_used_epoch = epoch;
            return Ok(*hit);
        }
        self.perf_frame.misses = self.perf_frame.misses.saturating_add(1);

        let pad = self.padding_px;
        let w_pad = w.saturating_add(pad.saturating_mul(2));
        let h_pad = h.saturating_add(pad.saturating_mul(2));
        if w == 0 || h == 0 || w_pad == 0 || h_pad == 0 || w_pad > self.width || h_pad > self.height
        {
            self.perf_frame.too_large = self.perf_frame.too_large.saturating_add(1);
            return Err(GlyphAtlasInsertError::TooLarge);
        }

        let size = etagere::Size::new(w_pad as i32, h_pad as i32);

        if self.pages.is_empty() && !self.try_grow_pages() {
            self.perf_frame.out_of_space = self.perf_frame.out_of_space.saturating_add(1);
            return Err(GlyphAtlasInsertError::OutOfSpace);
        }

        let mut guard = 0_u32;
        loop {
            guard = guard.saturating_add(1);
            if guard >= 128 {
                self.perf_frame.out_of_space = self.perf_frame.out_of_space.saturating_add(1);
                return Err(GlyphAtlasInsertError::OutOfSpace);
            }

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

                self.perf_frame.inserts = self.perf_frame.inserts.saturating_add(1);
                self.perf_frame.pending_uploads = self.perf_frame.pending_uploads.saturating_add(1);
                self.perf_frame.pending_upload_bytes =
                    self.perf_frame.pending_upload_bytes.saturating_add(
                        u64::from(w)
                            .saturating_mul(u64::from(h))
                            .saturating_mul(u64::from(bytes_per_pixel)),
                    );
                self.used_px = self
                    .used_px
                    .saturating_add(u64::from(w_pad).saturating_mul(u64::from(h_pad)));

                let entry = GlyphAtlasEntry {
                    page: page_index as u16,
                    alloc_id: allocation.id,
                    x,
                    y,
                    w,
                    h,
                    placement_left,
                    placement_top,
                    live_refs: 0,
                    last_used_epoch: epoch,
                };
                self.glyphs.insert(key, entry);
                self.revision = self.revision.saturating_add(1);
                return Ok(entry);
            }

            if self.try_grow_pages() {
                continue;
            }
            if self.evict_lru_unreferenced_glyph() {
                continue;
            }
            if self.evict_lru_unreferenced_page() {
                continue;
            }
            self.perf_frame.out_of_space = self.perf_frame.out_of_space.saturating_add(1);
            return Err(GlyphAtlasInsertError::OutOfSpace);
        }
    }
}

impl TextSystem {
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

    pub(super) fn ensure_glyph_in_atlas(&mut self, key: GlyphKey, epoch: u64) {
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
        let Some(font_data) = self
            .font_data_by_face
            .get(&(key.font.font_data_id, key.font.face_index))
        else {
            return;
        };

        let Some(font_ref) =
            parley::swash::FontRef::from_index(font_data.data.data(), key.font.face_index as usize)
        else {
            return;
        };
        let Ok(glyph_id) = u16::try_from(key.glyph_id) else {
            return;
        };

        let font_size = f32::from_bits(key.size_bits).max(1.0);
        let mut scaler_builder = self
            .parley_scale
            .builder(font_ref)
            .size(font_size)
            .hint(false);
        if let Some(coords) = self.font_instance_coords_by_face.get(&key.font) {
            scaler_builder = scaler_builder.normalized_coords(coords.iter());
        }
        let mut scaler = scaler_builder.build();

        let offset_px = parley::swash::zeno::Vector::new(
            subpixel_bin_as_float(key.x_bin),
            subpixel_bin_as_float(key.y_bin),
        );
        let mut render = parley::swash::scale::Render::new(&[
            parley::swash::scale::Source::ColorOutline(0),
            parley::swash::scale::Source::ColorBitmap(parley::swash::scale::StrikeWith::BestFit),
            parley::swash::scale::Source::Outline,
        ]);
        render.offset(offset_px);

        if key.font.synthesis_embolden {
            let strength = (font_size / 48.0).clamp(0.25, 1.0);
            render.embolden(strength);
        }

        if key.font.synthesis_skew_degrees != 0 {
            let angle =
                parley::swash::zeno::Angle::from_degrees(key.font.synthesis_skew_degrees as f32);
            let t = parley::swash::zeno::Transform::skew(angle, parley::swash::zeno::Angle::ZERO);
            render.transform(Some(t));
        }

        let Some(image) = render.render(&mut scaler, glyph_id) else {
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
                    image.placement.left,
                    image.placement.top,
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
                    image.placement.left,
                    image.placement.top,
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
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
        }
    }
}
