use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_backdrop_acrylic_recipe_is_scissored_deterministic_and_preserves_ordering() {
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

    // Left half white, right half black: sharp edge at x=32.
    base.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
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
    base.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(32.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    // Foreground marker quad, used to assert ordering around the PushEffect boundary.
    let foreground = SceneOp::Quad {
        order: DrawOrder(2),
        rect: Rect::new(
            Point::new(Px(26.0), Px(48.0)),
            Size::new(Px(12.0), Px(12.0)),
        ),
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

    let mut without_effect = base.clone();
    without_effect.push(foreground);

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[
            EffectStep::GaussianBlur {
                radius_px: Px(4.0),
                downsample: 2,
            },
            EffectStep::ColorAdjust {
                saturation: 1.2,
                brightness: 0.0,
                contrast: 1.0,
            },
            EffectStep::NoiseV1(fret_core::scene::NoiseV1 {
                strength: 0.04,
                scale_px: Px(6.0),
                phase: 0.0,
            }),
        ]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(foreground);
    with_effect.push(SceneOp::PopEffect);

    let direct = render_scene_rgba8(&ctx, &mut renderer, &without_effect, size, 1.0);
    let blurred = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);
    let blurred_again = render_scene_rgba8(&ctx, &mut renderer, &with_effect, size, 1.0);

    // Outside bounds: unchanged.
    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_blurred = pixel_rgba(&blurred, size.0, 8, 32);
    assert_eq!(
        outside, outside_blurred,
        "pixels outside effect bounds must remain unchanged"
    );

    // Inside bounds near an edge: the acrylic chain should affect the pixel.
    let inside = pixel_rgba(&direct, size.0, 32, 32);
    let inside_blurred = pixel_rgba(&blurred, size.0, 32, 32);
    assert_ne!(
        inside, inside_blurred,
        "pixels inside effect bounds near an edge should be affected by the recipe"
    );

    // Deterministic: no hidden time dependency.
    let inside_blurred_again = pixel_rgba(&blurred_again, size.0, 32, 32);
    assert_eq!(
        inside_blurred, inside_blurred_again,
        "recipe must be deterministic for a fixed NoiseV1 phase"
    );

    // Foreground quad must remain on top (PushEffect is a sequence point).
    let fg = pixel_rgba(&blurred, size.0, 32, 56);
    assert!(
        fg[0] > 200 && fg[1] < 40 && fg[2] < 40 && fg[3] > 200,
        "foreground quad should remain visible on top of the backdrop recipe"
    );
}
