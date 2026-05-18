use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_color_matrix_swaps_channels() {
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

    let quad = SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
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

    let mut direct = Scene::default();
    direct.push(quad);

    // Output.r = input.b, Output.g = input.g, Output.b = input.r, Output.a = input.a.
    let swap_rb: [f32; 20] = [
        0.0, 0.0, 1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, 0.0, //
        1.0, 0.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 1.0, 0.0, //
    ];

    let mut filtered = Scene::default();
    filtered.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::ColorMatrix { m: swap_rb }]),
        quality: EffectQuality::Auto,
    });
    filtered.push(quad);
    filtered.push(SceneOp::PopEffect);

    let direct_pixels = render_scene_rgba8(&ctx, &mut renderer, &direct, size, 1.0);
    let filtered_pixels = render_scene_rgba8(&ctx, &mut renderer, &filtered, size, 1.0);

    let direct_px = pixel_rgba(&direct_pixels, size.0, 32, 32);
    assert!(
        direct_px[0] > 200 && direct_px[1] < 40 && direct_px[2] < 40 && direct_px[3] > 200,
        "direct pixel should be red"
    );

    let filtered_px = pixel_rgba(&filtered_pixels, size.0, 32, 32);
    assert!(
        filtered_px[2] > 200 && filtered_px[0] < 40 && filtered_px[1] < 40 && filtered_px[3] > 200,
        "filtered pixel should be blue after swapping channels"
    );
}
