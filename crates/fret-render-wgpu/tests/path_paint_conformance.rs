mod support;

use fret_core::PathService;
use fret_core::geometry::{Point, Px, Size};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint, RadialGradient,
    Scene, SceneOp, TileMode,
};
use fret_core::{FillRule, FillStyle, PathCommand, PathConstraints, PathStyle};
use fret_render_wgpu::{Renderer, WgpuContext};
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

fn prepare_fill_path(
    renderer: &mut Renderer,
    commands: &[PathCommand],
    scale_factor: f32,
) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::Fill(FillStyle {
            rule: FillRule::NonZero,
        }),
        PathConstraints { scale_factor },
    );
    id
}

#[test]
fn gpu_path_linear_gradient_smoke_conformance_across_scale_factors() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let square = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(64.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(64.0), Px(64.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(64.0))),
        PathCommand::Close,
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

    // NOTE: the paint surface for paths is evaluated in the same logical coordinate space as
    // quads (origin + local vertex pos, then scaled by `scale_factor` in the encoder).
    for sf in [1.0_f32, 1.5, 2.0] {
        let size = (u(64.0, sf), u(64.0, sf));
        let path = prepare_fill_path(&mut renderer, &square, sf);

        let gradient = LinearGradient {
            start: Point::new(Px(0.0), Px(0.0)),
            end: Point::new(Px(64.0), Px(0.0)),
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
        let left = pixel_rgba(&pixels, size.0, u(4.0, sf), u(32.0, sf));
        let mid = pixel_rgba(&pixels, size.0, u(32.0, sf), u(32.0, sf));
        let right = pixel_rgba(&pixels, size.0, u(59.0, sf), u(32.0, sf));

        assert!(
            left[3] > 240 && mid[3] > 240 && right[3] > 240,
            "expected opaque alpha (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );

        assert!(
            left[0] <= mid[0] && mid[0] <= right[0],
            "red must be non-decreasing (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
        assert!(
            left[1] <= mid[1] && mid[1] <= right[1],
            "green must be non-decreasing (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
        assert!(
            left[2] <= mid[2] && mid[2] <= right[2],
            "blue must be non-decreasing (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );

        let dr = right[0].saturating_sub(left[0]);
        let dg = right[1].saturating_sub(left[1]);
        let db = right[2].saturating_sub(left[2]);
        assert!(
            dr >= 8 && dg >= 8 && db >= 8,
            "expected visible gradient range (sf={sf}): left={left:?} mid={mid:?} right={right:?}"
        );
    }
}

#[test]
fn gpu_path_radial_gradient_smoke_conformance_across_scale_factors() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let square = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(64.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(64.0), Px(64.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(64.0))),
        PathCommand::Close,
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
        let path = prepare_fill_path(&mut renderer, &square, sf);

        let gradient = RadialGradient {
            center: Point::new(Px(32.0), Px(32.0)),
            radius: Size::new(Px(32.0), Px(32.0)),
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
            paint: (Paint::RadialGradient(gradient)).into(),
        });

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);
        let center = pixel_rgba(&pixels, size.0, u(32.0, sf), u(32.0, sf));
        let mid = pixel_rgba(&pixels, size.0, u(48.0, sf), u(32.0, sf));
        let edge = pixel_rgba(&pixels, size.0, u(62.0, sf), u(32.0, sf));

        assert!(
            center[3] > 240 && mid[3] > 240 && edge[3] > 240,
            "expected opaque alpha (sf={sf}): center={center:?} mid={mid:?} edge={edge:?}"
        );

        assert!(
            center[0] <= mid[0] && mid[0] <= edge[0],
            "red must be non-decreasing (sf={sf}): center={center:?} mid={mid:?} edge={edge:?}"
        );
        assert!(
            center[1] <= mid[1] && mid[1] <= edge[1],
            "green must be non-decreasing (sf={sf}): center={center:?} mid={mid:?} edge={edge:?}"
        );
        assert!(
            center[2] <= mid[2] && mid[2] <= edge[2],
            "blue must be non-decreasing (sf={sf}): center={center:?} mid={mid:?} edge={edge:?}"
        );
    }
}
