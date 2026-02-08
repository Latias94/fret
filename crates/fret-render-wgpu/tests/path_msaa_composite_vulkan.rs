use fret_core::geometry::{Point, Px};
use fret_core::geometry::{Rect, Size};
use fret_core::scene::{Color, DrawOrder, Scene, SceneOp};
use fret_core::{PathCommand, PathConstraints, PathService, PathStyle, StrokeStyle};
use fret_render_wgpu::{ClearColor, RenderSceneParams, Renderer, WgpuContext};
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
        label: Some("path_msaa_composite_vulkan readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("path_msaa_composite_vulkan readback encoder"),
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

fn pixel_bgra(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [
        pixels[idx],
        pixels[idx + 1],
        pixels[idx + 2],
        pixels[idx + 3],
    ]
}

#[test]
fn gpu_path_msaa_composite_vulkan_smoke() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    if ctx.adapter.get_info().backend != wgpu::Backend::Vulkan {
        // This test targets the Vulkan backend specifically.
        return;
    }

    // Force the MSAA+composite codepath even on Vulkan (some configs default it off).
    //
    // SAFETY: test-only process global configuration. Nextest runs tests in separate processes by
    // default, so this is isolated.
    unsafe {
        std::env::set_var("FRET_ALLOW_VULKAN_PATH_MSAA", "1");
        std::env::set_var("FRET_FORCE_PATH_MSAA_SAMPLES", "4");
    }

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (256u32, 256u32);
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("path_msaa_composite_vulkan output"),
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

    let (path_top, _metrics) = renderer.prepare(
        &[
            PathCommand::MoveTo(Point::new(Px(16.0), Px(80.0))),
            PathCommand::LineTo(Point::new(Px(240.0), Px(80.0))),
        ],
        PathStyle::Stroke(StrokeStyle { width: Px(6.0) }),
        PathConstraints { scale_factor: 1.0 },
    );
    let (path_bottom, _metrics) = renderer.prepare(
        &[
            PathCommand::MoveTo(Point::new(Px(16.0), Px(160.0))),
            PathCommand::LineTo(Point::new(Px(240.0), Px(160.0))),
        ],
        PathStyle::Stroke(StrokeStyle { width: Px(6.0) }),
        PathConstraints { scale_factor: 1.0 },
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipRect {
        rect: Rect::new(
            Point::new(Px(64.0), Px(64.0)),
            Size::new(Px(128.0), Px(64.0)),
        ),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(0.0)),
        path: path_top,
        color: Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    });
    scene.push(SceneOp::PopClip);

    // Ensure we generate a second `PathMsaaBatch` pass by changing the clip stack and thus the
    // path draw's uniform index. The old (buggy) renderer wrote the composite quad vertices into
    // a shared buffer at offset 0 for each pass, which meant only the final write was observed by
    // all passes in the same submission.
    scene.push(SceneOp::PushClipRect {
        rect: Rect::new(
            Point::new(Px(64.0), Px(144.0)),
            Size::new(Px(128.0), Px(64.0)),
        ),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(0.0)),
        path: path_bottom,
        color: Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    });
    scene.push(SceneOp::PopClip);

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &view,
            scene: &scene,
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

    let pixels = read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size);
    // BGRA order for Bgra8 outputs.
    let top_line = pixel_bgra(&pixels, size.0, 128, 80);
    let bottom_line = pixel_bgra(&pixels, size.0, 128, 160);

    // Expect the red stroke to be visible at y=80 and the green stroke at y=160.
    assert!(
        top_line[3] > 32 && top_line[2] > 32,
        "expected a visible red pixel at (128, 80), got BGRA={top_line:?}"
    );
    assert!(
        bottom_line[3] > 32 && bottom_line[1] > 32,
        "expected a visible green pixel at (128, 160), got BGRA={bottom_line:?}"
    );
}
