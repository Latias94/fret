use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size, Transform2D};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint, PaintBindingV1,
    PaintEvalSpaceV1, Scene, SceneOp, TileMode,
};
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
        label: Some("paint_eval_space_viewport_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("paint_eval_space_viewport_conformance readback encoder"),
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
        label: Some("paint_eval_space_viewport_conformance output"),
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
            scale_factor,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
    read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size)
}

fn stops_2(a: Color, b: Color) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, a);
    stops[1] = GradientStop::new(1.0, b);
    (stops, 2)
}

fn u(v: f32, sf: f32) -> u32 {
    (v * sf).round() as u32
}

#[test]
fn quad_paint_viewport_px_differs_from_local_px() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let (stops, stop_count) = stops_2(
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    );

    for sf in [1.0_f32, 1.5, 2.0] {
        let size = (u(256.0, sf), u(128.0, sf));
        let rect = Rect::new(
            Point::new(Px(0.0), Px(10.0)),
            Size::new(Px(100.0), Px(60.0)),
        );
        let transform = Transform2D::translation(Point::new(Px(50.0), Px(0.0)));
        let gradient = LinearGradient {
            start: Point::new(Px(0.0), Px(0.0)),
            end: Point::new(Px(100.0), Px(0.0)),
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count,
            stops,
        };
        let paint = Paint::LinearGradient(gradient);

        let mut local_scene = Scene::default();
        local_scene.push(SceneOp::PushTransform { transform });
        local_scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect,
            background: PaintBindingV1::with_eval_space(paint, PaintEvalSpaceV1::LocalPx),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        local_scene.push(SceneOp::PopTransform);

        let mut viewport_scene = Scene::default();
        viewport_scene.push(SceneOp::PushTransform { transform });
        viewport_scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect,
            background: PaintBindingV1::with_eval_space(paint, PaintEvalSpaceV1::ViewportPx),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        viewport_scene.push(SceneOp::PopTransform);

        let local_pixels = render_and_readback(&ctx, &mut renderer, &local_scene, size, sf);
        let viewport_pixels = render_and_readback(&ctx, &mut renderer, &viewport_scene, size, sf);

        // Sample at the same pixel coordinate inside the quad. The transform shifts the quad by +50
        // in viewport pixels without changing its local scene coordinates. LocalPx evaluates at
        // x=50 within the quad, while ViewportPx evaluates at x=100 in the viewport.
        let sample_x = 100.0;
        let sample_y = rect.origin.y.0 + 30.0;
        let local = pixel_rgba(&local_pixels, size.0, u(sample_x, sf), u(sample_y, sf));
        let viewport = pixel_rgba(&viewport_pixels, size.0, u(sample_x, sf), u(sample_y, sf));

        assert!(
            local[3] > 240 && viewport[3] > 240,
            "expected opaque alpha (sf={sf}): local={local:?} viewport={viewport:?}"
        );
        assert!(
            local[0] < viewport[0],
            "expected viewport-space to be brighter at x=100 (sf={sf}): local={local:?} viewport={viewport:?}"
        );
        assert!(
            viewport[0] > 240,
            "expected near-white in viewport-space at x=100 (sf={sf}): viewport={viewport:?}"
        );
        assert!(
            local[0] > 40 && local[0] < 220,
            "expected mid-gray in local-space at x=50 (sf={sf}): local={local:?}"
        );
    }
}
