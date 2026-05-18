use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_scissored_blur_preserves_outside_region() {
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
    let mut scene = Scene::default();

    // Left half white, right half black: sharp edge at x=32.
    scene.push(SceneOp::Quad {
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
    scene.push(SceneOp::Quad {
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

    let direct = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    renderer.set_debug_blur_radius(2);
    renderer.set_debug_blur_scissor(Some((24, 0, 16, 64)));
    let blurred = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_blurred = pixel_rgba(&blurred, size.0, 8, 32);
    assert_eq!(
        outside, outside_blurred,
        "pixels outside scissor must remain unchanged"
    );

    let inside = pixel_rgba(&direct, size.0, 32, 32);
    let inside_blurred = pixel_rgba(&blurred, size.0, 32, 32);
    assert_ne!(
        inside, inside_blurred,
        "pixels inside scissor near an edge should be affected by blur"
    );
}
