use fret_core::{ColorRange, RectPx, YuvMatrix};

fn align_up(value: u32, align: u32) -> u32 {
    if align == 0 {
        return value;
    }
    value.div_ceil(align).saturating_mul(align)
}

#[doc(hidden)]
pub struct Nv12Planes {
    pub size: (u32, u32),
    pub chroma_size: (u32, u32),
    pub y: wgpu::Texture,
    pub y_view: wgpu::TextureView,
    pub uv: wgpu::Texture,
    pub uv_view: wgpu::TextureView,
}

impl Nv12Planes {
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let (width, height) = size;
        let chroma_size = (width.div_ceil(2), height.div_ceil(2));

        let y = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret nv12 y plane"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let y_view = y.create_view(&wgpu::TextureViewDescriptor::default());

        let uv = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret nv12 uv plane"),
            size: wgpu::Extent3d {
                width: chroma_size.0,
                height: chroma_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let uv_view = uv.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            size,
            chroma_size,
            y,
            y_view,
            uv,
            uv_view,
        }
    }
}

fn repack_rect_u8(
    src_bytes_per_row: u32,
    src: &[u8],
    rect_x_bytes: u32,
    rect_y: u32,
    rect_row_bytes: u32,
    rect_h: u32,
) -> Result<(Vec<u8>, u32), String> {
    let dst_bytes_per_row = align_up(rect_row_bytes, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let dst_len = (dst_bytes_per_row as usize).saturating_mul(rect_h as usize);
    let mut out = vec![0u8; dst_len];

    for row in 0..rect_h {
        let src_row = rect_y.saturating_add(row);
        let src_off = (src_row as usize)
            .saturating_mul(src_bytes_per_row as usize)
            .saturating_add(rect_x_bytes as usize);
        let src_end = src_off.saturating_add(rect_row_bytes as usize);
        if src_end > src.len() {
            return Err("rect exceeds source plane bytes".to_string());
        }

        let dst_off = (row as usize).saturating_mul(dst_bytes_per_row as usize);
        let dst_end = dst_off.saturating_add(rect_row_bytes as usize);
        out[dst_off..dst_end].copy_from_slice(&src[src_off..src_end]);
    }

    Ok((out, dst_bytes_per_row))
}

#[doc(hidden)]
pub fn write_nv12_rect(
    queue: &wgpu::Queue,
    planes: &Nv12Planes,
    rect: RectPx,
    y_bytes_per_row: u32,
    y_plane: &[u8],
    uv_bytes_per_row: u32,
    uv_plane: &[u8],
) -> Result<u64, String> {
    if rect.w == 0 || rect.h == 0 {
        return Ok(0);
    }
    if rect.x >= planes.size.0 || rect.y >= planes.size.1 {
        return Err("rect out of bounds".to_string());
    }
    if rect.x.saturating_add(rect.w) > planes.size.0
        || rect.y.saturating_add(rect.h) > planes.size.1
    {
        return Err("rect out of bounds".to_string());
    }

    // Fast-path: if the source planes already satisfy wgpu's row-alignment requirements and the
    // copy rect starts at x=0, we can upload directly from the caller-provided planes without
    // repacking into an aligned staging buffer.
    //
    // This is a common case for decoders that already provide 256-byte-aligned strides.
    let x_uv = rect.x / 2;
    let y_uv = rect.y / 2;
    let w_uv = rect.w.div_ceil(2);
    let h_uv = rect.h.div_ceil(2);
    let can_direct_upload = rect.x == 0
        && x_uv == 0
        && y_bytes_per_row.is_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        && uv_bytes_per_row.is_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        && y_bytes_per_row >= rect.w
        && uv_bytes_per_row >= w_uv.saturating_mul(2);
    if can_direct_upload {
        let y_off = (rect.y as usize).saturating_mul(y_bytes_per_row as usize);
        let y_needed = (y_bytes_per_row as usize).saturating_mul(rect.h as usize);
        if y_off.saturating_add(y_needed) > y_plane.len() {
            return Err("rect exceeds source plane bytes".to_string());
        }

        let uv_off = (y_uv as usize).saturating_mul(uv_bytes_per_row as usize);
        let uv_needed = (uv_bytes_per_row as usize).saturating_mul(h_uv as usize);
        if uv_off.saturating_add(uv_needed) > uv_plane.len() {
            return Err("rect exceeds source plane bytes".to_string());
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &planes.y,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: rect.x,
                    y: rect.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &y_plane[y_off..y_off.saturating_add(y_needed)],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(y_bytes_per_row),
                rows_per_image: Some(rect.h),
            },
            wgpu::Extent3d {
                width: rect.w,
                height: rect.h,
                depth_or_array_layers: 1,
            },
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &planes.uv,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x_uv,
                    y: y_uv,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &uv_plane[uv_off..uv_off.saturating_add(uv_needed)],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(uv_bytes_per_row),
                rows_per_image: Some(h_uv),
            },
            wgpu::Extent3d {
                width: w_uv,
                height: h_uv,
                depth_or_array_layers: 1,
            },
        );

        let uploaded_y = (y_bytes_per_row as u64).saturating_mul(rect.h as u64);
        let uploaded_uv = (uv_bytes_per_row as u64).saturating_mul(h_uv as u64);
        return Ok(uploaded_y.saturating_add(uploaded_uv));
    }

    // Y plane upload (1 byte per pixel).
    let (y_bytes, y_bpr) =
        repack_rect_u8(y_bytes_per_row, y_plane, rect.x, rect.y, rect.w, rect.h)?;
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &planes.y,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: rect.x,
                y: rect.y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &y_bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(y_bpr),
            rows_per_image: Some(rect.h),
        },
        wgpu::Extent3d {
            width: rect.w,
            height: rect.h,
            depth_or_array_layers: 1,
        },
    );
    let mut uploaded_bytes = (y_bpr as u64).saturating_mul(rect.h as u64);

    // UV plane upload (2 bytes per pixel, half resolution).
    let x_uv = rect.x / 2;
    let y_uv = rect.y / 2;
    let w_uv = rect.w.div_ceil(2);
    let h_uv = rect.h.div_ceil(2);
    if w_uv == 0 || h_uv == 0 {
        return Ok(uploaded_bytes);
    }
    if x_uv.saturating_add(w_uv) > planes.chroma_size.0
        || y_uv.saturating_add(h_uv) > planes.chroma_size.1
    {
        return Err("uv rect out of bounds".to_string());
    }

    let rect_x_uv_bytes = x_uv.saturating_mul(2);
    let rect_row_uv_bytes = w_uv.saturating_mul(2);
    let (uv_bytes, uv_bpr) = repack_rect_u8(
        uv_bytes_per_row,
        uv_plane,
        rect_x_uv_bytes,
        y_uv,
        rect_row_uv_bytes,
        h_uv,
    )?;
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &planes.uv,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: x_uv,
                y: y_uv,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &uv_bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(uv_bpr),
            rows_per_image: Some(h_uv),
        },
        wgpu::Extent3d {
            width: w_uv,
            height: h_uv,
            depth_or_array_layers: 1,
        },
    );

    uploaded_bytes = uploaded_bytes.saturating_add((uv_bpr as u64).saturating_mul(h_uv as u64));

    Ok(uploaded_bytes)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Nv12Params {
    range: u32,
    matrix: u32,
    _pad0: u32,
    _pad1: u32,
}

fn range_u32(range: ColorRange) -> u32 {
    match range {
        ColorRange::Full => 0,
        ColorRange::Limited => 1,
    }
}

fn matrix_u32(matrix: YuvMatrix) -> u32 {
    match matrix {
        YuvMatrix::Bt601 => 0,
        YuvMatrix::Bt709 => 1,
        YuvMatrix::Bt2020 => 2,
    }
}

#[doc(hidden)]
pub struct Nv12GpuConverter {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    params_buffer: wgpu::Buffer,
}

impl Nv12GpuConverter {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret nv12 convert"),
            source: wgpu::ShaderSource::Wgsl(include_str!("yuv_nv12_convert.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fret nv12 convert bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            std::mem::size_of::<Nv12Params>() as u64,
                        ),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret nv12 convert pl"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret nv12 convert pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret nv12 convert params"),
            size: std::mem::size_of::<Nv12Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            params_buffer,
        }
    }

    pub fn convert_rect_into(&self, args: Nv12ConvertRectIntoArgs<'_>) {
        let Nv12ConvertRectIntoArgs {
            device,
            queue,
            dst_view,
            rect,
            y_view,
            uv_view,
            range,
            matrix,
        } = args;

        if rect.w == 0 || rect.h == 0 {
            return;
        }

        let params = Nv12Params {
            range: range_u32(range),
            matrix: matrix_u32(matrix),
            _pad0: 0,
            _pad1: 0,
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret nv12 convert bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(y_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(uv_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.params_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fret nv12 convert encoder"),
        });
        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret nv12 convert pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &bind_group, &[]);
            rp.set_scissor_rect(rect.x, rect.y, rect.w, rect.h);
            rp.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
    }
}

#[doc(hidden)]
pub struct Nv12ConvertRectIntoArgs<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub dst_view: &'a wgpu::TextureView,
    pub rect: RectPx,
    pub y_view: &'a wgpu::TextureView,
    pub uv_view: &'a wgpu::TextureView,
    pub range: ColorRange,
    pub matrix: YuvMatrix,
}
