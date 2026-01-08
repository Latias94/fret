use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PoolKey {
    size: (u32, u32),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    sample_count: u32,
}

pub(super) struct IntermediatePool {
    free: HashMap<PoolKey, Vec<wgpu::Texture>>,
}

impl Default for IntermediatePool {
    fn default() -> Self {
        Self {
            free: HashMap::new(),
        }
    }
}

pub(super) struct PooledTexture {
    key: PoolKey,
    pub(super) texture: wgpu::Texture,
}

impl IntermediatePool {
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

        if let Some(textures) = self.free.get_mut(&key)
            && let Some(texture) = textures.pop()
        {
            return PooledTexture { key, texture };
        }

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
        PooledTexture { key, texture }
    }

    pub(super) fn release(&mut self, texture: PooledTexture) {
        self.free
            .entry(texture.key)
            .or_default()
            .push(texture.texture);
    }
}
