mod support;

use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint, RadialGradient,
    Scene, SceneOp, SweepGradient, TileMode,
};
use fret_render_wgpu::{Renderer, WgpuContext};
use support::{pixel_rgba, render_scene_rgba8};

fn stops_2(a: Color, b: Color) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, a);
    stops[1] = GradientStop::new(1.0, b);
    (stops, 2)
}

fn stops_3(a: Color, b: Color, c: Color) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, a);
    stops[1] = GradientStop::new(0.5, b);
    stops[2] = GradientStop::new(1.0, c);
    (stops, 3)
}

#[test]
fn gpu_linear_gradient_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_2(
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    );

    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(64.0), Px(0.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::LinearGradient(gradient)).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let left = pixel_rgba(&pixels, size.0, 4, 32);
    let mid = pixel_rgba(&pixels, size.0, 32, 32);
    let right = pixel_rgba(&pixels, size.0, 59, 32);

    assert!(
        left[3] > 240 && mid[3] > 240 && right[3] > 240,
        "expected opaque alpha: left={left:?} mid={mid:?} right={right:?}"
    );

    assert!(
        left[0] <= mid[0] && mid[0] <= right[0],
        "red must be non-decreasing: left={left:?} mid={mid:?} right={right:?}"
    );
    assert!(
        left[1] <= mid[1] && mid[1] <= right[1],
        "green must be non-decreasing: left={left:?} mid={mid:?} right={right:?}"
    );
    assert!(
        left[2] <= mid[2] && mid[2] <= right[2],
        "blue must be non-decreasing: left={left:?} mid={mid:?} right={right:?}"
    );

    let dr = right[0].saturating_sub(left[0]);
    let dg = right[1].saturating_sub(left[1]);
    let db = right[2].saturating_sub(left[2]);
    assert!(
        dr >= 8 && dg >= 8 && db >= 8,
        "expected visible gradient range: left={left:?} mid={mid:?} right={right:?}"
    );
}

#[test]
fn gpu_linear_gradient_repeat_tile_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_3(
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    );

    // Use a short gradient span so `t` crosses > 1.0 across the quad.
    // Repeat tiling should bring the `t=0.5` midpoint color back at `t=1.5`.
    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(16.0), Px(0.0)),
        tile_mode: TileMode::Repeat,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::LinearGradient(gradient)).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let a = pixel_rgba(&pixels, size.0, 8, 32); // t ~= 0.5
    let b = pixel_rgba(&pixels, size.0, 24, 32); // t ~= 1.5 (repeat -> 0.5)

    assert!(
        a[3] > 240 && b[3] > 240,
        "expected opaque alpha: a={a:?} b={b:?}"
    );
    assert!(
        a[0] > 160 && a[1] < 80 && a[2] < 80,
        "expected red-ish midpoint under repeat tiling: a={a:?}"
    );
    assert!(
        b[0] > 160 && b[1] < 80 && b[2] < 80,
        "expected repeated red-ish midpoint at t=1.5: b={b:?}"
    );
}

#[test]
fn gpu_linear_gradient_mirror_tile_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_2(
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    );

    let gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(16.0), Px(0.0)),
        tile_mode: TileMode::Mirror,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::LinearGradient(gradient)).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let dark = pixel_rgba(&pixels, size.0, 4, 32); // t ~= 0.25
    let bright = pixel_rgba(&pixels, size.0, 20, 32); // t ~= 1.25 (mirror -> 0.75)

    assert!(
        dark[3] > 240 && bright[3] > 240,
        "expected opaque alpha: dark={dark:?} bright={bright:?}"
    );
    assert!(
        bright[0] > dark[0].saturating_add(40),
        "expected mirror tiling to increase brightness at t=1.25 vs t=0.25: dark={dark:?} bright={bright:?}"
    );
}

#[test]
fn gpu_linear_gradient_oklab_color_space_midpoint_differs_from_srgb() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (96u32, 64u32);
    let left = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(48.0), Px(64.0)));
    let right = Rect::new(Point::new(Px(48.0), Px(0.0)), Size::new(Px(48.0), Px(64.0)));

    let (stops, stop_count) = stops_2(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
    );

    let srgb_gradient = LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(48.0), Px(0.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };
    let oklab_gradient = LinearGradient {
        start: Point::new(Px(48.0), Px(0.0)),
        end: Point::new(Px(96.0), Px(0.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Oklab,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: left,
        background: (Paint::LinearGradient(srgb_gradient)).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: right,
        background: (Paint::LinearGradient(oklab_gradient)).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let srgb_mid = pixel_rgba(&pixels, size.0, 24, 32);
    let oklab_mid = pixel_rgba(&pixels, size.0, 72, 32);

    assert!(
        srgb_mid[3] > 240 && oklab_mid[3] > 240,
        "expected opaque alpha: srgb_mid={srgb_mid:?} oklab_mid={oklab_mid:?}"
    );

    let dr = u16::from(srgb_mid[0].abs_diff(oklab_mid[0]));
    let dg = u16::from(srgb_mid[1].abs_diff(oklab_mid[1]));
    let db = u16::from(srgb_mid[2].abs_diff(oklab_mid[2]));
    let diff = dr + dg + db;
    assert!(
        diff >= 24,
        "expected Oklab interpolation to differ visibly from sRGB/linear interpolation at the midpoint: srgb_mid={srgb_mid:?} oklab_mid={oklab_mid:?} diff={diff}"
    );
}

#[test]
fn gpu_radial_gradient_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_2(
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    );

    let gradient = RadialGradient {
        center: Point::new(Px(32.0), Px(32.0)),
        radius: Size::new(Px(32.0), Px(32.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::RadialGradient(gradient)).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let center = pixel_rgba(&pixels, size.0, 32, 32);
    let corner = pixel_rgba(&pixels, size.0, 2, 2);

    assert!(center[3] > 240 && corner[3] > 240);
    assert!(corner[0] < center[0] && corner[1] < center[1] && corner[2] < center[2]);
}

#[test]
fn gpu_sweep_gradient_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let (stops, stop_count) = stops_3(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
    );

    let gradient = SweepGradient {
        center: Point::new(Px(32.0), Px(32.0)),
        start_angle_turns: 0.0,
        end_angle_turns: 1.0,
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::SweepGradient(gradient)).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let right = pixel_rgba(&pixels, size.0, 59, 32);
    let left = pixel_rgba(&pixels, size.0, 4, 32);
    let down = pixel_rgba(&pixels, size.0, 32, 59);
    let up = pixel_rgba(&pixels, size.0, 32, 4);

    assert!(
        right[3] > 240 && left[3] > 240 && down[3] > 240 && up[3] > 240,
        "expected opaque alpha: right={right:?} left={left:?} down={down:?} up={up:?}"
    );

    assert!(
        right[0] > right[1] && right[0] > right[2],
        "expected red-dominant at +X (right): {right:?}"
    );
    assert!(
        left[1] > left[0] && left[1] > left[2],
        "expected green-dominant at -X (left): {left:?}"
    );
    assert!(
        down[0] > down[2] && down[1] > down[2],
        "expected red/green mix at +Y (down): {down:?}"
    );
    assert!(
        up[2] > up[0] && up[1] > up[0],
        "expected green/blue mix at -Y (up): {up:?}"
    );
}
