use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    BlendMode, Color, CompositeGroupDesc, DrawOrder, EffectQuality, Paint, Scene, SceneOp,
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
        label: Some("composite_group_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("composite_group_conformance readback encoder"),
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
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("composite_group_conformance output"),
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

fn u8_from_f32_clamped(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

fn linear_to_srgb_f32(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0031308 {
        x * 12.92
    } else {
        let a = 0.055;
        (1.0 + a) * x.powf(1.0 / 2.4) - a
    }
}

fn u8_from_linear_to_srgb_f32(x: f32) -> u8 {
    u8_from_f32_clamped(linear_to_srgb_f32(x))
}

fn assert_rgba_approx_eq(actual: [u8; 4], expected: [u8; 4], tol: u8, context: &str) {
    for i in 0..4 {
        let a = actual[i];
        let e = expected[i];
        let lo = e.saturating_sub(tol);
        let hi = e.saturating_add(tol);
        assert!(
            a >= lo && a <= hi,
            "{context}: channel[{i}] expected≈{expected:?} (tol={tol}) got={actual:?}"
        );
    }
}

#[test]
fn gpu_composite_group_add_is_scissored_and_additive() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let full = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let bounds = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: full,
        background: (Paint::Solid(Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    scene.push(SceneOp::PushCompositeGroup {
        desc: CompositeGroupDesc::new(bounds, BlendMode::Add, EffectQuality::Auto),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: bounds,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.5,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopCompositeGroup);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let outside = pixel_rgba(&pixels, size.0, 8, 8);
    let just_outside = pixel_rgba(&pixels, size.0, 15, 32);
    let inside = pixel_rgba(&pixels, size.0, 32, 32);

    assert!(
        outside[3] > 240 && just_outside[3] > 240 && inside[3] > 240,
        "expected opaque alpha: outside={outside:?} just_outside={just_outside:?} inside={inside:?}"
    );
    assert!(
        outside == just_outside,
        "expected scissor to preserve outside pixels: outside={outside:?} just_outside={just_outside:?} inside={inside:?}"
    );

    assert!(
        inside[0] > outside[0] + 6 && inside[1] > outside[1] + 6 && inside[2] > outside[2] + 6,
        "expected additive blend to brighten inside pixels: outside={outside:?} inside={inside:?}"
    );
}

#[test]
fn gpu_composite_group_blend_modes_v2_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let full = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let bounds = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let dst = Color {
        r: 0.6,
        g: 0.2,
        b: 0.8,
        a: 1.0,
    };
    let src = Color {
        r: 0.1,
        g: 0.5,
        b: 0.3,
        a: 1.0,
    };

    let modes = [BlendMode::Darken, BlendMode::Lighten, BlendMode::Subtract];

    for mode in modes {
        let mut scene = Scene::default();
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: full,
            background: (Paint::Solid(dst)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::TRANSPARENT).into(),
            corner_radii: Corners::all(Px(0.0)),
        });

        scene.push(SceneOp::PushCompositeGroup {
            desc: CompositeGroupDesc::new(bounds, mode, EffectQuality::Auto),
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: bounds,
            background: (Paint::Solid(src)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::TRANSPARENT).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopCompositeGroup);

        let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

        let outside = pixel_rgba(&pixels, size.0, 8, 8);
        let inside = pixel_rgba(&pixels, size.0, 32, 32);

        assert_rgba_approx_eq(
            outside,
            [
                u8_from_linear_to_srgb_f32(dst.r),
                u8_from_linear_to_srgb_f32(dst.g),
                u8_from_linear_to_srgb_f32(dst.b),
                255,
            ],
            4,
            &format!("mode={mode:?} outside"),
        );

        let expected_rgb_srgb = match mode {
            BlendMode::Darken => [
                linear_to_srgb_f32(dst.r.min(src.r)),
                linear_to_srgb_f32(dst.g.min(src.g)),
                linear_to_srgb_f32(dst.b.min(src.b)),
            ],
            BlendMode::Lighten => [
                linear_to_srgb_f32(dst.r.max(src.r)),
                linear_to_srgb_f32(dst.g.max(src.g)),
                linear_to_srgb_f32(dst.b.max(src.b)),
            ],
            BlendMode::Subtract => [
                linear_to_srgb_f32((dst.r - src.r).clamp(0.0, 1.0)),
                linear_to_srgb_f32((dst.g - src.g).clamp(0.0, 1.0)),
                linear_to_srgb_f32((dst.b - src.b).clamp(0.0, 1.0)),
            ],
            BlendMode::Over | BlendMode::Add | BlendMode::Multiply | BlendMode::Screen => {
                unreachable!("modes loop must include only v2 fixed-function modes")
            }
        };

        assert_rgba_approx_eq(
            inside,
            [
                u8_from_f32_clamped(expected_rgb_srgb[0]),
                u8_from_f32_clamped(expected_rgb_srgb[1]),
                u8_from_f32_clamped(expected_rgb_srgb[2]),
                255,
            ],
            6,
            &format!("mode={mode:?} inside"),
        );
    }
}

#[test]
fn gpu_composite_group_opacity_is_isolated_for_overlapping_children() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let full = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let a = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );
    let b = Rect::new(
        Point::new(Px(24.0), Px(24.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let paint = Paint::Solid(Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.5,
    })
    .into();

    let mut stack_opacity_scene = Scene::default();
    stack_opacity_scene.push(SceneOp::PushOpacity { opacity: 0.5 });
    stack_opacity_scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: a,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    stack_opacity_scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: b,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    stack_opacity_scene.push(SceneOp::PopOpacity);

    let mut isolated_opacity_scene = Scene::default();
    isolated_opacity_scene.push(SceneOp::PushCompositeGroup {
        desc: CompositeGroupDesc::new(full, BlendMode::Over, EffectQuality::Auto).with_opacity(0.5),
    });
    isolated_opacity_scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: a,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    isolated_opacity_scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: b,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    isolated_opacity_scene.push(SceneOp::PopCompositeGroup);

    let stack_pixels = render_and_readback(&ctx, &mut renderer, &stack_opacity_scene, size);
    let isolated_pixels = render_and_readback(&ctx, &mut renderer, &isolated_opacity_scene, size);

    let stack_single = pixel_rgba(&stack_pixels, size.0, 18, 18);
    let isolated_single = pixel_rgba(&isolated_pixels, size.0, 18, 18);
    let stack_overlap = pixel_rgba(&stack_pixels, size.0, 32, 32);
    let isolated_overlap = pixel_rgba(&isolated_pixels, size.0, 32, 32);

    // In a non-overlapping region, isolated opacity should match multiplicative opacity.
    for c in 0..4 {
        let a = stack_single[c] as i16;
        let b = isolated_single[c] as i16;
        assert!(
            (a - b).abs() <= 3,
            "expected single-quad pixels to match: stack={stack_single:?} isolated={isolated_single:?}"
        );
    }

    // In an overlapping region, isolated opacity differs from multiplicative opacity (the group
    // alpha is applied after internal blending).
    assert!(
        stack_overlap[3] >= isolated_overlap[3].saturating_add(8),
        "expected isolated overlap alpha to be lower: stack={stack_overlap:?} isolated={isolated_overlap:?}"
    );
}

#[test]
fn gpu_composite_group_opacity_degrades_under_tight_intermediate_budget() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(1024);

    let size = (64u32, 64u32);
    let full = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let a = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );
    let b = Rect::new(
        Point::new(Px(24.0), Px(24.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let paint = Paint::Solid(Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.5,
    })
    .into();

    let mut baseline = Scene::default();
    baseline.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: a,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    baseline.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: b,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let mut degraded = Scene::default();
    degraded.push(SceneOp::PushCompositeGroup {
        desc: CompositeGroupDesc::new(full, BlendMode::Over, EffectQuality::Auto).with_opacity(0.5),
    });
    degraded.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: a,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    degraded.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: b,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    degraded.push(SceneOp::PopCompositeGroup);

    let baseline_pixels = render_and_readback(&ctx, &mut renderer, &baseline, size);
    let degraded_pixels = render_and_readback(&ctx, &mut renderer, &degraded, size);

    let baseline_overlap = pixel_rgba(&baseline_pixels, size.0, 32, 32);
    let degraded_overlap = pixel_rgba(&degraded_pixels, size.0, 32, 32);

    for c in 0..4 {
        let a = baseline_overlap[c] as i16;
        let b = degraded_overlap[c] as i16;
        assert!(
            (a - b).abs() <= 3,
            "expected deterministic degradation to match baseline draws: baseline={baseline_overlap:?} degraded={degraded_overlap:?}"
        );
    }
}
