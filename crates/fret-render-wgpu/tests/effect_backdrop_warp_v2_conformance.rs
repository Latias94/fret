use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    BackdropWarpFieldV2, BackdropWarpKindV1, BackdropWarpV1, BackdropWarpV2, Color, DrawOrder,
    EffectChain, EffectMode, EffectQuality, EffectStep, ImageSamplingHint, Paint, Scene, SceneOp,
    UvRect, WarpMapEncodingV1,
};
use fret_render_wgpu::{
    ClearColor, ImageColorSpace, ImageDescriptor, RenderSceneParams, Renderer, WgpuContext,
};
use slotmap::Key;
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
        label: Some("effect_backdrop_warp_v2_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("effect_backdrop_warp_v2_conformance readback encoder"),
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
        label: Some("effect_backdrop_warp_v2_conformance output"),
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

fn stripe_scene_base(size: (u32, u32)) -> Scene {
    let mut base = Scene::default();
    base.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(size.0 as f32), Px(size.1 as f32)),
        ),
        background: Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });

    for i in 0..16u32 {
        let x = 24.0 + i as f32;
        let is_red = (i % 2) == 0;
        let bg = if is_red {
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else {
            Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }
        };
        base.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: Rect::new(
                Point::new(Px(x), Px(0.0)),
                Size::new(Px(1.0), Px(size.1 as f32)),
            ),
            background: Paint::Solid(bg),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
            corner_radii: Default::default(),
        });
    }
    base
}

fn register_constant_warp_map_rg_signed(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
    rgba: [u8; 4],
) -> (fret_core::ImageId, wgpu::Texture) {
    let size = (1u32, 1u32);
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("effect_backdrop_warp_v2_conformance warp map"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    ctx.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.0),
            rows_per_image: Some(size.1),
        },
        wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let id = renderer.register_image(ImageDescriptor {
        view: view.clone(),
        size,
        format: wgpu::TextureFormat::Rgba8Unorm,
        color_space: ImageColorSpace::Linear,
        alpha_mode: fret_core::AlphaMode::Opaque,
    });
    (id, texture)
}

#[test]
fn gpu_backdrop_warp_v2_image_map_is_scissored_and_preserves_ordering() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let size = (64u32, 64u32);
    let mut base = stripe_scene_base(size);

    // Constant (+X) displacement in RgSigned encoding:
    // r=255 => +1, g=128 => ~0.
    let (warp_image, _keep_alive) =
        register_constant_warp_map_rg_signed(&ctx, &mut renderer, [255, 128, 128, 255]);

    let foreground = SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(
            Point::new(Px(26.0), Px(48.0)),
            Size::new(Px(12.0), Px(12.0)),
        ),
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    };

    let mut without_effect = base.clone();
    without_effect.push(foreground);

    base.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::BackdropWarpV2(BackdropWarpV2 {
            base: BackdropWarpV1 {
                strength_px: Px(12.0),
                scale_px: Px(10.0),
                phase: 0.0,
                chromatic_aberration_px: Px(0.0),
                kind: BackdropWarpKindV1::Wave,
            },
            field: BackdropWarpFieldV2::ImageDisplacementMap {
                image: warp_image,
                uv: UvRect::FULL,
                sampling: ImageSamplingHint::Nearest,
                encoding: WarpMapEncodingV1::RgSigned,
            },
        })]),
        quality: EffectQuality::Auto,
    });
    base.push(foreground);
    base.push(SceneOp::PopEffect);

    let direct = render_and_readback(&ctx, &mut renderer, &without_effect, size);
    let warped = render_and_readback(&ctx, &mut renderer, &base, size);

    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_warped = pixel_rgba(&warped, size.0, 8, 32);
    assert_eq!(
        outside, outside_warped,
        "pixels outside effect bounds must remain unchanged"
    );

    let samples = [(25u32, 10u32), (26u32, 32u32), (37u32, 20u32)];
    let mut changed = 0u32;
    for (x, y) in samples {
        if pixel_rgba(&direct, size.0, x, y) != pixel_rgba(&warped, size.0, x, y) {
            changed += 1;
        }
    }
    assert!(
        changed > 0,
        "expected image-driven backdrop-warp to modify at least one sampled pixel inside bounds"
    );

    let fg = pixel_rgba(&warped, size.0, 32, 56);
    assert!(
        fg[0] > 200 && fg[1] > 200 && fg[2] > 200 && fg[3] > 200,
        "foreground quad should remain visible on top of the warped backdrop"
    );
}

#[test]
fn gpu_backdrop_warp_v2_missing_warp_image_falls_back_to_procedural() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let size = (64u32, 64u32);
    let base = stripe_scene_base(size);

    let base_params = BackdropWarpV1 {
        strength_px: Px(12.0),
        scale_px: Px(10.0),
        phase: 0.0,
        chromatic_aberration_px: Px(0.0),
        kind: BackdropWarpKindV1::Wave,
    };

    let missing_image = fret_core::ImageId::null();

    let mut with_missing_image = base.clone();
    with_missing_image.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::BackdropWarpV2(BackdropWarpV2 {
            base: base_params,
            field: BackdropWarpFieldV2::ImageDisplacementMap {
                image: missing_image,
                uv: UvRect::FULL,
                sampling: ImageSamplingHint::Nearest,
                encoding: WarpMapEncodingV1::RgSigned,
            },
        })]),
        quality: EffectQuality::Auto,
    });
    with_missing_image.push(SceneOp::PopEffect);

    let mut with_procedural = base;
    with_procedural.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::BackdropWarpV2(BackdropWarpV2 {
            base: base_params,
            field: BackdropWarpFieldV2::Procedural,
        })]),
        quality: EffectQuality::Auto,
    });
    with_procedural.push(SceneOp::PopEffect);

    let missing = render_and_readback(&ctx, &mut renderer, &with_missing_image, size);
    let procedural = render_and_readback(&ctx, &mut renderer, &with_procedural, size);

    for (x, y) in [(8u32, 32u32), (25u32, 10u32), (37u32, 20u32)] {
        assert_eq!(
            pixel_rgba(&missing, size.0, x, y),
            pixel_rgba(&procedural, size.0, x, y),
            "expected missing warp image to deterministically fall back to procedural (pixel {x},{y})"
        );
    }
}

#[test]
fn gpu_filter_content_warp_v2_is_deterministically_ignored() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let size = (64u32, 64u32);
    let base = stripe_scene_base(size);

    let quad = SceneOp::Quad {
        order: DrawOrder(200),
        rect: Rect::new(
            Point::new(Px(24.0), Px(16.0)),
            Size::new(Px(16.0), Px(16.0)),
        ),
        background: Paint::Solid(Color {
            r: 0.9,
            g: 0.9,
            b: 0.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    };

    let mut filter_content_noop = base.clone();
    filter_content_noop.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[]),
        quality: EffectQuality::Auto,
    });
    filter_content_noop.push(quad);
    filter_content_noop.push(SceneOp::PopEffect);

    let mut filter_content_warp = base;
    filter_content_warp.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::BackdropWarpV2(BackdropWarpV2 {
            base: BackdropWarpV1 {
                strength_px: Px(12.0),
                scale_px: Px(10.0),
                phase: 0.0,
                chromatic_aberration_px: Px(0.0),
                kind: BackdropWarpKindV1::Wave,
            },
            field: BackdropWarpFieldV2::Procedural,
        })]),
        quality: EffectQuality::Auto,
    });
    filter_content_warp.push(quad);
    filter_content_warp.push(SceneOp::PopEffect);

    let noop = render_and_readback(&ctx, &mut renderer, &filter_content_noop, size);
    let filtered = render_and_readback(&ctx, &mut renderer, &filter_content_warp, size);

    for (x, y) in [(25u32, 20u32), (30u32, 24u32), (39u32, 30u32)] {
        assert_eq!(
            pixel_rgba(&noop, size.0, x, y),
            pixel_rgba(&filtered, size.0, x, y),
            "FilterContent must ignore BackdropWarpV2 deterministically (pixel {x},{y})"
        );
    }
}
