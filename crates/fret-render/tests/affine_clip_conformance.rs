use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size, Transform2D};
use fret_core::scene::{Color, DrawOrder, Scene, SceneOp};
use fret_render::{ClearColor, RenderSceneParams, Renderer, WgpuContext};
use std::sync::mpsc;

fn rotation_about(center: Point, radians: f32) -> Transform2D {
    let (sin, cos) = radians.sin_cos();
    let rot = Transform2D {
        a: cos,
        b: sin,
        c: -sin,
        d: cos,
        tx: 0.0,
        ty: 0.0,
    };

    let neg_center = Point::new(Px(-center.x.0), Px(-center.y.0));
    Transform2D::translation(center) * rot * Transform2D::translation(neg_center)
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
        label: Some("affine_clip_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("affine_clip_conformance readback encoder"),
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
fn clip_rect_is_evaluated_in_clip_local_space_under_affine_transform() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (128u32, 128u32);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("affine_clip_conformance target"),
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
    let center = Point::new(Px(64.0), Px(64.0));
    let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
    scene.push(SceneOp::PushTransform { transform });
    scene.push(SceneOp::PushClipRect {
        rect: Rect::new(
            Point::new(Px(32.0), Px(32.0)),
            Size::new(Px(64.0), Px(64.0)),
        ),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(128.0), Px(128.0)),
        ),
        background: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopClip);
    scene.push(SceneOp::PopTransform);

    let render_cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &view,
            scene: &scene,
            clear: ClearColor(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    ctx.queue.submit([render_cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

    let pixels = read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size);

    // (32,32) is inside the axis-aligned clip rect but outside the rotated clip diamond.
    let outside = pixel_rgba(&pixels, size.0, 32, 32);
    let inside = pixel_rgba(&pixels, size.0, 64, 64);

    assert!(
        inside[3] > 200,
        "expected inside pixel to be opaque, got {inside:?}"
    );
    assert!(
        outside[3] < 20,
        "expected outside pixel to be transparent, got {outside:?}"
    );
}
