use fret_core::PathService;
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, DashPatternV1, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint,
    PaintBindingV1, PaintEvalSpaceV1, Scene, SceneOp, StrokeStyleV1, TileMode,
};
use fret_core::{
    PathCommand, PathConstraints, PathStyle, StrokeCapV1, StrokeJoinV1, StrokeStyleV2,
};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

fn stops_2(a: Color, b: Color) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, a);
    stops[1] = GradientStop::new(1.0, b);
    (stops, 2)
}

fn u(v: f32, sf: f32) -> u32 {
    (v * sf).round() as u32
}

fn prepare_stroke_path(
    renderer: &mut Renderer,
    commands: &[PathCommand],
    scale_factor: f32,
    dash: Option<DashPatternV1>,
) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::StrokeV2(StrokeStyleV2 {
            width: Px(10.0),
            join: StrokeJoinV1::Round,
            cap: StrokeCapV1::Round,
            miter_limit: 4.0,
            dash,
        }),
        PathConstraints { scale_factor },
    );
    id
}

#[test]
fn stroke_rrect_paint_stroke_s01_varies_along_perimeter() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

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

    for sf in [1.0_f32, 1.5, 2.0] {
        let size = (u(64.0, sf), u(64.0, sf));
        let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
        let gradient = LinearGradient {
            start: Point::new(Px(0.0), Px(0.0)),
            end: Point::new(Px(1.0), Px(0.0)),
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count,
            stops,
        };

        let mut scene = Scene::default();
        scene.push(SceneOp::StrokeRRect {
            order: DrawOrder(0),
            rect,
            stroke: Edges::all(Px(8.0)),
            stroke_paint: PaintBindingV1::with_eval_space(
                Paint::LinearGradient(gradient),
                PaintEvalSpaceV1::StrokeS01,
            ),
            corner_radii: Corners::all(Px(8.0)),
            style: StrokeStyleV1 { dash: None },
        });

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);

        // Sample two points that are far apart along the perimeter parameterization:
        // - top edge (near start of s)
        // - left edge (near end of s)
        let top = pixel_rgba(&pixels, size.0, u(32.0, sf), u(4.0, sf));
        let left = pixel_rgba(&pixels, size.0, u(4.0, sf), u(32.0, sf));

        assert!(
            top[3] > 200 && left[3] > 200,
            "expected visible alpha (sf={sf}): top={top:?} left={left:?}"
        );
        assert!(
            left[0] > top[0],
            "expected left edge to be brighter than top edge (sf={sf}): top={top:?} left={left:?}"
        );
    }
}

#[test]
fn path_stroke_paint_stroke_s01_is_monotonic_along_path() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let commands = [
        PathCommand::MoveTo(Point::new(Px(4.0), Px(4.0))),
        PathCommand::LineTo(Point::new(Px(60.0), Px(4.0))),
        PathCommand::LineTo(Point::new(Px(60.0), Px(60.0))),
    ];

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

    for sf in [1.0_f32, 1.5, 2.0] {
        let size = (u(64.0, sf), u(64.0, sf));
        let path = prepare_stroke_path(&mut renderer, &commands, sf, None);
        let gradient = LinearGradient {
            start: Point::new(Px(0.0), Px(0.0)),
            end: Point::new(Px(1.0), Px(0.0)),
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count,
            stops,
        };

        let mut scene = Scene::default();
        scene.push(SceneOp::Path {
            order: DrawOrder(0),
            origin: Point::new(Px(0.0), Px(0.0)),
            path,
            paint: PaintBindingV1::with_eval_space(
                Paint::LinearGradient(gradient),
                PaintEvalSpaceV1::StrokeS01,
            ),
        });

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);

        let start = pixel_rgba(&pixels, size.0, u(8.0, sf), u(4.0, sf));
        let mid = pixel_rgba(&pixels, size.0, u(60.0, sf), u(8.0, sf));
        let end = pixel_rgba(&pixels, size.0, u(60.0, sf), u(56.0, sf));

        assert!(
            start[3] > 100 && mid[3] > 100 && end[3] > 100,
            "expected visible alpha (sf={sf}): start={start:?} mid={mid:?} end={end:?}"
        );

        assert!(
            start[0] <= mid[0] && mid[0] <= end[0],
            "expected monotonic red along s01 (sf={sf}): start={start:?} mid={mid:?} end={end:?}"
        );
    }
}

#[test]
fn path_stroke_paint_stroke_s01_is_continuous_across_dashes() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let commands = [
        PathCommand::MoveTo(Point::new(Px(4.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(60.0), Px(32.0))),
    ];

    let (stops, stop_count) = stops_2(
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
    );

    for sf in [1.0_f32, 1.5, 2.0] {
        let size = (u(64.0, sf), u(64.0, sf));
        let path = prepare_stroke_path(
            &mut renderer,
            &commands,
            sf,
            Some(DashPatternV1::new(Px(8.0), Px(4.0), Px(0.0))),
        );
        let gradient = LinearGradient {
            start: Point::new(Px(0.0), Px(0.0)),
            end: Point::new(Px(1.0), Px(0.0)),
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count,
            stops,
        };

        let mut scene = Scene::default();
        scene.push(SceneOp::Path {
            order: DrawOrder(0),
            origin: Point::new(Px(0.0), Px(0.0)),
            path,
            paint: PaintBindingV1::with_eval_space(
                Paint::LinearGradient(gradient),
                PaintEvalSpaceV1::StrokeS01,
            ),
        });

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);

        // Sample two points that are known to lie on different dash segments (dash=8, gap=4).
        // If StrokeS01 is degraded to LocalPx, both will clamp to the end color and be equal.
        let a = pixel_rgba(&pixels, size.0, u(8.0, sf), u(32.0, sf));
        let b = pixel_rgba(&pixels, size.0, u(20.0, sf), u(32.0, sf));

        assert!(
            a[3] > 100 && b[3] > 100,
            "expected visible alpha (sf={sf}): a={a:?} b={b:?}"
        );
        assert!(
            b[0] > a[0],
            "expected red to increase along StrokeS01 across dashes (sf={sf}): a={a:?} b={b:?}"
        );
    }
}
