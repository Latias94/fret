use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    BackdropWarpKindV1, BackdropWarpV1, Color, DrawOrder, EffectChain, EffectMode, EffectQuality,
    EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

fn stripe_scene_base(size: (u32, u32)) -> Scene {
    let mut base = Scene::default();
    base.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(size.0 as f32), Px(size.1 as f32)),
        ),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
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
            order: DrawOrder(1 + i),
            rect: Rect::new(
                Point::new(Px(x), Px(0.0)),
                Size::new(Px(1.0), Px(size.1 as f32)),
            ),
            background: (Paint::Solid(bg)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Default::default(),
        });
    }

    base
}

#[test]
fn gpu_backdrop_warp_is_scissored_and_preserves_ordering() {
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
    let base = stripe_scene_base(size);

    let foreground = SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(
            Point::new(Px(26.0), Px(48.0)),
            Size::new(Px(12.0), Px(12.0)),
        ),
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
    };

    let mut without_effect = base.clone();
    without_effect.push(foreground);

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::BackdropWarpV1(BackdropWarpV1 {
            strength_px: Px(12.0),
            scale_px: Px(10.0),
            phase: 0.137,
            chromatic_aberration_px: Px(0.0),
            kind: BackdropWarpKindV1::Wave,
        })]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(foreground);
    with_effect.push(SceneOp::PopEffect);

    let direct = render_scene_rgba8(&ctx, &mut renderer, &without_effect, size, 1.0);
    let warped = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);

    // Outside bounds: unchanged (green).
    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_warped = pixel_rgba(&warped, size.0, 8, 32);
    assert_eq!(
        outside, outside_warped,
        "pixels outside effect bounds must remain unchanged"
    );

    // Inside bounds: at least one of the sampled pixels must change.
    let samples = [(25u32, 10u32), (26u32, 32u32), (37u32, 20u32)];
    let mut changed = 0u32;
    for (x, y) in samples {
        if pixel_rgba(&direct, size.0, x, y) != pixel_rgba(&warped, size.0, x, y) {
            changed += 1;
        }
    }
    assert!(
        changed > 0,
        "expected backdrop-warp to modify at least one sampled pixel inside bounds"
    );

    // Foreground must remain visible on top (sequence point).
    let fg = pixel_rgba(&warped, size.0, 32, 56);
    assert!(
        fg[0] > 200 && fg[1] > 200 && fg[2] > 200 && fg[3] > 200,
        "foreground quad should remain visible on top of the warped backdrop"
    );
}

#[test]
fn gpu_filter_content_warp_is_deterministically_ignored() {
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
    let base = stripe_scene_base(size);
    let quad = SceneOp::Quad {
        order: DrawOrder(200),
        rect: Rect::new(
            Point::new(Px(24.0), Px(16.0)),
            Size::new(Px(16.0), Px(16.0)),
        ),
        background: (Paint::Solid(Color {
            r: 0.9,
            g: 0.9,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    };

    // Baseline: same FilterContent wrapper, but with an empty chain.
    let mut filter_content_noop = base.clone();
    filter_content_noop.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[]),
        quality: EffectQuality::Auto,
    });
    filter_content_noop.push(quad);
    filter_content_noop.push(SceneOp::PopEffect);

    // Warp chain must not affect FilterContent.
    let mut filter_content_warp = base;
    filter_content_warp.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::BackdropWarpV1(BackdropWarpV1 {
            strength_px: Px(12.0),
            scale_px: Px(10.0),
            phase: 0.137,
            chromatic_aberration_px: Px(0.0),
            kind: BackdropWarpKindV1::Wave,
        })]),
        quality: EffectQuality::Auto,
    });
    filter_content_warp.push(quad);
    filter_content_warp.push(SceneOp::PopEffect);

    let noop = render_scene_rgba8(&ctx, &mut renderer, &filter_content_noop, size, 1.0);
    let filtered = render_scene_rgba8(&ctx, &mut renderer, &filter_content_warp, size, 1.0);

    for (x, y) in [(25u32, 20u32), (30u32, 24u32), (39u32, 30u32)] {
        assert_eq!(
            pixel_rgba(&noop, size.0, x, y),
            pixel_rgba(&filtered, size.0, x, y),
            "FilterContent must ignore BackdropWarpV1 deterministically (pixel {x},{y})"
        );
    }
}
