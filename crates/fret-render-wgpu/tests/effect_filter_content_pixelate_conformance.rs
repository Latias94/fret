use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_filter_content_pixelate_is_scissored_and_preserves_outside_content() {
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

    let outside_marker = SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(4.0), Px(0.0)), Size::new(Px(8.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    };

    let push_stripes = |scene: &mut Scene| {
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
            scene.push(SceneOp::Quad {
                order: DrawOrder(10 + i),
                rect: Rect::new(Point::new(Px(x), Px(0.0)), Size::new(Px(1.0), Px(64.0))),
                background: (Paint::Solid(bg)).into(),
                border: Edges::all(Px(0.0)),
                border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
                corner_radii: Default::default(),
            });
        }
    };

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
    without_effect.push(outside_marker);
    push_stripes(&mut without_effect);
    without_effect.push(foreground);

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::Pixelate { scale: 4 }]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(outside_marker);
    push_stripes(&mut with_effect);
    with_effect.push(SceneOp::PopEffect);
    with_effect.push(foreground);

    let direct = render_scene_rgba8(&ctx, &mut renderer, &without_effect, size, 1.0);
    let filtered = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);

    // Outside bounds but inside the effect group: should still be present and unchanged (bounds are not a clip).
    let marker_direct = pixel_rgba(&direct, size.0, 8, 32);
    let marker_filtered = pixel_rgba(&filtered, size.0, 8, 32);
    assert_eq!(
        marker_direct, marker_filtered,
        "content outside bounds must remain visible and unfiltered"
    );

    // Inside bounds: adjacent pixels that used to alternate should collapse to the same value.
    let a_direct = pixel_rgba(&direct, size.0, 25, 32);
    let b_direct = pixel_rgba(&direct, size.0, 26, 32);
    assert_ne!(
        a_direct, b_direct,
        "source stripes should alternate by column"
    );

    let a_filtered = pixel_rgba(&filtered, size.0, 25, 32);
    let b_filtered = pixel_rgba(&filtered, size.0, 26, 32);
    assert_eq!(
        a_filtered, b_filtered,
        "pixelate should make adjacent pixels within a block share the same value"
    );

    // Foreground must remain visible on top (sequence point).
    let fg = pixel_rgba(&filtered, size.0, 32, 56);
    assert!(
        fg[0] > 200 && fg[1] > 200 && fg[2] > 200 && fg[3] > 200,
        "foreground quad drawn after PopEffect should remain visible on top"
    );
}
