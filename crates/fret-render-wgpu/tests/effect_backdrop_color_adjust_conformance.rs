use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

fn is_grayish(px: [u8; 4]) -> bool {
    let r = px[0] as i32;
    let g = px[1] as i32;
    let b = px[2] as i32;
    (r - g).abs() <= 12 && (g - b).abs() <= 12 && (r - b).abs() <= 12
}

#[test]
fn gpu_backdrop_color_adjust_is_scissored_and_preserves_ordering() {
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

    // Left half red, right half blue: hard edge at x=32.
    base.push(SceneOp::Quad {
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
    });
    base.push(SceneOp::Quad {
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
    });

    let foreground = SceneOp::Quad {
        order: DrawOrder(2),
        rect: Rect::new(
            Point::new(Px(26.0), Px(48.0)),
            Size::new(Px(12.0), Px(12.0)),
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
    };

    let mut without_effect = base.clone();
    without_effect.push(foreground);

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::ColorAdjust {
            saturation: 0.0,
            brightness: 1.0,
            contrast: 1.0,
        }]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(foreground);
    with_effect.push(SceneOp::PopEffect);

    let direct = render_scene_rgba8(&ctx, &mut renderer, &without_effect, size, 1.0);
    let adjusted = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);

    // Outside bounds: unchanged.
    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_adjusted = pixel_rgba(&adjusted, size.0, 8, 32);
    assert_eq!(
        outside, outside_adjusted,
        "pixels outside effect bounds must remain unchanged"
    );

    // Inside bounds: desaturation should make colors grayish.
    let inside_direct = pixel_rgba(&direct, size.0, 28, 32);
    let inside_adjusted = pixel_rgba(&adjusted, size.0, 28, 32);
    assert_ne!(
        inside_direct, inside_adjusted,
        "pixels inside effect bounds should be affected by color adjustment"
    );
    assert!(
        is_grayish(inside_adjusted),
        "desaturated pixel should be roughly gray (r≈g≈b)"
    );

    // Foreground quad must remain on top (PushEffect is a sequence point).
    let fg = pixel_rgba(&adjusted, size.0, 32, 56);
    assert!(
        fg[1] > 200 && fg[0] < 40 && fg[2] < 40 && fg[3] > 200,
        "foreground quad should remain visible on top of the adjusted backdrop"
    );
}

#[test]
fn gpu_backdrop_color_adjust_brightness_is_a_multiplier_with_identity_1() {
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

    // Use non-saturated colors so "brightness as add" would clamp to white, while
    // "brightness as multiply" stays in range.
    base.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.25,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    base.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(32.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.25,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::ColorAdjust {
            saturation: 1.0,
            brightness: 1.02,
            contrast: 1.0,
        }]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(SceneOp::PopEffect);

    let adjusted = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);

    let px = pixel_rgba(&adjusted, size.0, 8, 32);
    assert!(
        px[0] > 40 && px[0] < 240 && px[1] < 40 && px[2] < 40 && px[3] > 240,
        "brightness should not clamp to white; got rgba={:?}",
        px
    );
}
