use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, CustomEffectImageInputV1, CustomEffectPyramidRequestV1, CustomEffectSourcesV3,
    DrawOrder, EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep,
    ImageSamplingHint, Paint, Scene, SceneOp, UvRect,
};
use fret_core::{
    AlphaMode, CustomEffectDescriptorV1, CustomEffectDescriptorV2, CustomEffectDescriptorV3,
    CustomEffectService as _, ImageId,
};
use fret_render_wgpu::{
    ClearColor, ImageColorSpace, ImageDescriptor, RenderSceneParams, Renderer, WgpuContext,
};
use std::sync::mpsc;

#[derive(Clone, Copy, Debug)]
enum CustomEffectAbi {
    V1,
    V2,
    V3,
}

fn register_passthrough_custom_effect(
    renderer: &mut Renderer,
    abi: CustomEffectAbi,
) -> fret_core::EffectId {
    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;

    match abi {
        CustomEffectAbi::V1 => renderer
            .register_custom_effect_v1(CustomEffectDescriptorV1::wgsl_utf8(wgsl))
            .expect("custom effect v1 registration must succeed on wgpu backends"),
        CustomEffectAbi::V2 => renderer
            .register_custom_effect_v2(CustomEffectDescriptorV2::wgsl_utf8(wgsl))
            .expect("custom effect v2 registration must succeed on wgpu backends"),
        CustomEffectAbi::V3 => renderer
            .register_custom_effect_v3(CustomEffectDescriptorV3::wgsl_utf8(wgsl))
            .expect("custom effect v3 registration must succeed on wgpu backends"),
    }
}

fn custom_effect_step(abi: CustomEffectAbi, id: fret_core::EffectId) -> EffectStep {
    match abi {
        CustomEffectAbi::V1 => EffectStep::CustomV1 {
            id,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
        },
        CustomEffectAbi::V2 => EffectStep::CustomV2 {
            id,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            input_image: None,
        },
        CustomEffectAbi::V3 => EffectStep::CustomV3 {
            id,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            user0: None,
            user1: None,
            sources: CustomEffectSourcesV3 {
                want_raw: false,
                pyramid: None,
            },
        },
    }
}

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
        label: Some("effect_custom_v3_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("effect_custom_v3_conformance readback encoder"),
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
        label: Some("effect_custom_v3_conformance output"),
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
fn gpu_custom_effect_v3_src_raw_is_chain_root_and_differs_from_src_after_prior_step() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // Encode both sources into output channels:
  // - R: raw chain root (pre-steps)
  // - G: current chain input (post-previous steps)
  let raw = fret_sample_src_raw_at_pos(pos_px);
  return vec4<f32>(raw.r, src.r, 0.0, 1.0);
}
"#;

    let effect = renderer
        .register_custom_effect_v3(CustomEffectDescriptorV3::wgsl_utf8(wgsl))
        .expect("custom effect v3 registration must succeed on wgpu backends");

    let tile_px = 32u32;
    let margin = 2u32;
    let size = (margin * 2 + tile_px, margin * 2 + tile_px);

    let bounds = Rect::new(
        Point::new(Px(margin as f32), Px(margin as f32)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );

    let left = Rect::new(
        Point::new(Px(margin as f32), Px(margin as f32)),
        Size::new(Px((tile_px / 2) as f32), Px(tile_px as f32)),
    );
    let right = Rect::new(
        Point::new(Px((margin + tile_px / 2) as f32), Px(margin as f32)),
        Size::new(Px((tile_px - tile_px / 2) as f32), Px(tile_px as f32)),
    );

    let mut scene = Scene::default();

    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[
            EffectStep::GaussianBlur {
                radius_px: Px(6.0),
                downsample: 2,
            },
            EffectStep::CustomV3 {
                id: effect,
                params: EffectParamsV1::ZERO,
                max_sample_offset_px: Px(0.0),
                user0: None,
                user1: None,
                sources: CustomEffectSourcesV3 {
                    want_raw: true,
                    pyramid: None,
                },
            },
        ]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: left,
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
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: right,
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
    scene.push(SceneOp::PopEffect);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    // Outside bounds: remain transparent (proves scissor is working for the effect group).
    let outside = pixel_rgba(&pixels, size.0, 0, 0);
    assert_eq!(
        outside,
        [0, 0, 0, 0],
        "pixels outside effect bounds must remain untouched"
    );

    // Sample a pixel slightly left of the edge. With blur applied, `src` should have red < 1.0
    // (mixed with the black half) while `src_raw` should remain pure red.
    let sample_x = margin + tile_px / 2 - 2;
    let sample_y = margin + tile_px / 2;
    let p = pixel_rgba(&pixels, size.0, sample_x, sample_y);
    assert_eq!(p[3], 255, "effect output must remain opaque");

    let raw_r = p[0];
    let src_r = p[1];
    assert!(
        raw_r >= 240,
        "expected src_raw to remain close to pure red (got {raw_r})"
    );
    assert!(
        src_r < raw_r.saturating_sub(10),
        "expected src (post-blur) to differ from src_raw (raw_r={raw_r}, src_r={src_r})"
    );
}

#[test]
fn gpu_custom_effect_v3_backdrop_source_group_raw_snapshots_before_prior_backdrop_steps() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);
    renderer.set_perf_enabled(true);

    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // Encode both sources into output channels:
  // - R: group snapshot (raw)
  // - G: current backdrop input (src)
  let raw = fret_sample_src_raw_at_pos(pos_px);
  return vec4<f32>(raw.r, src.r, 0.0, 1.0);
}
"#;

    let effect = renderer
        .register_custom_effect_v3(CustomEffectDescriptorV3::wgsl_utf8(wgsl))
        .expect("custom effect v3 registration must succeed on wgpu backends");

    let tile_px = 32u32;
    let margin = 2u32;
    let size = (margin * 2 + tile_px, margin * 2 + tile_px);

    let bounds = Rect::new(
        Point::new(Px(margin as f32), Px(margin as f32)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );

    let left = Rect::new(
        Point::new(Px(margin as f32), Px(margin as f32)),
        Size::new(Px((tile_px / 2) as f32), Px(tile_px as f32)),
    );
    let right = Rect::new(
        Point::new(Px((margin + tile_px / 2) as f32), Px(margin as f32)),
        Size::new(Px((tile_px - tile_px / 2) as f32), Px(tile_px as f32)),
    );

    let mut scene = Scene::default();

    // Base backdrop content: red/black split tile.
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: left,
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
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: right,
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

    // Snapshot the raw backdrop source once, then apply a blur and a CustomV3 effect. The CustomV3
    // step must see src_raw as the pre-blur snapshot while src reflects the current (blurred) scene.
    scene.push(SceneOp::PushBackdropSourceGroupV1 {
        bounds,
        pyramid: None,
        quality: EffectQuality::Auto,
    });

    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::GaussianBlur {
            radius_px: Px(6.0),
            downsample: 2,
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::PopEffect);

    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::CustomV3 {
            id: effect,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            user0: None,
            user1: None,
            sources: CustomEffectSourcesV3 {
                want_raw: true,
                pyramid: None,
            },
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::PopEffect);

    scene.push(SceneOp::PopBackdropSourceGroup);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let snap = renderer
        .take_last_frame_perf_snapshot()
        .expect("expected last_frame_perf snapshot with perf enabled");
    assert_eq!(
        snap.effect_degradations.backdrop_source_groups.requested, 1,
        "expected one backdrop source group push"
    );
    assert_eq!(
        snap.effect_degradations.backdrop_source_groups.applied_raw, 1,
        "expected group raw snapshot to be applied"
    );
    assert_eq!(
        snap.effect_degradations
            .backdrop_source_groups
            .pyramid_requested,
        0,
        "this conformance does not request a group pyramid"
    );

    let sample_x = margin + tile_px / 2 - 2;
    let sample_y = margin + tile_px / 2;
    let p = pixel_rgba(&pixels, size.0, sample_x, sample_y);
    assert_eq!(p[3], 255, "effect output must remain opaque");

    let raw_r = p[0];
    let src_r = p[1];
    assert!(
        raw_r >= 240,
        "expected src_raw to remain close to pure red (got {raw_r})"
    );
    assert!(
        src_r < raw_r.saturating_sub(10),
        "expected src (post-blur) to differ from src_raw (raw_r={raw_r}, src_r={src_r})"
    );
}

#[test]
fn gpu_custom_effect_v3_requested_but_skipped_under_tight_intermediate_budget() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(1024);
    renderer.set_perf_enabled(true);

    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;

    let effect = renderer
        .register_custom_effect_v3(CustomEffectDescriptorV3::wgsl_utf8(wgsl))
        .expect("custom effect v3 registration must succeed on wgpu backends");

    let size = (32u32, 32u32);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(size.0 as f32), Px(size.1 as f32)),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::CustomV3 {
            id: effect,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            user0: None,
            user1: None,
            sources: CustomEffectSourcesV3 {
                want_raw: true,
                pyramid: Some(CustomEffectPyramidRequestV1 {
                    max_levels: 4,
                    max_radius_px: Px(16.0),
                }),
            },
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
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
    });
    scene.push(SceneOp::PopEffect);

    let _pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let snap = renderer
        .take_last_frame_perf_snapshot()
        .expect("expected last_frame_perf snapshot with perf enabled");
    assert_eq!(
        snap.custom_effect_v3_steps_requested, 1,
        "expected one CustomV3 step to be requested by the effect chain"
    );
    assert_eq!(
        snap.custom_effect_v3_passes_emitted, 0,
        "expected CustomEffectV3 pass emission to be skipped under a tight intermediate budget"
    );
    assert_eq!(
        snap.effect_degradations
            .custom_effect_v3_sources
            .raw_requested,
        0,
        "source counters remain at 0 when the pass is not emitted"
    );
    assert_eq!(
        snap.effect_degradations
            .custom_effect_v3_sources
            .pyramid_requested,
        0,
        "source counters remain at 0 when the pass is not emitted"
    );
}

#[test]
fn gpu_custom_effect_v1_requested_but_skipped_under_tight_intermediate_budget() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(1024);
    renderer.set_perf_enabled(true);

    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;

    let effect = renderer
        .register_custom_effect_v1(CustomEffectDescriptorV1::wgsl_utf8(wgsl))
        .expect("custom effect v1 registration must succeed on wgpu backends");

    let size = (32u32, 32u32);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(size.0 as f32), Px(size.1 as f32)),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::CustomV1 {
            id: effect,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
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
    });
    scene.push(SceneOp::PopEffect);

    let _pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let snap = renderer
        .take_last_frame_perf_snapshot()
        .expect("expected last_frame_perf snapshot with perf enabled");
    assert_eq!(
        snap.custom_effect_v1_steps_requested, 1,
        "expected one CustomV1 step to be requested by the effect chain"
    );
    assert_eq!(
        snap.custom_effect_v1_passes_emitted, 0,
        "expected CustomEffect pass emission to be skipped under a tight intermediate budget"
    );
}

#[test]
fn gpu_custom_effect_v2_requested_but_skipped_under_tight_intermediate_budget() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(1024);
    renderer.set_perf_enabled(true);

    let wgsl = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;

    let effect = renderer
        .register_custom_effect_v2(CustomEffectDescriptorV2::wgsl_utf8(wgsl))
        .expect("custom effect v2 registration must succeed on wgpu backends");

    let size = (32u32, 32u32);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(size.0 as f32), Px(size.1 as f32)),
    );

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
    });
    scene.push(SceneOp::PopEffect);

    let _pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    let snap = renderer
        .take_last_frame_perf_snapshot()
        .expect("expected last_frame_perf snapshot with perf enabled");
    assert_eq!(
        snap.custom_effect_v2_steps_requested, 1,
        "expected one CustomV2 step to be requested by the effect chain"
    );
    assert_eq!(
        snap.custom_effect_v2_passes_emitted, 0,
        "expected CustomEffectV2 pass emission to be skipped under a tight intermediate budget"
    );
}

#[test]
fn gpu_custom_effect_requested_but_skipped_with_budget_zero() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let size = (32u32, 32u32);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(size.0 as f32), Px(size.1 as f32)),
    );

    for abi in [
        CustomEffectAbi::V1,
        CustomEffectAbi::V2,
        CustomEffectAbi::V3,
    ] {
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        renderer.set_intermediate_budget_bytes(0);
        renderer.set_perf_enabled(true);

        let effect = register_passthrough_custom_effect(&mut renderer, abi);

        let mut scene = Scene::default();
        scene.push(SceneOp::PushEffect {
            bounds,
            mode: EffectMode::FilterContent,
            chain: EffectChain::from_steps(&[custom_effect_step(abi, effect)]),
            quality: EffectQuality::Auto,
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: bounds,
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
        });
        scene.push(SceneOp::PopEffect);

        let _pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

        let snap = renderer
            .take_last_frame_perf_snapshot()
            .expect("expected last_frame_perf snapshot with perf enabled");
        assert!(
            snap.render_plan_degradations_budget_zero > 0,
            "expected at least one budget-zero degradation when intermediate budget is zero (abi={abi:?})"
        );
        assert!(
            snap.render_plan_degradations_filter_content_disabled > 0,
            "expected FilterContentDisabled degradation when the effect chain cannot be applied (abi={abi:?})"
        );

        match abi {
            CustomEffectAbi::V1 => {
                assert_eq!(snap.custom_effect_v1_steps_requested, 1);
                assert_eq!(snap.custom_effect_v1_passes_emitted, 0);
            }
            CustomEffectAbi::V2 => {
                assert_eq!(snap.custom_effect_v2_steps_requested, 1);
                assert_eq!(snap.custom_effect_v2_passes_emitted, 0);
            }
            CustomEffectAbi::V3 => {
                assert_eq!(snap.custom_effect_v3_steps_requested, 1);
                assert_eq!(snap.custom_effect_v3_passes_emitted, 0);
            }
        }
    }
}

#[test]
fn gpu_custom_effect_requested_but_skipped_due_to_target_exhaustion() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let size = (32u32, 32u32);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(size.0 as f32), Px(size.1 as f32)),
    );

    // Exhaust all intermediate targets by nesting four FilterContent scopes. The innermost scope
    // still allocates its content target, but has no free scratch target for the custom effect. The
    // render plan should therefore skip custom effect pass emission with a target-exhausted
    // degradation.
    for abi in [
        CustomEffectAbi::V1,
        CustomEffectAbi::V2,
        CustomEffectAbi::V3,
    ] {
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        renderer.set_intermediate_budget_bytes(u64::MAX);
        renderer.set_perf_enabled(true);

        let effect = register_passthrough_custom_effect(&mut renderer, abi);

        let mut scene = Scene::default();
        for _ in 0..3 {
            scene.push(SceneOp::PushEffect {
                bounds,
                mode: EffectMode::FilterContent,
                chain: EffectChain::from_steps(&[]),
                quality: EffectQuality::Auto,
            });
        }
        scene.push(SceneOp::PushEffect {
            bounds,
            mode: EffectMode::FilterContent,
            chain: EffectChain::from_steps(&[custom_effect_step(abi, effect)]),
            quality: EffectQuality::Auto,
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: bounds,
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
        });
        for _ in 0..4 {
            scene.push(SceneOp::PopEffect);
        }

        let _pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

        let snap = renderer
            .take_last_frame_perf_snapshot()
            .expect("expected last_frame_perf snapshot with perf enabled");
        assert!(
            snap.render_plan_degradations_target_exhausted > 0,
            "expected at least one target-exhausted degradation when intermediates are fully occupied (abi={abi:?})"
        );
        assert!(
            snap.render_plan_degradations_filter_content_disabled > 0,
            "expected FilterContentDisabled degradation when the effect chain cannot be applied (abi={abi:?})"
        );

        match abi {
            CustomEffectAbi::V1 => {
                assert_eq!(snap.custom_effect_v1_steps_requested, 1);
                assert_eq!(snap.custom_effect_v1_passes_emitted, 0);
            }
            CustomEffectAbi::V2 => {
                assert_eq!(snap.custom_effect_v2_steps_requested, 1);
                assert_eq!(snap.custom_effect_v2_passes_emitted, 0);
            }
            CustomEffectAbi::V3 => {
                assert_eq!(snap.custom_effect_v3_steps_requested, 1);
                assert_eq!(snap.custom_effect_v3_passes_emitted, 0);
            }
        }
    }
}

#[test]
fn gpu_custom_effect_requested_but_skipped_due_to_budget_insufficient() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let size = (32u32, 32u32);
    let full_target_bytes = size.0 as u64 * size.1 as u64 * 4;
    let budget_insufficient_bytes = full_target_bytes.saturating_mul(2).saturating_sub(1).max(1);

    for abi in [
        CustomEffectAbi::V1,
        CustomEffectAbi::V2,
        CustomEffectAbi::V3,
    ] {
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        renderer.set_intermediate_budget_bytes(budget_insufficient_bytes);
        renderer.set_perf_enabled(true);

        let effect = register_passthrough_custom_effect(&mut renderer, abi);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(size.0 as f32), Px(size.1 as f32)),
        );

        let mut scene = Scene::default();
        scene.push(SceneOp::PushEffect {
            bounds,
            mode: EffectMode::FilterContent,
            chain: EffectChain::from_steps(&[custom_effect_step(abi, effect)]),
            quality: EffectQuality::Auto,
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: bounds,
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
        });
        scene.push(SceneOp::PopEffect);

        let _pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

        let snap = renderer
            .take_last_frame_perf_snapshot()
            .expect("expected last_frame_perf snapshot with perf enabled");
        assert_eq!(
            snap.intermediate_full_target_bytes, full_target_bytes,
            "expected intermediate_full_target_bytes to match the output format/viewport size"
        );
        assert_eq!(
            snap.render_plan_degradations_budget_insufficient, 1,
            "expected exactly one budget-insufficient degradation in this scenario (abi={abi:?})"
        );
        assert_eq!(
            snap.render_plan_degradations_filter_content_disabled, 1,
            "expected the FilterContent effect chain to be disabled due to insufficient budget (abi={abi:?})"
        );

        match abi {
            CustomEffectAbi::V1 => {
                assert_eq!(snap.custom_effect_v1_steps_requested, 1);
                assert_eq!(snap.custom_effect_v1_passes_emitted, 0);
            }
            CustomEffectAbi::V2 => {
                assert_eq!(snap.custom_effect_v2_steps_requested, 1);
                assert_eq!(snap.custom_effect_v2_passes_emitted, 0);
            }
            CustomEffectAbi::V3 => {
                assert_eq!(snap.custom_effect_v3_steps_requested, 1);
                assert_eq!(snap.custom_effect_v3_passes_emitted, 0);
            }
        }
    }
}

#[test]
fn gpu_custom_effect_v3_pyramid_level1_differs_from_raw_near_an_unaligned_edge() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let wgsl = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  let raw = fret_sample_src_raw_at_pos(pos_px);
  let pyr = fret_sample_src_pyramid_at_pos(1u, pos_px);
  return vec4<f32>(raw.r, 0.0, pyr.r, 1.0);
}
"#;

    let effect = renderer
        .register_custom_effect_v3(CustomEffectDescriptorV3::wgsl_utf8(wgsl))
        .expect("custom effect v3 registration must succeed on wgpu backends");

    let tile_px = 32u32;
    let margin = 2u32;
    let size = (margin * 2 + tile_px, margin * 2 + tile_px);

    let bounds = Rect::new(
        Point::new(Px(margin as f32), Px(margin as f32)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );

    // Make the edge land on an odd x so that the 2x2 box downsample straddles it.
    let left_w = tile_px / 2 + 1;
    let right_w = tile_px - left_w;
    let edge_x = margin + left_w;

    let left = Rect::new(
        Point::new(Px(margin as f32), Px(margin as f32)),
        Size::new(Px(left_w as f32), Px(tile_px as f32)),
    );
    let right = Rect::new(
        Point::new(Px(edge_x as f32), Px(margin as f32)),
        Size::new(Px(right_w as f32), Px(tile_px as f32)),
    );

    let mut scene = Scene::default();

    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::CustomV3 {
            id: effect,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            user0: None,
            user1: None,
            sources: CustomEffectSourcesV3 {
                want_raw: true,
                pyramid: Some(CustomEffectPyramidRequestV1 {
                    max_levels: 3,
                    max_radius_px: Px(24.0),
                }),
            },
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: left,
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
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: right,
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
    scene.push(SceneOp::PopEffect);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);

    // Sample a pixel just inside the left (red) half. With the edge on an odd x, this still maps
    // to a mip level 1 texel whose 2x2 source block straddles the red/black boundary.
    let sample_x = edge_x - 1;
    let sample_y = margin + tile_px / 2;
    let p = pixel_rgba(&pixels, size.0, sample_x, sample_y);
    assert_eq!(p[3], 255, "effect output must remain opaque");

    let raw_r = p[0];
    let pyr_r = p[2];
    assert!(
        raw_r >= 160,
        "expected src_raw to remain strongly red (got {raw_r})"
    );
    assert!(
        pyr_r >= 16 && pyr_r < raw_r.saturating_sub(8),
        "expected mip level 1 to differ near the edge (raw_r={raw_r}, pyr_r={pyr_r})"
    );
}

#[test]
fn gpu_custom_effect_v3_rejects_non_filterable_user_image_formats_by_falling_back_and_counts_it() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);
    renderer.set_perf_enabled(true);

    // Create a non-filterable float format and register it as an ImageId. The CustomV3 ABI
    // requires filterable sampled textures for `user0` / `user1`; the backend should
    // deterministically fall back instead of triggering a wgpu validation error at bind group
    // creation time.
    let size = (1u32, 1u32);
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("effect_custom_v3_conformance non-filterable user0"),
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
    let non_filterable: ImageId = renderer.register_image(ImageDescriptor {
        view,
        size,
        format: wgpu::TextureFormat::Rgba32Float,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });

    let wgsl = r#"
fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  // If the user image is incompatible, the backend should bind the deterministic fallback
  // (1x1 transparent black) rather than crashing.
  return fret_sample_user0_at_pos(pos_px);
}
"#;
    let effect = renderer
        .register_custom_effect_v3(CustomEffectDescriptorV3::wgsl_utf8(wgsl))
        .expect("custom effect v3 registration must succeed on wgpu backends");

    let bounds = Rect::new(Point::new(Px(3.0), Px(2.0)), Size::new(Px(18.0), Px(12.0)));
    let size = (32u32, 24u32);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::CustomV3 {
            id: effect,
            params: EffectParamsV1::ZERO,
            max_sample_offset_px: Px(0.0),
            user0: Some(CustomEffectImageInputV1 {
                image: non_filterable,
                uv: UvRect::FULL,
                sampling: ImageSamplingHint::Linear,
            }),
            user1: None,
            sources: CustomEffectSourcesV3 {
                want_raw: false,
                pyramid: None,
            },
        }]),
        quality: EffectQuality::Auto,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
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
    scene.push(SceneOp::PopEffect);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let inside = pixel_rgba(&pixels, size.0, 10, 10);
    assert_eq!(
        inside,
        [0, 0, 0, 0],
        "expected deterministic fallback sampling for incompatible user image formats"
    );

    let perf = renderer
        .take_last_frame_perf_snapshot()
        .expect("expected a last-frame perf snapshot when perf is enabled");
    assert_eq!(
        perf.custom_effect_v3_user0_image_incompatible_fallbacks, 1,
        "expected one incompatible user0 fallback for the rendered CustomEffectV3 pass"
    );
    assert_eq!(
        perf.custom_effect_v3_user1_image_incompatible_fallbacks, 0,
        "expected no user1 fallbacks when user1 is not provided"
    );
}
