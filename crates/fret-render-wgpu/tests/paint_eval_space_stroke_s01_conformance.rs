use fret_core::PathService;
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint, PaintBindingV1,
    PaintEvalSpaceV1, Scene, SceneOp, StrokeStyleV1, TileMode,
};
use fret_core::{
    PathCommand, PathConstraints, PathStyle, StrokeCapV1, StrokeJoinV1, StrokeStyleV2,
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
        label: Some("paint_eval_space_stroke_s01_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("paint_eval_space_stroke_s01_conformance readback encoder"),
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
        label: Some("paint_eval_space_stroke_s01_conformance output"),
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

fn prepare_stroke_path(
    renderer: &mut Renderer,
    commands: &[PathCommand],
    scale_factor: f32,
) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::StrokeV2(StrokeStyleV2 {
            width: Px(10.0),
            join: StrokeJoinV1::Round,
            cap: StrokeCapV1::Round,
            miter_limit: 4.0,
            dash: None,
        }),
        PathConstraints { scale_factor },
    );
    id
}

#[test]
fn stroke_rrect_paint_stroke_s01_varies_along_perimeter() {
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
        let size = (u(64.0, sf), u(64.0, sf));
        let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
        let gradient = LinearGradient {
            start: Point::new(Px(0.0), Px(0.0)),
            end: Point::new(Px(1.0), Px(0.0)),
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count,
            stops,
        };

        let mut scene = Scene::default();
        scene.push(SceneOp::StrokeRRect {
            order: DrawOrder(0),
            rect,
            stroke: Edges::all(Px(8.0)),
            stroke_paint: PaintBindingV1::with_eval_space(
                Paint::LinearGradient(gradient),
                PaintEvalSpaceV1::StrokeS01,
            ),
            corner_radii: Corners::all(Px(8.0)),
            style: StrokeStyleV1 { dash: None },
        });

        let pixels = render_and_readback(&ctx, &mut renderer, &scene, size, sf);

        // Sample two points that are far apart along the perimeter parameterization:
        // - top edge (near start of s)
        // - left edge (near end of s)
        let top = pixel_rgba(&pixels, size.0, u(32.0, sf), u(4.0, sf));
        let left = pixel_rgba(&pixels, size.0, u(4.0, sf), u(32.0, sf));

        assert!(
            top[3] > 200 && left[3] > 200,
            "expected visible alpha (sf={sf}): top={top:?} left={left:?}"
        );
        assert!(
            left[0] > top[0],
            "expected left edge to be brighter than top edge (sf={sf}): top={top:?} left={left:?}"
        );
    }
}

#[test]
fn path_stroke_paint_stroke_s01_is_monotonic_along_path() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let commands = [
        PathCommand::MoveTo(Point::new(Px(4.0), Px(4.0))),
        PathCommand::LineTo(Point::new(Px(60.0), Px(4.0))),
        PathCommand::LineTo(Point::new(Px(60.0), Px(60.0))),
    ];

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
        let size = (u(64.0, sf), u(64.0, sf));
        let path = prepare_stroke_path(&mut renderer, &commands, sf);
        let gradient = LinearGradient {
            start: Point::new(Px(0.0), Px(0.0)),
            end: Point::new(Px(1.0), Px(0.0)),
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count,
            stops,
        };

        let mut scene = Scene::default();
        scene.push(SceneOp::Path {
            order: DrawOrder(0),
            origin: Point::new(Px(0.0), Px(0.0)),
            path,
            paint: PaintBindingV1::with_eval_space(
                Paint::LinearGradient(gradient),
                PaintEvalSpaceV1::StrokeS01,
            ),
        });

        let pixels = render_and_readback(&ctx, &mut renderer, &scene, size, sf);

        let start = pixel_rgba(&pixels, size.0, u(8.0, sf), u(4.0, sf));
        let mid = pixel_rgba(&pixels, size.0, u(60.0, sf), u(8.0, sf));
        let end = pixel_rgba(&pixels, size.0, u(60.0, sf), u(56.0, sf));

        assert!(
            start[3] > 100 && mid[3] > 100 && end[3] > 100,
            "expected visible alpha (sf={sf}): start={start:?} mid={mid:?} end={end:?}"
        );

        assert!(
            start[0] <= mid[0] && mid[0] <= end[0],
            "expected monotonic red along s01 (sf={sf}): start={start:?} mid={mid:?} end={end:?}"
        );
    }
}
