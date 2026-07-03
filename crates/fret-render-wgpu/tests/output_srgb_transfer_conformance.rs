use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_render_wgpu::{
    ClearColor, RenderSceneParams, RenderSceneSourceSelection, Renderer, WgpuContext,
};

#[path = "support/readback.rs"]
mod support;

use support::{pixel_rgba, read_texture_rgba8};

fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.003_130_8 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

fn approx_u8(actual: u8, expected: u8, tol: u8) -> bool {
    actual.abs_diff(expected) <= tol
}

#[test]
fn gpu_non_srgb_output_applies_explicit_srgb_transfer() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let size = (64u32, 64u32);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("output_srgb_transfer_conformance output"),
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

    let mut scene = Scene::default();
    // Use a mid-tone linear value where sRGB transfer is clearly visible.
    let linear = 0.25f32;
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: linear,
            g: linear,
            b: linear,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &view,
            source: RenderSceneSourceSelection::flat_compat(&scene),
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

    let pixels = read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size);
    let p = pixel_rgba(&pixels, size.0, 32, 32);

    // The renderer currently renders into a linear 8-bit intermediate, then encodes to sRGB in a
    // final output blit. That implies one extra unorm quantization step in linear space.
    let quantized_linear = (linear * 255.0).round() / 255.0;
    let expected = (linear_to_srgb(quantized_linear) * 255.0).round() as u8;

    assert!(
        p[0] > 100 && p[1] > 100 && p[2] > 100,
        "expected explicit sRGB transfer (encoded channel values should be well above linear 0.25 * 255 ~= 64); got {:?}",
        p
    );
    assert!(
        approx_u8(p[0], expected, 2)
            && approx_u8(p[1], expected, 2)
            && approx_u8(p[2], expected, 2),
        "expected encoded RGB ~= {expected} (tol=2), got {:?}",
        p
    );
    assert_eq!(p[3], 255, "opaque alpha should remain 255");
}
