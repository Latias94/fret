use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_alpha_threshold_hard_and_soft() {
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

    let left = SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.25,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    };
    let right = SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(32.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.75,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    };

    let mut hard = Scene::default();
    hard.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::AlphaThreshold {
            cutoff: 0.5,
            soft: 0.0,
        }]),
        quality: EffectQuality::Auto,
    });
    hard.push(left);
    hard.push(right);
    hard.push(SceneOp::PopEffect);

    let hard_pixels = render_scene_rgba8(&ctx, &mut renderer, &hard, size, 1.0);
    let left_px = pixel_rgba(&hard_pixels, size.0, 16, 32);
    let right_px = pixel_rgba(&hard_pixels, size.0, 48, 32);

    assert!(
        left_px[3] < 10,
        "left half should be thresholded out (alpha)"
    );
    assert!(
        right_px[0] > 150 && right_px[3] > 150,
        "right half should survive the hard threshold"
    );

    let mid = SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.50,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    };

    let mut soft = Scene::default();
    soft.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::AlphaThreshold {
            cutoff: 0.5,
            soft: 0.1,
        }]),
        quality: EffectQuality::Auto,
    });
    soft.push(mid);
    soft.push(SceneOp::PopEffect);

    let soft_pixels = render_scene_rgba8(&ctx, &mut renderer, &soft, size, 1.0);
    let px = pixel_rgba(&soft_pixels, size.0, 32, 32);
    assert!(
        px[3] >= 40 && px[3] <= 90,
        "soft threshold at the midpoint should reduce coverage (alpha={})",
        px[3]
    );
}
