use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_backdrop_pixelate_is_scissored_and_preserves_ordering() {
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
    let mut base = Scene::default();

    // Green background for outside-region invariance.
    base.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
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

    // Vertical 1px stripes inside x=[24, 40): alternating red/blue.
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
            rect: Rect::new(Point::new(Px(x), Px(0.0)), Size::new(Px(1.0), Px(64.0))),
            background: (Paint::Solid(bg)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Default::default(),
        });
    }

    // Foreground marker quad.
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
        chain: EffectChain::from_steps(&[EffectStep::Pixelate { scale: 4 }]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(foreground);
    with_effect.push(SceneOp::PopEffect);

    let direct = render_scene_rgba8(&ctx, &mut renderer, &without_effect, size, 1.0);
    let pixelated = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);

    // Outside bounds: unchanged (green).
    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_pixelated = pixel_rgba(&pixelated, size.0, 8, 32);
    assert_eq!(
        outside, outside_pixelated,
        "pixels outside effect bounds must remain unchanged"
    );

    // Inside bounds: adjacent pixels that used to alternate should collapse to the same value.
    let a_direct = pixel_rgba(&direct, size.0, 25, 32);
    let b_direct = pixel_rgba(&direct, size.0, 26, 32);
    assert_ne!(
        a_direct, b_direct,
        "source stripes should alternate by column"
    );

    let a_pix = pixel_rgba(&pixelated, size.0, 25, 32);
    let b_pix = pixel_rgba(&pixelated, size.0, 26, 32);
    assert_eq!(
        a_pix, b_pix,
        "pixelate should make adjacent pixels within a block share the same value"
    );

    // Foreground must remain visible on top (sequence point).
    let fg = pixel_rgba(&pixelated, size.0, 32, 56);
    assert!(
        fg[0] > 200 && fg[1] > 200 && fg[2] > 200 && fg[3] > 200,
        "foreground quad should remain visible on top of the pixelated backdrop"
    );
}

#[test]
fn gpu_backdrop_pixelate_is_anchored_to_effect_bounds() {
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
    let mut base = Scene::default();

    // Green background for detecting bleed from outside the effect bounds.
    base.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
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

    // Vertical 1px stripes inside x=[25, 41): repeating pattern [R, B, B].
    // This intentionally starts at a non-multiple of the pixelate scale.
    for i in 0..16u32 {
        let x = 25.0 + i as f32;
        let bg = match i % 3 {
            0 => Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            _ => Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
        };
        base.push(SceneOp::Quad {
            order: DrawOrder(1 + i),
            rect: Rect::new(Point::new(Px(x), Px(0.0)), Size::new(Px(1.0), Px(64.0))),
            background: (Paint::Solid(bg)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Default::default(),
        });
    }

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(25.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::Pixelate { scale: 4 }]),
        quality: EffectQuality::Auto,
    });
    // Foreground marker quad (also ensures the effect scope has content).
    with_effect.push(SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(
            Point::new(Px(28.0), Px(48.0)),
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
    });
    with_effect.push(SceneOp::PopEffect);

    let pixelated = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);

    let outside = pixel_rgba(&pixelated, size.0, 8, 32);
    let a = pixel_rgba(&pixelated, size.0, 25, 32);
    let b = pixel_rgba(&pixelated, size.0, 28, 32);
    let next_block = pixel_rgba(&pixelated, size.0, 29, 32);

    assert_ne!(
        outside, a,
        "pixelate must not sample from outside the effect bounds when bounds are not scale-aligned"
    );
    assert_eq!(
        a, b,
        "pixels within a pixelate block should share the same sampled value"
    );
    assert_ne!(
        a, next_block,
        "pixelate blocks should be anchored to the effect bounds, not the window origin"
    );
}
