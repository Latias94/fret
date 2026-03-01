use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep, Paint,
    Scene, SceneOp,
};
use fret_core::{CustomEffectDescriptorV1, CustomEffectService};
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
        label: Some("effect_custom_v1_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("effect_custom_v1_conformance readback encoder"),
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
        label: Some("effect_custom_v1_conformance output"),
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

#[test]
fn gpu_custom_effect_v1_is_scissored_deterministic_and_preserves_ordering() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let strength = clamp(params.vec4s[0].x, 0.0, 1.0);
  let inv = vec3<f32>(1.0, 1.0, 1.0) - src.rgb;
  let rgb = mix(src.rgb, inv, strength);
  return vec4<f32>(rgb, src.a);
}
"#;

    let effect = renderer
        .register_custom_effect_v1(CustomEffectDescriptorV1::wgsl_utf8(wgsl))
        .expect("custom effect registration must succeed on wgpu backends");

    let params = EffectParamsV1 {
        vec4s: [[0.85, 0.0, 0.0, 0.0], [0.0; 4], [0.0; 4], [0.0; 4]],
    };

    let size = (64u32, 64u32);
    let mut base = Scene::default();

    // Left half white, right half black: sharp edge at x=32.
    base.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    base.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(32.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    // Foreground marker quad, used to assert ordering around the PushEffect boundary.
    let foreground = SceneOp::Quad {
        order: DrawOrder(2),
        rect: Rect::new(
            Point::new(Px(26.0), Px(48.0)),
            Size::new(Px(12.0), Px(12.0)),
        ),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    };

    let mut without_effect = base.clone();
    without_effect.push(foreground);

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::CustomV1 {
            id: effect,
            params,
            max_sample_offset_px: Px(0.0),
        }]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(foreground);
    with_effect.push(SceneOp::PopEffect);

    let direct = render_and_readback(&ctx, &mut renderer, &without_effect, size);
    let effected = render_and_readback(&ctx, &mut renderer, &with_effect, size);
    let effected_again = render_and_readback(&ctx, &mut renderer, &with_effect, size);

    // Outside bounds: unchanged.
    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_effected = pixel_rgba(&effected, size.0, 8, 32);
    assert_eq!(
        outside, outside_effected,
        "pixels outside effect bounds must remain unchanged"
    );

    // Inside bounds near an edge: the custom effect should affect the pixel.
    let inside = pixel_rgba(&direct, size.0, 32, 32);
    let inside_effected = pixel_rgba(&effected, size.0, 32, 32);
    assert_ne!(
        inside, inside_effected,
        "pixels inside effect bounds near an edge should be affected by the custom effect"
    );

    // Deterministic: no hidden time dependency.
    let inside_effected_again = pixel_rgba(&effected_again, size.0, 32, 32);
    assert_eq!(
        inside_effected, inside_effected_again,
        "custom effect must be deterministic for a fixed scene and params"
    );

    // Foreground quad must remain on top (PushEffect is a sequence point).
    let fg = pixel_rgba(&effected, size.0, 32, 56);
    assert!(
        fg[0] > 200 && fg[1] < 40 && fg[2] < 40 && fg[3] > 200,
        "foreground quad should remain visible on top of the custom effect"
    );
}

#[test]
fn gpu_custom_effect_v1_can_read_render_space_in_fragment() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    // Exercise `@group(0) @binding(5) render_space` from the fragment stage.
    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  let p = pos_px - render_space.origin_px;
  let nx = clamp(p.x / max(render_space.size_px.x, 1.0), 0.0, 1.0);
  // Simple left-to-right tint based on render-space normalized x.
  return vec4<f32>(src.r * (1.0 - nx), src.g, src.b * nx, src.a);
}
"#;

    let effect = renderer
        .register_custom_effect_v1(CustomEffectDescriptorV1::wgsl_utf8(wgsl))
        .expect("custom effect registration must succeed on wgpu backends");

    let size = (64u32, 64u32);
    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::CustomV1 {
            id: effect,
            params: EffectParamsV1 {
                vec4s: [[0.0; 4]; 4],
            },
            max_sample_offset_px: Px(0.0),
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::PopEffect);

    // Ensure the pipeline compiles and renders without wgpu validation errors.
    let _pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
}

#[test]
fn gpu_custom_effect_v1_render_space_origin_matches_effect_bounds_scissor() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    // If `render_space.origin_px` is the effect bounds origin (not the viewport origin),
    // then the left-most column inside the effect bounds should see `local_x < 1.0`.
    let wgsl = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  let local = pos_px - render_space.origin_px;
  let t = step(1.0, local.x);
  return vec4<f32>(t, 0.0, 0.0, 1.0);
}
"#;

    let effect = renderer
        .register_custom_effect_v1(CustomEffectDescriptorV1::wgsl_utf8(wgsl))
        .expect("custom effect registration must succeed on wgpu backends");

    let size = (64u32, 64u32);
    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    // Offset bounds from the viewport origin so origin matters.
    let bounds = Rect::new(Point::new(Px(16.0), Px(8.0)), Size::new(Px(24.0), Px(16.0)));
    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::CustomV1 {
            id: effect,
            params: EffectParamsV1 {
                vec4s: [[0.0; 4]; 4],
            },
            max_sample_offset_px: Px(0.0),
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::PopEffect);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    // Pixel inside effect bounds at the left-most column should have t=0 (red=0),
    // and a pixel a couple columns in should have t=1 (red>0).
    let left = pixel_rgba(&pixels, size.0, 16, 8);
    let inner = pixel_rgba(&pixels, size.0, 18, 8);
    assert_eq!(
        left[0], 0,
        "expected render_space.origin_px to match effect bounds origin (left-most column local_x < 1)"
    );
    assert!(
        inner[0] > 0,
        "expected local_x to increase inside bounds (a couple columns in should cross the step threshold)"
    );
}

#[test]
fn gpu_custom_effect_v1_can_sample_renderer_pattern_atlas_helpers() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    // Exercise `fret_catalog_*` helpers from the fragment stage.
    let wgsl = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // Use screen-like coordinates for hash noise, and effect-local coordinates for Bayer to ensure
  // `render_space` is also linked in.
  let n = fret_catalog_hash_noise01(pos_px);
  let b = fret_catalog_bayer8x8_01(fret_local_px(pos_px));
  return vec4<f32>(b, n, 0.0, 1.0);
}
"#;

    let effect = renderer
        .register_custom_effect_v1(CustomEffectDescriptorV1::wgsl_utf8(wgsl))
        .expect("custom effect registration must succeed on wgpu backends");

    let size = (64u32, 64u32);
    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::CustomV1 {
            id: effect,
            params: EffectParamsV1 {
                vec4s: [[0.0; 4]; 4],
            },
            max_sample_offset_px: Px(0.0),
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::PopEffect);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    // The Bayer matrix row 0 is known to differ between x=0 and x=1, so the red channel should differ.
    let p00 = pixel_rgba(&pixels, size.0, 0, 0);
    let p10 = pixel_rgba(&pixels, size.0, 1, 0);
    assert_ne!(
        p00[0], p10[0],
        "expected Bayer helper to produce different values for adjacent pixels"
    );
}

#[test]
fn gpu_custom_effect_v1_chain_padding_matches_expanded_bounds_reference() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    // Custom effect that samples a fixed +8px X offset from the source texture, so it requires
    // padding when it follows a blur in an effect chain.
    let wgsl = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  let dims_u = textureDimensions(src_texture);
  let x = clamp(i32(floor(pos_px.x)) + 8, 0, i32(dims_u.x) - 1);
  let y = clamp(i32(floor(pos_px.y)), 0, i32(dims_u.y) - 1);
  return textureLoad(src_texture, vec2<i32>(x, y), 0);
}
"#;
    let effect = renderer
        .register_custom_effect_v1(CustomEffectDescriptorV1::wgsl_utf8(wgsl))
        .expect("custom effect registration must succeed on wgpu backends");

    let chain = EffectChain::from_steps(&[
        EffectStep::GaussianBlur {
            radius_px: Px(4.0),
            downsample: 2,
        },
        EffectStep::CustomV1 {
            id: effect,
            params: EffectParamsV1 {
                vec4s: [[0.0; 4]; 4],
            },
            max_sample_offset_px: Px(8.0),
        },
    ]);

    let size = (64u32, 64u32);
    let mut base = Scene::default();
    // Left half white, right half black: sharp edge at x=32.
    base.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    base.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(32.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    let bounds_a = Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0)));
    let bounds_b = Rect::new(Point::new(Px(16.0), Px(0.0)), Size::new(Px(32.0), Px(64.0)));

    let mut scene_a = base.clone();
    scene_a.push(SceneOp::PushEffect {
        bounds: bounds_a,
        mode: EffectMode::Backdrop,
        chain,
        quality: EffectQuality::Auto,
    });
    scene_a.push(SceneOp::PopEffect);

    let mut scene_b = base;
    scene_b.push(SceneOp::PushEffect {
        bounds: bounds_b,
        mode: EffectMode::Backdrop,
        chain,
        quality: EffectQuality::Auto,
    });
    scene_b.push(SceneOp::PopEffect);

    let a = render_and_readback(&ctx, &mut renderer, &scene_a, size);
    let b = render_and_readback(&ctx, &mut renderer, &scene_b, size);

    // Within the original bounds, results should match the expanded-bounds reference when padding
    // semantics are correct.
    for x in 24..40 {
        let ya = pixel_rgba(&a, size.0, x, 32);
        let yb = pixel_rgba(&b, size.0, x, 32);
        assert_eq!(
            ya, yb,
            "padding mismatch at x={x}; expected chain evaluation to match expanded bounds"
        );
    }
}
