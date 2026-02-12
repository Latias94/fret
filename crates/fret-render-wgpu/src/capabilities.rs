use crate::WgpuContext;

#[derive(Debug, Clone)]
pub struct AdapterCapabilities {
    pub backend: String,
    pub name: String,
    pub device_type: String,
    pub driver: String,
    pub driver_info: String,
    pub vendor: u32,
    pub device: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StreamingImageCapabilities {
    pub nv12_gpu_convert: bool,
    pub i420_gpu_convert: bool,
    pub external_texture_import: bool,
}

#[derive(Debug, Clone)]
pub struct RendererCapabilities {
    pub adapter: AdapterCapabilities,
    pub max_texture_dimension_2d: u32,
    pub streaming_images: StreamingImageCapabilities,
    pub sampled_materials_catalog_textures: bool,
}

impl RendererCapabilities {
    pub fn from_wgpu_context(ctx: &WgpuContext) -> Self {
        let info = ctx.adapter.get_info();

        let streaming_images = StreamingImageCapabilities {
            nv12_gpu_convert: supports_nv12_gpu_convert(&ctx.adapter),
            i420_gpu_convert: false,
            external_texture_import: false,
        };

        Self {
            adapter: AdapterCapabilities {
                backend: format!("{:?}", info.backend),
                name: info.name,
                device_type: format!("{:?}", info.device_type),
                driver: info.driver,
                driver_info: info.driver_info,
                vendor: info.vendor,
                device: info.device,
            },
            max_texture_dimension_2d: ctx.device.limits().max_texture_dimension_2d,
            streaming_images,
            sampled_materials_catalog_textures: supports_material_catalog_textures(&ctx.adapter),
        }
    }
}

fn supports_nv12_gpu_convert(adapter: &wgpu::Adapter) -> bool {
    let y = adapter.get_texture_format_features(wgpu::TextureFormat::R8Unorm);
    if !y
        .allowed_usages
        .contains(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
    {
        return false;
    }

    let uv = adapter.get_texture_format_features(wgpu::TextureFormat::Rg8Unorm);
    if !uv
        .allowed_usages
        .contains(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
    {
        return false;
    }

    let dst = adapter.get_texture_format_features(wgpu::TextureFormat::Rgba8UnormSrgb);
    if !dst
        .allowed_usages
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
    {
        return false;
    }

    true
}

fn supports_material_catalog_textures(adapter: &wgpu::Adapter) -> bool {
    // v2 catalog textures use a conservative and widely supported format.
    let f = adapter.get_texture_format_features(wgpu::TextureFormat::Rgba8Unorm);
    if !f
        .allowed_usages
        .contains(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
    {
        return false;
    }
    f.flags
        .contains(wgpu::TextureFormatFeatureFlags::FILTERABLE)
}
