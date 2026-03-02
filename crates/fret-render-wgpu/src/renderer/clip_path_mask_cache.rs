use super::intermediate_pool::{IntermediatePool, PooledTexture, estimate_clip_mask_bytes};
use std::collections::HashMap;

pub(super) struct ClipPathMaskCache {
    entries: HashMap<u64, ClipPathMaskCacheEntry>,
    bytes_live: u64,
    budget_bytes: u64,
}

struct ClipPathMaskCacheEntry {
    size: (u32, u32),
    texture: PooledTexture,
    last_used_frame: u64,
}

impl ClipPathMaskCache {
    pub(super) fn new(budget_bytes: u64) -> Self {
        Self {
            entries: HashMap::new(),
            bytes_live: 0,
            budget_bytes,
        }
    }

    pub(super) fn bytes_live(&self) -> u64 {
        self.bytes_live
    }

    pub(super) fn entries_live(&self) -> u64 {
        self.entries.len() as u64
    }

    pub(super) fn enforce_budget(&mut self, pool: &mut IntermediatePool, budget_bytes: u64) {
        self.budget_bytes = budget_bytes;
        self.evict_until_within_budget(pool);
    }

    pub(super) fn try_copy_into(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        cache_key: u64,
        size: (u32, u32),
        dst: &wgpu::Texture,
        frame_index: u64,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(&cache_key) else {
            return false;
        };
        if entry.size != size {
            return false;
        }
        entry.last_used_frame = frame_index;

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: dst,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
        );
        true
    }

    pub(super) fn store_from(
        &mut self,
        pool: &mut IntermediatePool,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        cache_key: u64,
        size: (u32, u32),
        src: &wgpu::Texture,
        frame_index: u64,
    ) {
        let size = (size.0.max(1), size.1.max(1));
        let bytes = estimate_clip_mask_bytes(size);

        let usage = wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST;

        let entry = self.entries.entry(cache_key).or_insert_with(|| {
            let texture = pool.acquire_texture(
                device,
                "fret clip-path mask cache",
                size,
                wgpu::TextureFormat::R8Unorm,
                usage,
                1,
            );
            self.bytes_live = self.bytes_live.saturating_add(texture.bytes);
            ClipPathMaskCacheEntry {
                size,
                texture,
                last_used_frame: frame_index,
            }
        });

        if entry.size != size {
            let new_texture = pool.acquire_texture(
                device,
                "fret clip-path mask cache",
                size,
                wgpu::TextureFormat::R8Unorm,
                usage,
                1,
            );
            let old_texture = std::mem::replace(&mut entry.texture, new_texture);
            self.bytes_live = self.bytes_live.saturating_sub(old_texture.bytes);
            pool.release(old_texture);
            self.bytes_live = self.bytes_live.saturating_add(entry.texture.bytes);
            entry.size = size;
        }

        entry.last_used_frame = frame_index;

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );

        if bytes > self.budget_bytes {
            return;
        }
        self.evict_until_within_budget(pool);
    }

    fn evict_until_within_budget(&mut self, pool: &mut IntermediatePool) {
        if self.budget_bytes == 0 {
            self.clear(pool);
            return;
        }

        while self.bytes_live > self.budget_bytes {
            let mut victim_key: Option<u64> = None;
            let mut victim_frame: u64 = u64::MAX;
            for (&k, v) in &self.entries {
                if v.last_used_frame < victim_frame {
                    victim_frame = v.last_used_frame;
                    victim_key = Some(k);
                }
            }
            let Some(victim_key) = victim_key else {
                break;
            };
            if let Some(entry) = self.entries.remove(&victim_key) {
                self.bytes_live = self.bytes_live.saturating_sub(entry.texture.bytes);
                pool.release(entry.texture);
            } else {
                break;
            }
        }
    }

    fn clear(&mut self, pool: &mut IntermediatePool) {
        for (_, entry) in std::mem::take(&mut self.entries) {
            pool.release(entry.texture);
        }
        self.bytes_live = 0;
    }
}
