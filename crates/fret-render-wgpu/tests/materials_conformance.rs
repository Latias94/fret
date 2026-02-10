use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{DrawOrder, MaterialParams, Paint, Scene, SceneOp};
use fret_core::{MaterialDescriptor, MaterialKind, MaterialService};
use fret_render_wgpu::{ClearColor, RenderSceneParams, Renderer, WgpuContext};
use slotmap::Key;
use std::sync::mpsc;

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
        label: Some("materials_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("materials_conformance readback encoder"),
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

fn render_and_readback(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
    scene: &Scene,
    size: (u32, u32),
) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("materials_conformance output"),
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

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &view,
            scene,
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
    read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size)
}

#[test]
fn gpu_dot_grid_material_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let id = renderer
        .register_material(MaterialDescriptor::new(MaterialKind::DotGrid))
        .expect("dot_grid material must register");

    let params = MaterialParams {
        vec4s: [
            // base (black)
            [0.0, 0.0, 0.0, 1.0],
            // fg (white)
            [1.0, 1.0, 1.0, 1.0],
            // spacing/thickness/seed
            [8.0, 8.0, 2.0, 1.0],
            // time/angle/offset
            [0.0, 0.0, 0.0, 0.0],
        ],
    };

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Paint::Material { id, params },
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let dot = pixel_rgba(&pixels, size.0, 4, 4);
    let bg = pixel_rgba(&pixels, size.0, 1, 1);

    assert!(
        dot[3] > 240 && bg[3] > 240,
        "expected opaque alpha: dot={dot:?} bg={bg:?}"
    );
    assert!(
        dot[0] > 200 && dot[1] > 200 && dot[2] > 200,
        "expected bright dot: dot={dot:?}"
    );
    assert!(
        bg[0] < 30 && bg[1] < 30 && bg[2] < 30,
        "expected dark background: bg={bg:?}"
    );
}

#[test]
fn unknown_material_id_degrades_to_transparent() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let params = MaterialParams {
        vec4s: [
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [8.0, 8.0, 2.0, 1.0],
            [0.0, 0.0, 0.0, 0.0],
        ],
    };

    let size = (32u32, 32u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(32.0)));

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Paint::Material {
            id: fret_core::MaterialId::null(),
            params,
        },
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let mid = pixel_rgba(&pixels, size.0, 16, 16);
    assert!(mid[3] < 5, "expected transparent fallback: mid={mid:?}");
}
