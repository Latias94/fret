use fret_core::PathService;
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint, Scene, SceneOp,
    StrokeStyleV1, TileMode,
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
) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::StrokeV2(StrokeStyleV2 {
            width: Px(10.0),
            join: StrokeJoinV1::Round,
            cap: StrokeCapV1::Round,
            miter_limit: 4.0,
            dash: None,
        }),
        PathConstraints { scale_factor },
    );
    id
}

#[test]
fn gpu_stroke_rrect_linear_gradient_smoke_conformance_across_scale_factors() {
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
            end: Point::new(Px(64.0), Px(0.0)),
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
            stroke_paint: Paint::LinearGradient(gradient).into(),
            corner_radii: Corners::all(Px(8.0)),
            style: StrokeStyleV1 { dash: None },
        });

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);
        let left = pixel_rgba(&pixels, size.0, u(4.0, sf), u(4.0, sf));
        let mid = pixel_rgba(&pixels, size.0, u(32.0, sf), u(4.0, sf));
        let right = pixel_rgba(&pixels, size.0, u(59.0, sf), u(4.0, sf));

        assert!(
            left[3] > 240 && mid[3] > 240 && right[3] > 240,
            "expected opaque alpha on stroke (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );

        assert!(
            left[0] <= mid[0] && mid[0] <= right[0],
            "red must be non-decreasing across stroke (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
        assert!(
            left[1] <= mid[1] && mid[1] <= right[1],
            "green must be non-decreasing across stroke (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
        assert!(
            left[2] <= mid[2] && mid[2] <= right[2],
            "blue must be non-decreasing across stroke (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
    }
}

#[test]
fn gpu_path_stroke_linear_gradient_smoke_conformance_across_scale_factors() {
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
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    );

    for sf in [1.0_f32, 1.5, 2.0] {
        let size = (u(64.0, sf), u(64.0, sf));
        let path = prepare_stroke_path(&mut renderer, &commands, sf);
        let gradient = LinearGradient {
            start: Point::new(Px(4.0), Px(0.0)),
            end: Point::new(Px(60.0), Px(0.0)),
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
            paint: (Paint::LinearGradient(gradient)).into(),
        });

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);
        let left = pixel_rgba(&pixels, size.0, u(8.0, sf), u(32.0, sf));
        let mid = pixel_rgba(&pixels, size.0, u(32.0, sf), u(32.0, sf));
        let right = pixel_rgba(&pixels, size.0, u(56.0, sf), u(32.0, sf));

        assert!(
            left[3] > 200 && mid[3] > 200 && right[3] > 200,
            "expected visible alpha on stroke (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );

        assert!(
            left[0] <= mid[0] && mid[0] <= right[0],
            "red must be non-decreasing across stroke (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
        assert!(
            left[1] <= mid[1] && mid[1] <= right[1],
            "green must be non-decreasing across stroke (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
        assert!(
            left[2] <= mid[2] && mid[2] <= right[2],
            "blue must be non-decreasing across stroke (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
    }
}
