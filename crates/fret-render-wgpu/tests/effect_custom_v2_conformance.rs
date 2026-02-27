use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, CustomEffectImageInputV1, DrawOrder, EffectChain, EffectMode,
    EffectParamsV1, EffectQuality, EffectStep, GradientStop, ImageSamplingHint, LinearGradient,
    MAX_STOPS, Mask, Paint, Scene, SceneOp, TileMode, UvRect,
};
use fret_core::{AlphaMode, CustomEffectDescriptorV2, CustomEffectService, ImageId};
use fret_render_wgpu::{
    ClearColor, ImageColorSpace, ImageDescriptor, RenderSceneParams, Renderer, WgpuContext,
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
        label: Some("effect_custom_v2_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("effect_custom_v2_conformance readback encoder"),
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
        label: Some("effect_custom_v2_conformance output"),
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

fn register_rgba_checkerboard(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
) -> (ImageId, wgpu::Texture) {
    let size = (2u32, 2u32);
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("effect_custom_v2_conformance checkerboard"),
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

    // 2x2 RGBA8 checkerboard:
    // (0,0)=black, (1,0)=white
    // (0,1)=white, (1,1)=black
    let bytes: [u8; 16] = [
        0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
    ];
    ctx.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
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
        alpha_mode: AlphaMode::Opaque,
    });
    (id, texture)
}

#[test]
fn gpu_custom_effect_v2_can_sample_user_image_and_respects_sampling_hint() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let (image, _texture) = register_rgba_checkerboard(&ctx, &mut renderer);

    let wgsl = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // Exercise both `render_space` and the v2 input helpers.
  let local = fret_local_px(pos_px);
  let n = clamp(local.x / max(render_space.size_px.x, 1.0), 0.0, 1.0);
  let c = fret_sample_input_at_pos(pos_px);
  // Nudge red a bit to avoid perfect symmetry in some drivers.
  return vec4<f32>(clamp(c.r + n * 0.0001, 0.0, 1.0), c.g, c.b, c.a);
}
"#;

    let effect = renderer
        .register_custom_effect_v2(CustomEffectDescriptorV2::wgsl_utf8(wgsl))
        .expect("custom effect v2 registration must succeed on wgpu backends");

    let tile_px = 12u32;
    let margin = 2u32;
    let size = (margin * 2 + tile_px * 2, margin * 2 + tile_px);

    let bounds_left = Rect::new(
        Point::new(Px(margin as f32), Px(margin as f32)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );
    let bounds_right = Rect::new(
        Point::new(Px((margin + tile_px) as f32), Px(margin as f32)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );

    let mut scene = Scene::default();

    for (bounds, sampling) in [
        (bounds_left, ImageSamplingHint::Nearest),
        (bounds_right, ImageSamplingHint::Linear),
    ] {
        scene.push(SceneOp::PushEffect {
            bounds,
            mode: EffectMode::FilterContent,
            chain: EffectChain::from_steps(&[EffectStep::CustomV2 {
                id: effect,
                params: EffectParamsV1::ZERO,
                max_sample_offset_px: Px(0.0),
                input_image: Some(CustomEffectImageInputV1 {
                    image,
                    uv: UvRect::FULL,
                    sampling,
                }),
            }]),
            quality: EffectQuality::Auto,
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: bounds,
            background: Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
            corner_radii: Default::default(),
        });
        scene.push(SceneOp::PopEffect);
    }

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    // Outside bounds: remain transparent (proves scissor is working for the effect group).
    let outside = pixel_rgba(&pixels, size.0, 0, 0);
    assert_eq!(
        outside,
        [0, 0, 0, 0],
        "pixels outside effect bounds must remain untouched"
    );

    let mut nearest_has_mid = false;
    let mut linear_has_mid = false;
    // Scan interior to avoid border effects.
    for y in (margin + 1)..(margin + tile_px - 1) {
        for x in (margin + 1)..(margin + tile_px - 1) {
            let p = pixel_rgba(&pixels, size.0, x, y);
            assert_eq!(p[3], 255, "effect output must remain opaque");
            let r = p[0];
            nearest_has_mid |= (16..=239).contains(&r);
        }
        for x in (margin + tile_px + 1)..(margin + tile_px * 2 - 1) {
            let p = pixel_rgba(&pixels, size.0, x, y);
            assert_eq!(p[3], 255, "effect output must remain opaque");
            let r = p[0];
            linear_has_mid |= (16..=239).contains(&r);
        }
    }

    assert!(
        linear_has_mid,
        "expected at least one blended texel for linear sampling in CustomV2"
    );
    assert!(
        !nearest_has_mid,
        "expected no blended texels for nearest sampling in CustomV2"
    );
}

#[test]
fn gpu_custom_effect_v2_with_no_input_image_uses_fallback_texture_and_is_deterministic() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let wgsl = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // If no image is supplied, the backend binds a deterministic 1x1 (0,0,0,0) fallback.
  return fret_sample_input_at_pos(pos_px);
}
"#;
    let effect = renderer
        .register_custom_effect_v2(CustomEffectDescriptorV2::wgsl_utf8(wgsl))
        .expect("custom effect v2 registration must succeed on wgpu backends");

    let bounds = Rect::new(Point::new(Px(4.0), Px(3.0)), Size::new(Px(18.0), Px(12.0)));
    let size = (32u32, 24u32);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::CustomV2 {
            id: effect,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            input_image: None,
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopEffect);

    let pixels_a = render_and_readback(&ctx, &mut renderer, &scene, size);
    let pixels_b = render_and_readback(&ctx, &mut renderer, &scene, size);

    let inside_a = pixel_rgba(&pixels_a, size.0, 8, 8);
    let inside_b = pixel_rgba(&pixels_b, size.0, 8, 8);
    assert_eq!(
        inside_a, inside_b,
        "custom effect v2 must be deterministic for a fixed scene"
    );
    assert_eq!(
        inside_a,
        [0, 0, 0, 0],
        "fallback input texture must sample as transparent black"
    );
}

#[test]
fn gpu_custom_effect_v2_rejects_non_filterable_input_format_by_falling_back() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    // Create a non-filterable float format and register it as an ImageId. The CustomV2 ABI requires
    // filterable sampled textures; the backend should deterministically fall back instead of
    // triggering a wgpu validation error at bind group creation time.
    let size = (1u32, 1u32);
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("effect_custom_v2_conformance non-filterable input"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let non_filterable = renderer.register_image(ImageDescriptor {
        view,
        size,
        format: wgpu::TextureFormat::Rgba32Float,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });

    let wgsl = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // If the input image is incompatible, the backend should bind the deterministic fallback
  // (1x1 transparent black) rather than crashing.
  return fret_sample_input_at_pos(pos_px);
}
"#;
    let effect = renderer
        .register_custom_effect_v2(CustomEffectDescriptorV2::wgsl_utf8(wgsl))
        .expect("custom effect v2 registration must succeed on wgpu backends");

    let bounds = Rect::new(Point::new(Px(3.0), Px(2.0)), Size::new(Px(18.0), Px(12.0)));
    let size = (32u32, 24u32);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::CustomV2 {
            id: effect,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            input_image: Some(CustomEffectImageInputV1 {
                image: non_filterable,
                uv: UvRect::FULL,
                sampling: ImageSamplingHint::Linear,
            }),
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopEffect);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let inside = pixel_rgba(&pixels, size.0, 10, 10);
    assert_eq!(
        inside,
        [0, 0, 0, 0],
        "expected deterministic fallback sampling for incompatible input formats"
    );
}

#[test]
fn gpu_custom_effect_v2_compiles_and_runs_in_masked_path() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let (image, _texture) = register_rgba_checkerboard(&ctx, &mut renderer);

    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  let tint = fret_sample_input(fret_input_uv(pos_px));
  // Ensure both src + input are used in the fragment stage.
  return vec4<f32>(mix(src.rgb, tint.rgb, 0.75), 1.0);
}
"#;
    let effect = renderer
        .register_custom_effect_v2(CustomEffectDescriptorV2::wgsl_utf8(wgsl))
        .expect("custom effect v2 registration must succeed on wgpu backends");

    let size = (48u32, 32u32);
    let bounds = Rect::new(Point::new(Px(6.0), Px(4.0)), Size::new(Px(24.0), Px(18.0)));

    let mut scene = Scene::default();
    // Establish a visible base (outside mask should remain as this base).
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(size.0 as f32), Px(size.1 as f32)),
        ),
        background: Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });

    // Mask: small rounded rect (forces the masked pipeline variant).
    let gradient = LinearGradient {
        start: bounds.origin,
        end: Point::new(bounds.origin.x + bounds.size.width, bounds.origin.y),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count: 2,
        stops: {
            let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
            stops[0] = GradientStop::new(
                0.0,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.0,
                },
            );
            stops[1] = GradientStop::new(
                1.0,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            );
            stops
        },
    };
    scene.push(SceneOp::PushMask {
        bounds,
        mask: Mask::linear_gradient(gradient),
    });
    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::CustomV2 {
            id: effect,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            input_image: Some(CustomEffectImageInputV1 {
                image,
                uv: UvRect::FULL,
                sampling: ImageSamplingHint::Default,
            }),
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: bounds,
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopEffect);
    scene.push(SceneOp::PopMask);

    // Must render without wgpu validation errors.
    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    // A pixel outside the mask should remain black (base), a pixel inside should be non-black.
    let outside = pixel_rgba(&pixels, size.0, 2, 2);
    let inside = pixel_rgba(&pixels, size.0, 12, 10);
    assert!(
        outside[0] < 10 && outside[1] < 10 && outside[2] < 10 && outside[3] > 200,
        "expected base quad outside masked effect"
    );
    assert!(
        inside[0] > 10 || inside[1] > 10 || inside[2] > 10,
        "expected masked custom effect output inside bounds"
    );
}
