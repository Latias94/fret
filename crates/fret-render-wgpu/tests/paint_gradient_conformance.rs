use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint, RadialGradient,
    Scene, SceneOp, SweepGradient, TileMode,
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
        label: Some("paint_gradient_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("paint_gradient_conformance readback encoder"),
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
        label: Some("paint_gradient_conformance output"),
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

fn stops_2(a: Color, b: Color) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, a);
    stops[1] = GradientStop::new(1.0, b);
    (stops, 2)
}

fn stops_3(a: Color, b: Color, c: Color) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, a);
    stops[1] = GradientStop::new(0.5, b);
    stops[2] = GradientStop::new(1.0, c);
    (stops, 3)
}

#[test]
fn gpu_linear_gradient_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
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

    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(64.0), Px(0.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Paint::LinearGradient(gradient),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let left = pixel_rgba(&pixels, size.0, 4, 32);
    let mid = pixel_rgba(&pixels, size.0, 32, 32);
    let right = pixel_rgba(&pixels, size.0, 59, 32);

    assert!(
        left[3] > 240 && mid[3] > 240 && right[3] > 240,
        "expected opaque alpha: left={left:?} mid={mid:?} right={right:?}"
    );

    assert!(
        left[0] <= mid[0] && mid[0] <= right[0],
        "red must be non-decreasing: left={left:?} mid={mid:?} right={right:?}"
    );
    assert!(
        left[1] <= mid[1] && mid[1] <= right[1],
        "green must be non-decreasing: left={left:?} mid={mid:?} right={right:?}"
    );
    assert!(
        left[2] <= mid[2] && mid[2] <= right[2],
        "blue must be non-decreasing: left={left:?} mid={mid:?} right={right:?}"
    );

    let dr = right[0].saturating_sub(left[0]);
    let dg = right[1].saturating_sub(left[1]);
    let db = right[2].saturating_sub(left[2]);
    assert!(
        dr >= 8 && dg >= 8 && db >= 8,
        "expected visible gradient range: left={left:?} mid={mid:?} right={right:?}"
    );
}

#[test]
fn gpu_linear_gradient_repeat_tile_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_3(
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    );

    // Use a short gradient span so `t` crosses > 1.0 across the quad.
    // Repeat tiling should bring the `t=0.5` midpoint color back at `t=1.5`.
    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(16.0), Px(0.0)),
        tile_mode: TileMode::Repeat,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Paint::LinearGradient(gradient),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let a = pixel_rgba(&pixels, size.0, 8, 32); // t ~= 0.5
    let b = pixel_rgba(&pixels, size.0, 24, 32); // t ~= 1.5 (repeat -> 0.5)

    assert!(
        a[3] > 240 && b[3] > 240,
        "expected opaque alpha: a={a:?} b={b:?}"
    );
    assert!(
        a[0] > 160 && a[1] < 80 && a[2] < 80,
        "expected red-ish midpoint under repeat tiling: a={a:?}"
    );
    assert!(
        b[0] > 160 && b[1] < 80 && b[2] < 80,
        "expected repeated red-ish midpoint at t=1.5: b={b:?}"
    );
}

#[test]
fn gpu_linear_gradient_mirror_tile_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
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

    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(16.0), Px(0.0)),
        tile_mode: TileMode::Mirror,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Paint::LinearGradient(gradient),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let dark = pixel_rgba(&pixels, size.0, 4, 32); // t ~= 0.25
    let bright = pixel_rgba(&pixels, size.0, 20, 32); // t ~= 1.25 (mirror -> 0.75)

    assert!(
        dark[3] > 240 && bright[3] > 240,
        "expected opaque alpha: dark={dark:?} bright={bright:?}"
    );
    assert!(
        bright[0] > dark[0].saturating_add(40),
        "expected mirror tiling to increase brightness at t=1.25 vs t=0.25: dark={dark:?} bright={bright:?}"
    );
}

#[test]
fn gpu_linear_gradient_oklab_color_space_midpoint_differs_from_srgb() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (96u32, 64u32);
    let left = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(48.0), Px(64.0)));
    let right = Rect::new(Point::new(Px(48.0), Px(0.0)), Size::new(Px(48.0), Px(64.0)));

    let (stops, stop_count) = stops_2(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
    );

    let srgb_gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(48.0), Px(0.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };
    let oklab_gradient = LinearGradient {
        start: Point::new(Px(48.0), Px(0.0)),
        end: Point::new(Px(96.0), Px(0.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Oklab,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: left,
        background: Paint::LinearGradient(srgb_gradient),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: right,
        background: Paint::LinearGradient(oklab_gradient),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let srgb_mid = pixel_rgba(&pixels, size.0, 24, 32);
    let oklab_mid = pixel_rgba(&pixels, size.0, 72, 32);

    assert!(
        srgb_mid[3] > 240 && oklab_mid[3] > 240,
        "expected opaque alpha: srgb_mid={srgb_mid:?} oklab_mid={oklab_mid:?}"
    );

    let dr = u16::from(srgb_mid[0].abs_diff(oklab_mid[0]));
    let dg = u16::from(srgb_mid[1].abs_diff(oklab_mid[1]));
    let db = u16::from(srgb_mid[2].abs_diff(oklab_mid[2]));
    let diff = dr + dg + db;
    assert!(
        diff >= 24,
        "expected Oklab interpolation to differ visibly from sRGB/linear interpolation at the midpoint: srgb_mid={srgb_mid:?} oklab_mid={oklab_mid:?} diff={diff}"
    );
}

#[test]
fn gpu_radial_gradient_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_2(
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    );

    let gradient = RadialGradient {
        center: Point::new(Px(32.0), Px(32.0)),
        radius: Size::new(Px(32.0), Px(32.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Paint::RadialGradient(gradient),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let center = pixel_rgba(&pixels, size.0, 32, 32);
    let corner = pixel_rgba(&pixels, size.0, 2, 2);

    assert!(center[3] > 240 && corner[3] > 240);
    assert!(corner[0] < center[0] && corner[1] < center[1] && corner[2] < center[2]);
}

#[test]
fn gpu_sweep_gradient_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_3(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
    );

    let gradient = SweepGradient {
        center: Point::new(Px(32.0), Px(32.0)),
        start_angle_turns: 0.0,
        end_angle_turns: 1.0,
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Paint::SweepGradient(gradient),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let right = pixel_rgba(&pixels, size.0, 59, 32);
    let left = pixel_rgba(&pixels, size.0, 4, 32);
    let down = pixel_rgba(&pixels, size.0, 32, 59);
    let up = pixel_rgba(&pixels, size.0, 32, 4);

    assert!(
        right[3] > 240 && left[3] > 240 && down[3] > 240 && up[3] > 240,
        "expected opaque alpha: right={right:?} left={left:?} down={down:?} up={up:?}"
    );

    assert!(
        right[0] > right[1] && right[0] > right[2],
        "expected red-dominant at +X (right): {right:?}"
    );
    assert!(
        left[1] > left[0] && left[1] > left[2],
        "expected green-dominant at -X (left): {left:?}"
    );
    assert!(
        down[0] > down[2] && down[1] > down[2],
        "expected red/green mix at +Y (down): {down:?}"
    );
    assert!(
        up[2] > up[0] && up[1] > up[0],
        "expected green/blue mix at -Y (up): {up:?}"
    );
}
