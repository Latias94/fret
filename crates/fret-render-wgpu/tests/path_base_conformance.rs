use fret_core::geometry::{Point, Px, Rect, Size, Transform2D};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_core::{FillRule, FillStyle, PathCommand, PathConstraints, PathService, PathStyle};
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
        label: Some("path_base_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("path_base_conformance readback encoder"),
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
        label: Some("path_base_conformance output"),
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

fn white() -> Color {
    Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    }
}

fn prepare_fill_path(
    renderer: &mut Renderer,
    commands: &[PathCommand],
    rule: FillRule,
) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::Fill(FillStyle { rule }),
        PathConstraints { scale_factor: 1.0 },
    );
    id
}

fn intersecting_same_winding_regions() -> [PathCommand; 10] {
    [
        PathCommand::MoveTo(Point::new(Px(8.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(56.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(56.0), Px(64.0))),
        PathCommand::LineTo(Point::new(Px(8.0), Px(64.0))),
        PathCommand::Close,
        PathCommand::MoveTo(Point::new(Px(32.0), Px(8.0))),
        PathCommand::LineTo(Point::new(Px(64.0), Px(40.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(72.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(40.0))),
        PathCommand::Close,
    ]
}

fn centered_square_path(half: f32) -> [PathCommand; 5] {
    [
        PathCommand::MoveTo(Point::new(Px(-half), Px(-half))),
        PathCommand::LineTo(Point::new(Px(half), Px(-half))),
        PathCommand::LineTo(Point::new(Px(half), Px(half))),
        PathCommand::LineTo(Point::new(Px(-half), Px(half))),
        PathCommand::Close,
    ]
}

#[test]
fn gpu_path_fill_rules_distinguish_overlapping_winding_regions() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let commands = intersecting_same_winding_regions();

    let non_zero = prepare_fill_path(&mut renderer, &commands, FillRule::NonZero);
    let even_odd = prepare_fill_path(&mut renderer, &commands, FillRule::EvenOdd);

    let white = Paint::Solid(white());
    let mut scene = Scene::default();
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(0.0)),
        path: non_zero,
        paint: white.into(),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: Point::new(Px(0.0), Px(84.0)),
        path: even_odd,
        paint: white.into(),
    });

    let size = (72, 168);
    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let non_zero_overlap = pixel_rgba(&pixels, size.0, 32, 40);
    assert!(
        non_zero_overlap[3] > 200,
        "non-zero fill should cover same-winding intersecting overlap; got {non_zero_overlap:?}"
    );

    let even_odd_single_winding = pixel_rgba(&pixels, size.0, 12, 104);
    assert!(
        even_odd_single_winding[3] > 200,
        "even-odd fill should cover a single-winding lobe; got {even_odd_single_winding:?}"
    );

    let even_odd_overlap = pixel_rgba(&pixels, size.0, 32, 124);
    assert!(
        even_odd_overlap[3] < 40,
        "even-odd fill should clear the intersecting double-winding overlap; got {even_odd_overlap:?}"
    );
}

#[test]
fn gpu_path_transform_and_clip_compose_for_rotated_paths() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let square = centered_square_path(18.0);
    let path = prepare_fill_path(&mut renderer, &square, FillRule::NonZero);

    let center = Point::new(Px(48.0), Px(48.0));
    let transform = Transform2D::rotation_about_radians(std::f32::consts::FRAC_PI_4, center);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipRect {
        rect: Rect::new(
            Point::new(Px(24.0), Px(24.0)),
            Size::new(Px(48.0), Px(36.0)),
        ),
    });
    scene.push(SceneOp::PushTransform { transform });
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: center,
        path,
        paint: (Paint::Solid(white())).into(),
    });
    scene.push(SceneOp::PopTransform);
    scene.push(SceneOp::PopClip);

    let size = (96, 96);
    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let rotated_only_region = pixel_rgba(&pixels, size.0, 48, 27);
    assert!(
        rotated_only_region[3] > 200,
        "rotated path should cover a point outside the unrotated square but inside the clip; got {rotated_only_region:?}"
    );

    let center_pixel = pixel_rgba(&pixels, size.0, 48, 48);
    assert!(
        center_pixel[3] > 200,
        "rotated path center should remain visible; got {center_pixel:?}"
    );

    let clipped_region = pixel_rgba(&pixels, size.0, 48, 69);
    assert!(
        clipped_region[3] < 40,
        "clip rect should clip the transformed path; got {clipped_region:?}"
    );
}
