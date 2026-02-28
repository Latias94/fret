use super::*;

#[derive(Debug)]
pub(super) struct CustomEffectV3PyramidScratch {
    pub(super) size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) levels: u32,
    pub(super) full_view: wgpu::TextureView,
    pub(super) mip_views: Vec<wgpu::TextureView>,
}

impl CustomEffectV3PyramidScratch {
    pub(super) fn mip_size(&self, level: u32) -> (u32, u32) {
        let mut w = self.size.0.max(1);
        let mut h = self.size.1.max(1);
        let mut l = 0u32;
        while l < level {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            l += 1;
        }
        (w, h)
    }
}

impl Renderer {
    pub(super) fn ensure_custom_effect_v3_pyramid_scratch(
        &mut self,
        device: &wgpu::Device,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        levels: u32,
    ) -> &CustomEffectV3PyramidScratch {
        let size = (size.0.max(1), size.1.max(1));
        let levels = levels.max(1);
        let recreate = self
            .custom_effect_v3_pyramid_scratch
            .as_ref()
            .is_none_or(|s| s.size != size || s.format != format || s.levels != levels);
        if recreate {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fret custom-effect v3 pyramid scratch"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            let full_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut mip_views = Vec::with_capacity(levels as usize);
            for level in 0..levels {
                mip_views.push(texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("fret custom-effect v3 pyramid mip view"),
                    format: Some(format),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    base_array_layer: 0,
                    array_layer_count: Some(1),
                    usage: None,
                }));
            }

            self.custom_effect_v3_pyramid_scratch = Some(CustomEffectV3PyramidScratch {
                size,
                format,
                levels,
                full_view,
                mip_views,
            });
        }

        self.custom_effect_v3_pyramid_scratch
            .as_ref()
            .expect("pyramid scratch must exist")
    }
}
