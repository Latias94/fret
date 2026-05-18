use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_filter_content_dither_is_scissored_and_preserves_outside_content() {
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
    let foreground = SceneOp::Quad {
        order: DrawOrder(2),
        rect: Rect::new(
            Point::new(Px(26.0), Px(52.0)),
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

    let mut direct = Scene::default();
    direct.push(left_red);
    direct.push(right_blue);
    direct.push(foreground);

    let mut filtered = Scene::default();
    filtered.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::Dither {
            mode: fret_core::DitherMode::Bayer4x4,
        }]),
        quality: EffectQuality::Auto,
    });
    filtered.push(left_red);
    filtered.push(right_blue);
    filtered.push(SceneOp::PopEffect);
    filtered.push(foreground);

    let direct_pixels = render_scene_rgba8(&ctx, &mut renderer, &direct, size, 1.0);
    let filtered_pixels = render_scene_rgba8(&ctx, &mut renderer, &filtered, size, 1.0);

    let outside = pixel_rgba(&direct_pixels, size.0, 8, 32);
    let outside_filtered = pixel_rgba(&filtered_pixels, size.0, 8, 32);
    assert_eq!(
        outside, outside_filtered,
        "pixels outside effect bounds should match (unfiltered content preserved)"
    );

    let fg = pixel_rgba(&filtered_pixels, size.0, 32, 58);
    assert!(
        fg[1] > 200 && fg[0] < 40 && fg[2] < 40 && fg[3] > 200,
        "foreground quad drawn after PopEffect should remain on top"
    );
}
