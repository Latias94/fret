use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_render_wgpu::{ClearColor, RenderSceneParams, Renderer, RendererCapabilities};
use std::sync::mpsc;

fn request_engine_hosted_gpu_objects()
-> Result<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue), String> {
    pollster::block_on(async move {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|err| format!("request_adapter failed: {err}"))?;

        let required_downlevel = wgpu::DownlevelFlags::VERTEX_STORAGE;
        let actual_downlevel = adapter.get_downlevel_capabilities().flags;
        if !actual_downlevel.contains(required_downlevel) {
            return Err(format!(
                "adapter missing required downlevel flags: required={required_downlevel:?} actual={actual_downlevel:?}"
            ));
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("host-provided topology smoke device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::default(),
            })
            .await
            .map_err(|err| format!("request_device failed: {err}"))?;

        Ok((instance, adapter, device, queue))
    })
}

fn read_texture_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> Vec<u8> {
    let (width, height) = size;
    let bytes_per_pixel: u32 = 4;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(256) * 256;
    let buffer_size = padded_bytes_per_row as u64 * height as u64;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("host_provided_gpu_topology_smoke readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("host_provided_gpu_topology_smoke readback encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("map_async channel closed").unwrap();

    let mapped = slice.get_mapped_range();
    let mut pixels = vec![0u8; (unpadded_bytes_per_row * height) as usize];
    for row in 0..height as usize {
        let src = row * padded_bytes_per_row as usize;
        let dst = row * unpadded_bytes_per_row as usize;
        pixels[dst..dst + unpadded_bytes_per_row as usize]
            .copy_from_slice(&mapped[src..src + unpadded_bytes_per_row as usize]);
    }
    drop(mapped);
    buffer.unmap();
    pixels
}

fn pixel_rgba(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [
        pixels[idx],
        pixels[idx + 1],
        pixels[idx + 2],
        pixels[idx + 3],
    ]
}

#[test]
fn renderer_accepts_host_provided_gpu_topology() {
    let Ok((_instance, adapter, device, queue)) = request_engine_hosted_gpu_objects() else {
        return;
    };

    let capabilities = RendererCapabilities::from_adapter_device(&adapter, &device);
    let adapter_info = adapter.get_info();

    assert_eq!(capabilities.adapter.name, adapter_info.name);
    assert_eq!(
        capabilities.max_texture_dimension_2d,
        device.limits().max_texture_dimension_2d
    );

    let mut renderer = Renderer::new(&adapter, &device);
    let size = (32u32, 32u32);
    let format = wgpu::TextureFormat::Rgba8Unorm;

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("host_provided_gpu_topology_smoke output"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(16.0), Px(16.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    let cb = renderer.render_scene(
        &device,
        &queue,
        RenderSceneParams {
            format,
            target_view: &view,
            scene: &scene,
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    queue.submit([cb]);
    let _ = device.poll(wgpu::PollType::wait_indefinitely());

    let pixels = read_texture_rgba8(&device, &queue, &texture, size);
    let inside = pixel_rgba(&pixels, size.0, 16, 16);
    let outside = pixel_rgba(&pixels, size.0, 2, 2);

    assert_eq!(
        inside,
        [255, 0, 0, 255],
        "center pixel should contain the quad"
    );
    assert_eq!(
        outside,
        [0, 0, 0, 0],
        "clear region should remain transparent"
    );
}
