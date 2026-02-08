use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PoolKey {
    size: (u32, u32),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    sample_count: u32,
}

#[derive(Default)]
pub(super) struct IntermediatePool {
    free: HashMap<PoolKey, Vec<wgpu::Texture>>,
    free_bytes: u64,
    perf: PoolPerfStats,
}

pub(super) struct PooledTexture {
    key: PoolKey,
    pub(super) bytes: u64,
    pub(super) texture: wgpu::Texture,
}

impl IntermediatePool {
    #[cfg(test)]
    pub(super) fn free_bytes(&self) -> u64 {
        self.free_bytes
    }

    pub(super) fn take_perf_snapshot(&mut self) -> PoolPerfSnapshot {
        let snap = PoolPerfSnapshot {
            allocations: self.perf.allocations,
            reuses: self.perf.reuses,
            releases: self.perf.releases,
            evictions: self.perf.evictions,
            free_bytes: self.free_bytes,
            free_textures: self.free.values().map(|v| v.len() as u64).sum(),
        };
        self.perf = PoolPerfStats::default();
        snap
    }

    pub(super) fn acquire_texture(
        &mut self,
        device: &wgpu::Device,
        label: &'static str,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        sample_count: u32,
    ) -> PooledTexture {
        let key = PoolKey {
            size,
            format,
            usage,
            sample_count,
        };
        let bytes = estimate_texture_bytes(size, format, sample_count);

        if let Some(textures) = self.free.get_mut(&key)
            && let Some(texture) = textures.pop()
        {
            self.free_bytes = self.free_bytes.saturating_sub(bytes);
            self.perf.reuses = self.perf.reuses.saturating_add(1);
            return PooledTexture {
                key,
                bytes,
                texture,
            };
        }

        self.perf.allocations = self.perf.allocations.saturating_add(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        PooledTexture {
            key,
            bytes,
            texture,
        }
    }

    pub(super) fn release(&mut self, texture: PooledTexture) {
        self.perf.releases = self.perf.releases.saturating_add(1);
        self.free_bytes = self.free_bytes.saturating_add(texture.bytes);
        self.free
            .entry(texture.key)
            .or_default()
            .push(texture.texture);
    }

    pub(super) fn enforce_budget(&mut self, budget_bytes: u64) {
        if budget_bytes == 0 {
            self.free.clear();
            self.free_bytes = 0;
            return;
        }

        while self.free_bytes > budget_bytes {
            let Some(victim_key) = self.pick_eviction_key() else {
                break;
            };
            let victim_bytes =
                estimate_texture_bytes(victim_key.size, victim_key.format, victim_key.sample_count);

            let remove_key = match self.free.get_mut(&victim_key) {
                Some(textures) => {
                    let _ = textures.pop();
                    textures.is_empty()
                }
                None => true,
            };
            if remove_key {
                self.free.remove(&victim_key);
            }

            self.free_bytes = self.free_bytes.saturating_sub(victim_bytes);
            self.perf.evictions = self.perf.evictions.saturating_add(1);
        }
    }

    fn pick_eviction_key(&self) -> Option<PoolKey> {
        let mut keys: Vec<PoolKey> = self.free.keys().copied().collect();
        keys.sort_by(|a, b| {
            let a_bytes = estimate_texture_bytes(a.size, a.format, a.sample_count);
            let b_bytes = estimate_texture_bytes(b.size, b.format, b.sample_count);
            b_bytes
                .cmp(&a_bytes)
                .then_with(|| b.size.0.cmp(&a.size.0))
                .then_with(|| b.size.1.cmp(&a.size.1))
                .then_with(|| (b.sample_count).cmp(&a.sample_count))
                .then_with(|| (b.usage.bits()).cmp(&a.usage.bits()))
                .then_with(|| format!("{:?}", b.format).cmp(&format!("{:?}", a.format)))
        });
        keys.into_iter().next()
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub(super) struct PoolPerfStats {
    pub(super) allocations: u64,
    pub(super) reuses: u64,
    pub(super) releases: u64,
    pub(super) evictions: u64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PoolPerfSnapshot {
    pub(super) allocations: u64,
    pub(super) reuses: u64,
    pub(super) releases: u64,
    pub(super) evictions: u64,
    pub(super) free_bytes: u64,
    pub(super) free_textures: u64,
}

pub(super) fn estimate_texture_bytes(
    size: (u32, u32),
    format: wgpu::TextureFormat,
    sample_count: u32,
) -> u64 {
    let w = size.0.max(1) as u64;
    let h = size.1.max(1) as u64;
    let bpp = bytes_per_pixel(format) as u64;
    w.saturating_mul(h)
        .saturating_mul(bpp)
        .saturating_mul(sample_count.max(1) as u64)
}

fn bytes_per_pixel(format: wgpu::TextureFormat) -> u32 {
    match format {
        wgpu::TextureFormat::R8Unorm
        | wgpu::TextureFormat::R8Snorm
        | wgpu::TextureFormat::R8Uint
        | wgpu::TextureFormat::R8Sint => 1,

        wgpu::TextureFormat::R16Uint
        | wgpu::TextureFormat::R16Sint
        | wgpu::TextureFormat::R16Float
        | wgpu::TextureFormat::Rg8Unorm
        | wgpu::TextureFormat::Rg8Snorm
        | wgpu::TextureFormat::Rg8Uint
        | wgpu::TextureFormat::Rg8Sint => 2,

        wgpu::TextureFormat::R32Uint
        | wgpu::TextureFormat::R32Sint
        | wgpu::TextureFormat::R32Float
        | wgpu::TextureFormat::Rg16Uint
        | wgpu::TextureFormat::Rg16Sint
        | wgpu::TextureFormat::Rg16Float
        | wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Rgba8Snorm
        | wgpu::TextureFormat::Rgba8Uint
        | wgpu::TextureFormat::Rgba8Sint
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => 4,

        wgpu::TextureFormat::Rg32Uint
        | wgpu::TextureFormat::Rg32Sint
        | wgpu::TextureFormat::Rg32Float
        | wgpu::TextureFormat::Rgba16Uint
        | wgpu::TextureFormat::Rgba16Sint
        | wgpu::TextureFormat::Rgba16Float => 8,

        wgpu::TextureFormat::Rgba32Uint
        | wgpu::TextureFormat::Rgba32Sint
        | wgpu::TextureFormat::Rgba32Float => 16,

        _ => 16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_enforces_budget_by_eviction() {
        let ctx = match pollster::block_on(crate::WgpuContext::new()) {
            Ok(ctx) => ctx,
            Err(_err) => {
                return;
            }
        };

        let mut pool = IntermediatePool::default();
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT;

        let t0 = pool.acquire_texture(&ctx.device, "t0", (256, 256), format, usage, 1);
        let t1 = pool.acquire_texture(&ctx.device, "t1", (128, 128), format, usage, 1);
        let bytes0 = t0.bytes;
        let bytes1 = t1.bytes;

        pool.release(t0);
        pool.release(t1);
        assert_eq!(pool.free_bytes(), bytes0 + bytes1);

        // Keep only the smaller texture.
        pool.enforce_budget(bytes1);
        assert_eq!(pool.free_bytes(), bytes1);

        // Drop everything.
        pool.enforce_budget(0);
        assert_eq!(pool.free_bytes(), 0);
    }
}
