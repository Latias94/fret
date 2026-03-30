use fret_core::geometry::Point;
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp, TextOutlineV1};
use fret_core::text::TextCommonFallbackInjection;
use fret_core::{FrameId, Px, TextConstraints, TextInput, TextService, TextStyle, TextWrap};
use fret_render_wgpu::{
    ClearColor, RenderSceneParams, Renderer, TextFontFamilyConfig, WgpuContext,
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
        label: Some("text_outline_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("text_outline_conformance readback encoder"),
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
        label: Some("text_outline_conformance output"),
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

    read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size)
}

fn configure_deterministic_fonts(renderer: &mut Renderer) {
    let added = renderer.add_fonts(fret_fonts::test_support::face_blobs(
        fret_fonts::default_profile().faces.iter(),
    ));
    assert!(added > 0, "expected bundled fonts to add at least one face");

    // Keep this test deterministic across machines:
    // - no host-installed system fonts,
    // - explicit font families,
    // - explicit common fallback injection.
    let mut families = TextFontFamilyConfig::default();
    families.ui_sans = vec!["Inter".to_string()];
    families.ui_mono = vec!["JetBrains Mono".to_string()];
    families.common_fallback_injection = TextCommonFallbackInjection::CommonFallback;
    renderer.set_text_font_families(&families);

    let snap = renderer.text_fallback_policy_snapshot(FrameId(1));
    assert!(
        !snap.system_fonts_enabled,
        "expected system fonts to be disabled via FRET_TEXT_SYSTEM_FONTS=0"
    );
    assert_ne!(snap.font_stack_key, 0, "expected a non-zero font stack key");
}

#[test]
fn gpu_text_outline_v1_renders_a_visible_ring_for_mask_glyphs() {
    unsafe {
        std::env::set_var("FRET_TEXT_SYSTEM_FONTS", "0");
    }

    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    configure_deterministic_fonts(&mut renderer);

    let mut style = TextStyle::default();
    style.size = Px(56.0);

    let input = TextInput::Plain {
        text: "Outline".into(),
        style,
    };
    let (blob, _metrics) = renderer.prepare(
        &input,
        TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: Default::default(),
            align: Default::default(),
            scale_factor: 1.0,
        },
    );

    let origin = Point::new(Px(16.0), Px(92.0));
    let size = (320u32, 160u32);

    let fill_paint = Paint::Solid(Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    })
    .into();
    let outline_paint = Paint::Solid(Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    })
    .into();

    let mut fill_scene = Scene::default();
    fill_scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: fill_paint,
        outline: None,
        shadow: None,
    });

    let mut outline_scene = Scene::default();
    outline_scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: fill_paint,
        outline: Some(TextOutlineV1 {
            paint: outline_paint,
            width_px: Px(6.0),
        }),
        shadow: None,
    });

    let fill_pixels = render_and_readback(&ctx, &mut renderer, &fill_scene, size);
    let outline_pixels = render_and_readback(&ctx, &mut renderer, &outline_scene, size);

    let fill_max_alpha = fill_pixels
        .chunks_exact(4)
        .map(|px| px[3])
        .max()
        .unwrap_or(0);
    let outline_max_alpha = outline_pixels
        .chunks_exact(4)
        .map(|px| px[3])
        .max()
        .unwrap_or(0);
    assert!(
        fill_max_alpha > 0,
        "expected fill-only text to render at least one non-transparent pixel"
    );

    // Find any pixel where:
    // - fill-only is (near) transparent,
    // - outline scene is meaningfully non-transparent.
    //
    // We intentionally do not over-constrain by color here: text coverage + gamma correction can
    // make the ring's premultiplied channel values small at the edge.
    let mut found_new_coverage = false;
    let mut max_delta_alpha: u8 = 0;
    for y in 0..size.1 {
        for x in 0..size.0 {
            let a0 = pixel_rgba(&fill_pixels, size.0, x, y)[3];
            let a1 = pixel_rgba(&outline_pixels, size.0, x, y)[3];
            max_delta_alpha = max_delta_alpha.max(a1.saturating_sub(a0));
            if a0 <= 8 && a1 >= 16 {
                found_new_coverage = true;
                break;
            }
        }
        if found_new_coverage {
            break;
        }
    }
    assert!(
        found_new_coverage,
        "expected outline to add new coverage pixels (max_delta_alpha={max_delta_alpha}, fill_max_alpha={fill_max_alpha}, outline_max_alpha={outline_max_alpha})"
    );
}

#[test]
fn gpu_text_outline_v1_invalid_width_sanitizes_to_fill_only() {
    unsafe {
        std::env::set_var("FRET_TEXT_SYSTEM_FONTS", "0");
    }

    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    configure_deterministic_fonts(&mut renderer);

    let mut style = TextStyle::default();
    style.size = Px(48.0);

    let input = TextInput::Plain {
        text: "Hello".into(),
        style,
    };
    let (blob, _metrics) = renderer.prepare(
        &input,
        TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: Default::default(),
            align: Default::default(),
            scale_factor: 1.0,
        },
    );

    let origin = Point::new(Px(16.0), Px(80.0));
    let size = (256u32, 128u32);

    let fill_paint = Paint::Solid(Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    })
    .into();
    let outline_paint = Paint::Solid(Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    })
    .into();

    let mut fill_scene = Scene::default();
    fill_scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: fill_paint,
        outline: None,
        shadow: None,
    });

    let mut invalid_scene = Scene::default();
    invalid_scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: fill_paint,
        outline: Some(TextOutlineV1 {
            paint: outline_paint,
            width_px: Px(0.0),
        }),
        shadow: None,
    });

    let fill_pixels = render_and_readback(&ctx, &mut renderer, &fill_scene, size);
    let invalid_pixels = render_and_readback(&ctx, &mut renderer, &invalid_scene, size);

    assert_eq!(
        fill_pixels, invalid_pixels,
        "expected invalid outline width to sanitize to fill-only output"
    );
}
