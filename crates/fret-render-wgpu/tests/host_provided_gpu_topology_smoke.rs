use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_render_wgpu::{ClearColor, RenderSceneParams, Renderer, RendererCapabilities};

#[path = "support/readback.rs"]
mod support;

use support::{pixel_rgba, read_texture_rgba8};

fn request_engine_hosted_gpu_objects()
-> Result<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue), String> {
    pollster::block_on(async move {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
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
            scene_chunks: None,
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
