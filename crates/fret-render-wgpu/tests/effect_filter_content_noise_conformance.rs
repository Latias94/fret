use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_filter_content_noise_is_scissored_and_deterministic() {
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

    let left_red = SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
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
    let right_blue = SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(32.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    };

    let mut direct = Scene::default();
    direct.push(left_red);
    direct.push(right_blue);

    let mut filtered = Scene::default();
    filtered.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::NoiseV1(fret_core::scene::NoiseV1 {
            strength: 0.2,
            scale_px: Px(1.0),
            phase: 0.0,
        })]),
        quality: EffectQuality::Auto,
    });
    filtered.push(left_red);
    filtered.push(right_blue);
    filtered.push(SceneOp::PopEffect);

    let direct_pixels = render_scene_rgba8(&ctx, &mut renderer, &direct, size, 1.0);
    let filtered_pixels_a = render_scene_rgba8(&ctx, &mut renderer, &filtered, size, 1.0);
    let filtered_pixels_b = render_scene_rgba8(&ctx, &mut renderer, &filtered, size, 1.0);

    assert_eq!(
        filtered_pixels_a, filtered_pixels_b,
        "procedural noise must be deterministic for identical inputs"
    );

    let outside = pixel_rgba(&direct_pixels, size.0, 8, 32);
    let outside_filtered = pixel_rgba(&filtered_pixels_a, size.0, 8, 32);
    assert_eq!(
        outside, outside_filtered,
        "pixels outside effect bounds should match (unfiltered content preserved)"
    );

    let mut any_inside_diff = false;
    for (x, y) in [(28, 8), (30, 32), (35, 12), (38, 44), (39, 60)] {
        let inside_direct = pixel_rgba(&direct_pixels, size.0, x, y);
        let inside_filtered = pixel_rgba(&filtered_pixels_a, size.0, x, y);
        if inside_direct != inside_filtered {
            any_inside_diff = true;
            break;
        }
    }
    assert!(
        any_inside_diff,
        "at least one pixel inside effect bounds should differ (noise applied)"
    );
}
