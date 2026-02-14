use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    BlendMode, Color, CompositeGroupDesc, DrawOrder, EffectChain, EffectMode, EffectQuality, Paint,
    Scene, SceneOp,
};
use fret_core::{FillRule, FillStyle, PathCommand, PathConstraints, PathService, PathStyle};
use fret_render_wgpu::{
    ClearColor, RenderPlanCompilerFlavor, RenderSceneParams, Renderer, WgpuContext,
};
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
        label: Some("render_plan_compiler_compare_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render_plan_compiler_compare_conformance readback encoder"),
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

fn render_and_readback(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
    scene: &Scene,
    size: (u32, u32),
) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("render_plan_compiler_compare_conformance output"),
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

fn assert_pixels_close(a: &[u8], b: &[u8], tolerance: u8) {
    assert_eq!(a.len(), b.len(), "pixel buffers must have same length");
    let mut worst: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        let d = x.abs_diff(*y);
        worst = worst.max(d);
        assert!(d <= tolerance, "pixel mismatch: d={d} tol={tolerance}");
    }
    let _ = worst;
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

fn build_scene_composite_group_isolated_opacity() -> Scene {
    let size = (64.0, 64.0);
    let full = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(size.0), Px(size.1)),
    );
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let a = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );
    let b = Rect::new(
        Point::new(Px(24.0), Px(24.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: full,
        background: Paint::Solid(Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PushCompositeGroup {
        desc: CompositeGroupDesc::new(bounds, BlendMode::Over, EffectQuality::Auto)
            .with_opacity(0.5),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: a,
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(2),
        rect: b,
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopCompositeGroup);
    scene
}

fn build_scene_clip_path(renderer: &mut Renderer) -> Scene {
    let tri = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(renderer, &tri);

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
    scene
}

fn build_scene_backdrop_color_adjust() -> Scene {
    let size = (64.0, 64.0);
    let full = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(size.0), Px(size.1)),
    );
    let bounds = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: full,
        background: Paint::Solid(Color {
            r: 0.2,
            g: 0.3,
            b: 0.9,
            a: 1.0,
        }),
        border: Default::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });

    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[fret_core::scene::EffectStep::ColorAdjust {
            saturation: 0.0,
            brightness: 0.0,
            contrast: 1.0,
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::PopEffect);

    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: bounds,
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.2,
        }),
        border: Default::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });

    scene
}

#[test]
fn gpu_render_plan_compiler_flavors_match_on_fixed_scene_set() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let size = (64u32, 64u32);

    let cases: Vec<(&str, Box<dyn Fn(&mut Renderer) -> Scene>)> = vec![
        (
            "composite_group_isolated_opacity",
            Box::new(|_r| build_scene_composite_group_isolated_opacity()),
        ),
        ("clip_path_triangle", Box::new(|r| build_scene_clip_path(r))),
        (
            "backdrop_color_adjust",
            Box::new(|_r| build_scene_backdrop_color_adjust()),
        ),
    ];

    for (name, build) in cases {
        let mut legacy = Renderer::new(&ctx.adapter, &ctx.device);
        legacy.set_render_plan_compiler_flavor(RenderPlanCompilerFlavor::Legacy);
        let scene_legacy = build(&mut legacy);

        let mut vnext = Renderer::new(&ctx.adapter, &ctx.device);
        vnext.set_render_plan_compiler_flavor(RenderPlanCompilerFlavor::VNext);
        let scene_vnext = build(&mut vnext);

        let a = render_and_readback(&ctx, &mut legacy, &scene_legacy, size);
        let b = render_and_readback(&ctx, &mut vnext, &scene_vnext, size);

        assert_pixels_close(&a, &b, 2);

        let _ = name;
    }
}
