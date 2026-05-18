use fret_core::geometry::{Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, DropShadowV1, EffectChain, EffectMode, EffectQuality, EffectStep, Paint,
    Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_drop_shadow_v1_renders_behind_content_and_is_scissored() {
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

    let background = SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    };

    let content = SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(
            Point::new(Px(24.0), Px(24.0)),
            Size::new(Px(16.0), Px(16.0)),
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

    let mut direct = Scene::default();
    direct.push(background);
    direct.push(content);

    let shadow = DropShadowV1 {
        offset_px: Point::new(Px(4.0), Px(4.0)),
        blur_radius_px: Px(4.0),
        downsample: 2,
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.7,
        },
    };

    let mut filtered = Scene::default();
    filtered.push(background);
    filtered.push(SceneOp::PushEffect {
        bounds: Rect::new(
            Point::new(Px(16.0), Px(16.0)),
            Size::new(Px(40.0), Px(40.0)),
        ),
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::DropShadowV1(shadow)]),
        quality: EffectQuality::Auto,
    });
    filtered.push(content);
    filtered.push(SceneOp::PopEffect);

    let direct_pixels = render_scene_rgba8(&ctx, &mut renderer, &direct, size, 1.0);
    let filtered_pixels = render_scene_rgba8(&ctx, &mut renderer, &filtered, size, 1.0);

    let outside_direct = pixel_rgba(&direct_pixels, size.0, 8, 8);
    let outside_filtered = pixel_rgba(&filtered_pixels, size.0, 8, 8);
    assert_eq!(
        outside_direct, outside_filtered,
        "pixels outside effect bounds should match (unfiltered background preserved)"
    );

    let inside_content_direct = pixel_rgba(&direct_pixels, size.0, 30, 30);
    let inside_content_filtered = pixel_rgba(&filtered_pixels, size.0, 30, 30);
    assert_eq!(
        inside_content_direct, inside_content_filtered,
        "shadow should composite under opaque content (no darkening of the content itself)"
    );

    // Probe just outside the content rect, where the offset shadow should be clearly visible.
    let shadow_probe_direct = pixel_rgba(&direct_pixels, size.0, 41, 41);
    let shadow_probe_filtered = pixel_rgba(&filtered_pixels, size.0, 41, 41);
    assert!(
        shadow_probe_filtered[0] < shadow_probe_direct[0]
            && shadow_probe_filtered[1] < shadow_probe_direct[1]
            && shadow_probe_filtered[2] < shadow_probe_direct[2],
        "a pixel in the offset shadow region should darken the background (direct={shadow_probe_direct:?}, filtered={shadow_probe_filtered:?})"
    );
}
