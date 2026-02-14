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
        label: Some("clip_path_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("clip_path_conformance readback encoder"),
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
        label: Some("clip_path_conformance output"),
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

fn prepare_fill_path(renderer: &mut Renderer, commands: &[PathCommand]) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::Fill(FillStyle {
            rule: FillRule::NonZero,
        }),
        PathConstraints { scale_factor: 1.0 },
    );
    id
}

#[test]
fn gpu_clip_path_clips_to_shape_not_just_bounds() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let tri = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &tri);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        border: Default::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopClip);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let inside = pixel_rgba(&pixels, size.0, 8, 8);
    assert_eq!(
        inside,
        [0, 255, 0, 255],
        "inside clip-path should be visible"
    );

    let outside = pixel_rgba(&pixels, size.0, 48, 48);
    assert_eq!(
        outside,
        [0, 0, 0, 0],
        "outside clip-path should remain clear"
    );
}

#[test]
fn gpu_clip_path_is_captured_at_push_time_and_does_not_follow_later_transforms() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let square = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(16.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(16.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &square);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });

    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(16.0), Px(16.0))),
        background: Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Default::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });

    scene.push(SceneOp::PushTransform {
        transform: Transform2D::translation(Point::new(Px(32.0), Px(0.0))),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(16.0), Px(16.0))),
        background: Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        border: Default::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopTransform);

    scene.push(SceneOp::PopClip);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let inside = pixel_rgba(&pixels, size.0, 8, 8);
    assert_eq!(
        inside,
        [0, 0, 255, 255],
        "quad drawn inside captured clip-path should be visible"
    );

    let translated = pixel_rgba(&pixels, size.0, 40, 8);
    assert_eq!(
        translated,
        [0, 0, 0, 0],
        "clip-path must not follow transforms pushed after the clip entry"
    );
}

#[test]
fn gpu_clip_path_degrades_to_scissor_only_under_tight_intermediate_budget() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(1024);

    let size = (64u32, 64u32);

    let tri = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &tri);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        border: Default::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopClip);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let outside = pixel_rgba(&pixels, size.0, 48, 48);
    assert_eq!(
        outside,
        [0, 255, 0, 255],
        "under tight budgets, clip-path should deterministically degrade (scissor-only fallback)"
    );
}
