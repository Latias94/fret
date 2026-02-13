use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DashPatternV1, DrawOrder, Paint, Scene, SceneOp, StrokeStyleV1};
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
        label: Some("dashed_border_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("dashed_border_conformance readback encoder"),
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
    scale_factor: f32,
) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("dashed_border_conformance output"),
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
            clear: ClearColor(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }),
            scale_factor,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
    read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size)
}

#[test]
fn dashed_border_conformance_stroke_rrect_masks_border_coverage() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let rect_origin = (20.0_f32, 20.0_f32);
    let rect_size = (200.0_f32, 80.0_f32);
    let radius = 16.0_f32;
    let stroke_w = 4.0_f32;

    let dash = DashPatternV1::new(Px(10.0), Px(6.0), Px(0.0));
    let style = StrokeStyleV1 { dash: Some(dash) };

    let mut scene = Scene::default();
    scene.push(SceneOp::StrokeRRect {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(rect_origin.0), Px(rect_origin.1)),
            Size::new(Px(rect_size.0), Px(rect_size.1)),
        ),
        stroke: Edges::all(Px(stroke_w)),
        stroke_paint: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        corner_radii: Corners::all(Px(radius)),
        style,
    });

    fn u(v: f32, sf: f32) -> u32 {
        (v * sf).round() as u32
    }

    for scale_factor in [1.0_f32, 2.0_f32] {
        let size = (u(256.0, scale_factor), u(128.0, scale_factor));
        let pixels = render_and_readback(&ctx, &mut renderer, &scene, size, scale_factor);

        let start_x = rect_origin.0 + radius;
        let y_top = rect_origin.1 + stroke_w * 0.5;

        // Dash-on sample: s=3 (inside [0..dash)).
        let on_x = start_x + 3.0;
        let on = pixel_rgba(
            &pixels,
            size.0,
            u(on_x, scale_factor),
            u(y_top, scale_factor),
        );

        // Dash-off sample: s=13 (inside (dash..dash+gap)).
        let off_x = start_x + 13.0;
        let off = pixel_rgba(
            &pixels,
            size.0,
            u(off_x, scale_factor),
            u(y_top, scale_factor),
        );

        // Interior sample: should remain transparent.
        let mid_x = rect_origin.0 + rect_size.0 * 0.5;
        let mid_y = rect_origin.1 + rect_size.1 * 0.5;
        let mid = pixel_rgba(
            &pixels,
            size.0,
            u(mid_x, scale_factor),
            u(mid_y, scale_factor),
        );

        assert!(
            on[3] > 200,
            "expected dash-on pixel alpha to be high; got rgba={on:?} sf={scale_factor}"
        );
        assert!(
            off[3] < 40,
            "expected dash-off pixel alpha to be low; got rgba={off:?} sf={scale_factor}"
        );
        assert!(
            mid[3] < 5,
            "expected interior pixel alpha to remain transparent; got rgba={mid:?} sf={scale_factor}"
        );
    }
}
