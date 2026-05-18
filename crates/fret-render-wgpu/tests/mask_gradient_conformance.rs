mod support;

use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Mask, Paint,
    RadialGradient, Scene, SceneOp, TileMode,
};
use fret_render_wgpu::{Renderer, WgpuContext};
use support::{pixel_rgba, render_scene_rgba8};

fn stops_2_alpha(a: f32, b: f32) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(
        0.0,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a,
        },
    );
    stops[1] = GradientStop::new(
        1.0,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: b,
        },
    );
    (stops, 2)
}

fn stops_3_alpha(a: f32, b: f32, c: f32) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(
        0.0,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a,
        },
    );
    stops[1] = GradientStop::new(
        0.5,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: b,
        },
    );
    stops[2] = GradientStop::new(
        1.0,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: c,
        },
    );
    (stops, 3)
}

#[test]
fn gpu_linear_gradient_mask_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_2_alpha(0.0, 1.0);

    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(64.0), Px(0.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::PushMask {
        bounds: rect,
        mask: Mask::linear_gradient(gradient),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopMask);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let left = pixel_rgba(&pixels, size.0, 4, 32);
    let mid = pixel_rgba(&pixels, size.0, 32, 32);
    let right = pixel_rgba(&pixels, size.0, 59, 32);

    assert!(
        left[3] <= 32,
        "expected near-transparent alpha at left: left={left:?} mid={mid:?} right={right:?}"
    );
    assert!(
        mid[3] > 64 && mid[3] < 240,
        "expected intermediate alpha at mid: left={left:?} mid={mid:?} right={right:?}"
    );
    assert!(
        right[3] >= 224,
        "expected near-opaque alpha at right: left={left:?} mid={mid:?} right={right:?}"
    );
}

#[test]
fn gpu_linear_gradient_mask_repeat_tile_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_3_alpha(0.0, 1.0, 0.0);

    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(16.0), Px(0.0)),
        tile_mode: TileMode::Repeat,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::PushMask {
        bounds: rect,
        mask: Mask::linear_gradient(gradient),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopMask);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let a = pixel_rgba(&pixels, size.0, 8, 32); // t ~= 0.5
    let b = pixel_rgba(&pixels, size.0, 24, 32); // t ~= 1.5 (repeat -> 0.5)

    assert!(
        a[3] >= 224 && b[3] >= 224,
        "expected high alpha at repeated midpoint: a={a:?} b={b:?}"
    );
}

#[test]
fn gpu_linear_gradient_mask_mirror_tile_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_2_alpha(0.0, 1.0);

    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(16.0), Px(0.0)),
        tile_mode: TileMode::Mirror,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::PushMask {
        bounds: rect,
        mask: Mask::linear_gradient(gradient),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopMask);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let dark = pixel_rgba(&pixels, size.0, 4, 32); // t ~= 0.25
    let bright = pixel_rgba(&pixels, size.0, 20, 32); // t ~= 1.25 (mirror -> 0.75)

    assert!(
        bright[3] > dark[3].saturating_add(40),
        "expected mirror tiling to increase alpha at t=1.25 vs t=0.25: dark={dark:?} bright={bright:?}"
    );
}

#[test]
fn gpu_radial_gradient_mask_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_2_alpha(1.0, 0.0);

    let gradient = RadialGradient {
        center: Point::new(Px(32.0), Px(32.0)),
        radius: Size::new(Px(32.0), Px(32.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::PushMask {
        bounds: rect,
        mask: Mask::radial_gradient(gradient),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopMask);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let center = pixel_rgba(&pixels, size.0, 32, 32);
    let corner = pixel_rgba(&pixels, size.0, 2, 2);

    assert!(
        center[3] > 240,
        "expected near-opaque alpha at center: center={center:?} corner={corner:?}"
    );
    assert!(
        corner[3] < 32,
        "expected near-transparent alpha at corner: center={center:?} corner={corner:?}"
    );
}
