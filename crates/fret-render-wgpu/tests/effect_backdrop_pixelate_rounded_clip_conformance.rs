use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

fn push_bounds_stripes(scene: &mut Scene, bounds: Rect, order_base: u32) {
    for i in 0..bounds.size.width.0 as u32 {
        let x = bounds.origin.x.0 + i as f32;
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
        scene.push(SceneOp::Quad {
            order: DrawOrder(order_base + i),
            rect: Rect::new(
                Point::new(Px(x), bounds.origin.y),
                Size::new(Px(1.0), bounds.size.height),
            ),
            background: (Paint::Solid(bg)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Default::default(),
        });
    }
}

#[test]
fn gpu_backdrop_pixelate_respects_rounded_clip_stack_on_writeback() {
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
    let bounds = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let mut without_effect = Scene::default();
    without_effect.push(SceneOp::Quad {
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
    push_bounds_stripes(&mut without_effect, bounds, 1);
    without_effect.push(SceneOp::PushClipRRect {
        rect: bounds,
        corner_radii: Corners::all(Px(14.0)),
    });
    // Foreground marker: should remain visible regardless of effect order.
    without_effect.push(SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(Point::new(Px(28.0), Px(36.0)), Size::new(Px(8.0), Px(8.0))),
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
    without_effect.push(SceneOp::PopClip);

    let mut with_effect = Scene::default();
    with_effect.push(SceneOp::Quad {
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
    push_bounds_stripes(&mut with_effect, bounds, 1);
    with_effect.push(SceneOp::PushClipRRect {
        rect: bounds,
        corner_radii: Corners::all(Px(14.0)),
    });
    with_effect.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::Pixelate { scale: 4 }]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(Point::new(Px(28.0), Px(36.0)), Size::new(Px(8.0), Px(8.0))),
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
    with_effect.push(SceneOp::PopClip);

    let direct = render_scene_rgba8(&ctx, &mut renderer, &without_effect, size, 1.0);
    let pixelated = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);

    // Outside effect bounds: unchanged.
    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_pixelated = pixel_rgba(&pixelated, size.0, 8, 32);
    assert_eq!(
        outside, outside_pixelated,
        "pixels outside effect bounds must remain unchanged"
    );

    // Inside bounds but outside the rounded clip: unchanged (no leakage into corners).
    let corner_outside_clip = pixel_rgba(&direct, size.0, 17, 17);
    let corner_outside_clip_pixelated = pixel_rgba(&pixelated, size.0, 17, 17);
    assert_eq!(
        corner_outside_clip, corner_outside_clip_pixelated,
        "pixels outside the rounded clip (but inside effect bounds) must remain unchanged"
    );

    // Inside bounds near stripes: pixelation should affect at least some pixels.
    let mut any_changed = false;
    for x in 20u32..44u32 {
        let inside = pixel_rgba(&direct, size.0, x, 32);
        let inside_pixelated = pixel_rgba(&pixelated, size.0, x, 32);
        if inside != inside_pixelated {
            any_changed = true;
            break;
        }
    }
    assert!(
        any_changed,
        "expected pixelate to affect at least one pixel inside the rounded clip"
    );

    // Foreground marker must remain on top (PushEffect is a sequence point).
    let fg = pixel_rgba(&pixelated, size.0, 32, 40);
    assert!(
        fg[0] > 200 && fg[1] > 200 && fg[2] > 200 && fg[3] > 200,
        "foreground quad should remain visible on top of the backdrop pixelate"
    );
}
