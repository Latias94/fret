use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    BlendMode, Color, CompositeGroupDesc, DrawOrder, EffectQuality, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{Renderer, WgpuContext};

#[path = "support/render_format.rs"]
mod support;

use support::{pixel_rgba, render_scene_rgba8_with_format};

fn render_composite_scene_rgba8(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
    scene: &Scene,
    size: (u32, u32),
) -> Vec<u8> {
    render_scene_rgba8_with_format(
        ctx,
        renderer,
        scene,
        size,
        1.0,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    )
}

fn u8_from_f32_clamped(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

fn linear_to_srgb_f32(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0031308 {
        x * 12.92
    } else {
        let a = 0.055;
        (1.0 + a) * x.powf(1.0 / 2.4) - a
    }
}

fn u8_from_linear_to_srgb_f32(x: f32) -> u8 {
    u8_from_f32_clamped(linear_to_srgb_f32(x))
}

fn assert_rgba_approx_eq(actual: [u8; 4], expected: [u8; 4], tol: u8, context: &str) {
    for i in 0..4 {
        let a = actual[i];
        let e = expected[i];
        let lo = e.saturating_sub(tol);
        let hi = e.saturating_add(tol);
        assert!(
            a >= lo && a <= hi,
            "{context}: channel[{i}] expected≈{expected:?} (tol={tol}) got={actual:?}"
        );
    }
}

#[test]
fn gpu_composite_group_add_is_scissored_and_additive() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let full = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let bounds = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: full,
        background: (Paint::Solid(Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    scene.push(SceneOp::PushCompositeGroup {
        desc: CompositeGroupDesc::new(bounds, BlendMode::Add, EffectQuality::Auto),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: bounds,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.5,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopCompositeGroup);

    let pixels = render_composite_scene_rgba8(&ctx, &mut renderer, &scene, size);

    let outside = pixel_rgba(&pixels, size.0, 8, 8);
    let just_outside = pixel_rgba(&pixels, size.0, 15, 32);
    let inside = pixel_rgba(&pixels, size.0, 32, 32);

    assert!(
        outside[3] > 240 && just_outside[3] > 240 && inside[3] > 240,
        "expected opaque alpha: outside={outside:?} just_outside={just_outside:?} inside={inside:?}"
    );
    assert!(
        outside == just_outside,
        "expected scissor to preserve outside pixels: outside={outside:?} just_outside={just_outside:?} inside={inside:?}"
    );

    assert!(
        inside[0] > outside[0] + 6 && inside[1] > outside[1] + 6 && inside[2] > outside[2] + 6,
        "expected additive blend to brighten inside pixels: outside={outside:?} inside={inside:?}"
    );
}

#[test]
fn gpu_composite_group_blend_modes_v2_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let full = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let bounds = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let dst = Color {
        r: 0.6,
        g: 0.2,
        b: 0.8,
        a: 1.0,
    };
    let src = Color {
        r: 0.1,
        g: 0.5,
        b: 0.3,
        a: 1.0,
    };

    let modes = [BlendMode::Darken, BlendMode::Lighten, BlendMode::Subtract];

    for mode in modes {
        let mut scene = Scene::default();
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: full,
            background: (Paint::Solid(dst)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::TRANSPARENT).into(),
            corner_radii: Corners::all(Px(0.0)),
        });

        scene.push(SceneOp::PushCompositeGroup {
            desc: CompositeGroupDesc::new(bounds, mode, EffectQuality::Auto),
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: bounds,
            background: (Paint::Solid(src)).into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::TRANSPARENT).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopCompositeGroup);

        let pixels = render_composite_scene_rgba8(&ctx, &mut renderer, &scene, size);

        let outside = pixel_rgba(&pixels, size.0, 8, 8);
        let inside = pixel_rgba(&pixels, size.0, 32, 32);

        assert_rgba_approx_eq(
            outside,
            [
                u8_from_linear_to_srgb_f32(dst.r),
                u8_from_linear_to_srgb_f32(dst.g),
                u8_from_linear_to_srgb_f32(dst.b),
                255,
            ],
            4,
            &format!("mode={mode:?} outside"),
        );

        let expected_rgb_srgb = match mode {
            BlendMode::Darken => [
                linear_to_srgb_f32(dst.r.min(src.r)),
                linear_to_srgb_f32(dst.g.min(src.g)),
                linear_to_srgb_f32(dst.b.min(src.b)),
            ],
            BlendMode::Lighten => [
                linear_to_srgb_f32(dst.r.max(src.r)),
                linear_to_srgb_f32(dst.g.max(src.g)),
                linear_to_srgb_f32(dst.b.max(src.b)),
            ],
            BlendMode::Subtract => [
                linear_to_srgb_f32((dst.r - src.r).clamp(0.0, 1.0)),
                linear_to_srgb_f32((dst.g - src.g).clamp(0.0, 1.0)),
                linear_to_srgb_f32((dst.b - src.b).clamp(0.0, 1.0)),
            ],
            BlendMode::Over | BlendMode::Add | BlendMode::Multiply | BlendMode::Screen => {
                unreachable!("modes loop must include only v2 fixed-function modes")
            }
        };

        assert_rgba_approx_eq(
            inside,
            [
                u8_from_f32_clamped(expected_rgb_srgb[0]),
                u8_from_f32_clamped(expected_rgb_srgb[1]),
                u8_from_f32_clamped(expected_rgb_srgb[2]),
                255,
            ],
            6,
            &format!("mode={mode:?} inside"),
        );
    }
}

#[test]
fn gpu_composite_group_opacity_is_isolated_for_overlapping_children() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let full = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let a = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );
    let b = Rect::new(
        Point::new(Px(24.0), Px(24.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let paint = Paint::Solid(Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.5,
    })
    .into();

    let mut stack_opacity_scene = Scene::default();
    stack_opacity_scene.push(SceneOp::PushOpacity { opacity: 0.5 });
    stack_opacity_scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: a,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    stack_opacity_scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: b,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    stack_opacity_scene.push(SceneOp::PopOpacity);

    let mut isolated_opacity_scene = Scene::default();
    isolated_opacity_scene.push(SceneOp::PushCompositeGroup {
        desc: CompositeGroupDesc::new(full, BlendMode::Over, EffectQuality::Auto).with_opacity(0.5),
    });
    isolated_opacity_scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: a,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    isolated_opacity_scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: b,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    isolated_opacity_scene.push(SceneOp::PopCompositeGroup);

    let stack_pixels =
        render_composite_scene_rgba8(&ctx, &mut renderer, &stack_opacity_scene, size);
    let isolated_pixels =
        render_composite_scene_rgba8(&ctx, &mut renderer, &isolated_opacity_scene, size);

    let stack_single = pixel_rgba(&stack_pixels, size.0, 18, 18);
    let isolated_single = pixel_rgba(&isolated_pixels, size.0, 18, 18);
    let stack_overlap = pixel_rgba(&stack_pixels, size.0, 32, 32);
    let isolated_overlap = pixel_rgba(&isolated_pixels, size.0, 32, 32);

    // In a non-overlapping region, isolated opacity should match multiplicative opacity.
    for c in 0..4 {
        let a = stack_single[c] as i16;
        let b = isolated_single[c] as i16;
        assert!(
            (a - b).abs() <= 3,
            "expected single-quad pixels to match: stack={stack_single:?} isolated={isolated_single:?}"
        );
    }

    // In an overlapping region, isolated opacity differs from multiplicative opacity (the group
    // alpha is applied after internal blending).
    assert!(
        stack_overlap[3] >= isolated_overlap[3].saturating_add(8),
        "expected isolated overlap alpha to be lower: stack={stack_overlap:?} isolated={isolated_overlap:?}"
    );
}

#[test]
fn gpu_composite_group_opacity_degrades_under_tight_intermediate_budget() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(1024);

    let size = (64u32, 64u32);
    let full = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let a = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );
    let b = Rect::new(
        Point::new(Px(24.0), Px(24.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let paint = Paint::Solid(Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.5,
    })
    .into();

    let mut baseline = Scene::default();
    baseline.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: a,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    baseline.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: b,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let mut degraded = Scene::default();
    degraded.push(SceneOp::PushCompositeGroup {
        desc: CompositeGroupDesc::new(full, BlendMode::Over, EffectQuality::Auto).with_opacity(0.5),
    });
    degraded.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: a,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    degraded.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: b,
        background: paint,
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    degraded.push(SceneOp::PopCompositeGroup);

    let baseline_pixels = render_composite_scene_rgba8(&ctx, &mut renderer, &baseline, size);
    let degraded_pixels = render_composite_scene_rgba8(&ctx, &mut renderer, &degraded, size);

    let baseline_overlap = pixel_rgba(&baseline_pixels, size.0, 32, 32);
    let degraded_overlap = pixel_rgba(&degraded_pixels, size.0, 32, 32);

    for c in 0..4 {
        let a = baseline_overlap[c] as i16;
        let b = degraded_overlap[c] as i16;
        assert!(
            (a - b).abs() <= 3,
            "expected deterministic degradation to match baseline draws: baseline={baseline_overlap:?} degraded={degraded_overlap:?}"
        );
    }
}
