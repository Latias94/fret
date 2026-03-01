use fret_core::PathService as _;
use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DashPatternV1, DrawOrder, Paint, Scene, SceneOp, StrokeStyleV1};
use fret_core::{Corners, Edges, PathCommand, PathConstraints, PathStyle, StrokeStyleV2};
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
        label: Some("dash_semantics_rrect_vs_path_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("dash_semantics_rrect_vs_path_conformance readback encoder"),
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
        label: Some("dash_semantics_rrect_vs_path_conformance output"),
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

fn u(v: f32, sf: f32) -> u32 {
    (v * sf).round() as u32
}

fn dash_expected_on(s: f32, dash: f32, gap: f32, phase: f32) -> bool {
    let period = dash + gap;
    let period_safe = period.max(1e-6);
    let m = (s + phase).rem_euclid(period_safe);
    m < dash
}

#[test]
fn dash_pattern_phase_matches_between_stroke_rrect_and_path_stroke_v2_for_rects() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let dash = 10.0_f32;
    let gap = 6.0_f32;
    let phases = [-8.0_f32, 0.0_f32, 8.0_f32];
    let stroke = Px(8.0);

    let rect_size = Size::new(Px(200.0), Px(100.0));
    let x_rrect = Px(20.0);
    let x_path = Px(300.0);
    let y0 = Px(20.0);
    let row_gap = Px(140.0);

    let white = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(480.0)),
        ),
        background: (Paint::TRANSPARENT).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let mut row_origins: Vec<(f32, f32, f32)> = Vec::new();
    for (row, phase) in phases.into_iter().enumerate() {
        let y = y0 + Px(row as f32 * row_gap.0);
        row_origins.push((x_rrect.0, x_path.0, y.0));

        let pattern = DashPatternV1::new(Px(dash), Px(gap), Px(phase));

        scene.push(SceneOp::StrokeRRect {
            order: DrawOrder(1 + row as u32 * 10),
            rect: Rect::new(Point::new(x_rrect, y), rect_size),
            stroke: Edges::all(stroke),
            stroke_paint: white.into(),
            corner_radii: Corners::all(Px(0.0)),
            style: StrokeStyleV1 {
                dash: Some(pattern),
            },
        });

        let path_cmds = [
            PathCommand::MoveTo(Point::new(x_path, y)),
            PathCommand::LineTo(Point::new(x_path + rect_size.width, y)),
            PathCommand::LineTo(Point::new(x_path + rect_size.width, y + rect_size.height)),
            PathCommand::LineTo(Point::new(x_path, y + rect_size.height)),
            PathCommand::Close,
        ];
        let constraints = PathConstraints { scale_factor: 1.0 };
        let (path, _metrics) = renderer.prepare(
            &path_cmds,
            PathStyle::StrokeV2(StrokeStyleV2 {
                width: stroke,
                join: fret_core::StrokeJoinV1::Miter,
                cap: fret_core::StrokeCapV1::Butt,
                miter_limit: 4.0,
                dash: Some(pattern),
            }),
            constraints,
        );
        scene.push(SceneOp::Path {
            order: DrawOrder(2 + row as u32 * 10),
            origin: Point::new(Px(0.0), Px(0.0)),
            path,
            paint: white.into(),
        });
    }

    // Sample points inside the top/right stroke bands, away from corners.
    let sample_top_y_offset = stroke.0 * 0.25;
    let sample_right_x_offset = stroke.0 * 0.25;
    let sample_right_y_offset = 30.0_f32;
    let sample_bottom_y_offset = stroke.0 * 0.25;
    let sample_bottom_x_offset = 30.0_f32;
    let sample_left_x_offset = stroke.0 * 0.25;
    // Avoid sampling exactly on a dash boundary (AA region) for the default rect sizes below.
    let sample_left_y_offset = 29.0_f32;

    let s_top_on = 5.0_f32;
    let s_top_off = 12.0_f32;
    let s_right = rect_size.width.0 + sample_right_y_offset;
    let s_bottom =
        rect_size.width.0 + rect_size.height.0 + (rect_size.width.0 - sample_bottom_x_offset);
    let s_left = rect_size.width.0
        + rect_size.height.0
        + rect_size.width.0
        + (rect_size.height.0 - sample_left_y_offset);

    for sf in [1.0_f32, 1.5_f32, 2.0_f32] {
        let size = (u(560.0, sf), u(480.0, sf));
        let pixels = render_and_readback(&ctx, &mut renderer, &scene, size, sf);

        for (row, phase) in phases.into_iter().enumerate() {
            let (xr, xp, y) = row_origins[row];

            let y_top = y + sample_top_y_offset;
            let x_top_on_r = xr + s_top_on;
            let x_top_off_r = xr + s_top_off;
            let x_top_on_p = xp + s_top_on;
            let x_top_off_p = xp + s_top_off;

            let y_right = y + sample_right_y_offset;
            let x_right_r = xr + rect_size.width.0 - sample_right_x_offset;
            let x_right_p = xp + rect_size.width.0 - sample_right_x_offset;

            let y_bottom = y + rect_size.height.0 - sample_bottom_y_offset;
            let x_bottom_r = xr + sample_bottom_x_offset;
            let x_bottom_p = xp + sample_bottom_x_offset;

            let x_left_r = xr + sample_left_x_offset;
            let x_left_p = xp + sample_left_x_offset;
            let y_left = y + sample_left_y_offset;

            let expected_top_on = dash_expected_on(s_top_on, dash, gap, phase);
            let expected_top_off = dash_expected_on(s_top_off, dash, gap, phase);
            let expected_right = dash_expected_on(s_right, dash, gap, phase);
            let expected_bottom = dash_expected_on(s_bottom, dash, gap, phase);
            let expected_left = dash_expected_on(s_left, dash, gap, phase);

            let a_top_on_r = pixel_rgba(&pixels, size.0, u(x_top_on_r, sf), u(y_top, sf))[3];
            let a_top_on_p = pixel_rgba(&pixels, size.0, u(x_top_on_p, sf), u(y_top, sf))[3];

            let a_top_off_r = pixel_rgba(&pixels, size.0, u(x_top_off_r, sf), u(y_top, sf))[3];
            let a_top_off_p = pixel_rgba(&pixels, size.0, u(x_top_off_p, sf), u(y_top, sf))[3];

            let a_right_r = pixel_rgba(&pixels, size.0, u(x_right_r, sf), u(y_right, sf))[3];
            let a_right_p = pixel_rgba(&pixels, size.0, u(x_right_p, sf), u(y_right, sf))[3];

            let a_bottom_r = pixel_rgba(&pixels, size.0, u(x_bottom_r, sf), u(y_bottom, sf))[3];
            let a_bottom_p = pixel_rgba(&pixels, size.0, u(x_bottom_p, sf), u(y_bottom, sf))[3];

            let a_left_r = pixel_rgba(&pixels, size.0, u(x_left_r, sf), u(y_left, sf))[3];
            let a_left_p = pixel_rgba(&pixels, size.0, u(x_left_p, sf), u(y_left, sf))[3];

            let on_hi = 180u8;
            let off_lo = 60u8;

            let assert_onoff = |label: &str, expected_on: bool, a_r: u8, a_p: u8| {
                if expected_on {
                    assert!(
                        a_r > on_hi && a_p > on_hi,
                        "{label}: expected on; got a_rrect={a_r} a_path={a_p} sf={sf} phase={phase}"
                    );
                } else {
                    assert!(
                        a_r < off_lo && a_p < off_lo,
                        "{label}: expected off; got a_rrect={a_r} a_path={a_p} sf={sf} phase={phase}"
                    );
                }
            };

            assert_onoff("top_on", expected_top_on, a_top_on_r, a_top_on_p);
            assert_onoff("top_off", expected_top_off, a_top_off_r, a_top_off_p);
            assert_onoff("right", expected_right, a_right_r, a_right_p);
            assert_onoff("bottom", expected_bottom, a_bottom_r, a_bottom_p);
            assert_onoff("left", expected_left, a_left_r, a_left_p);
        }
    }
}
