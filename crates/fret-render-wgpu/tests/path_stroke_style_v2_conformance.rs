mod support;

use fret_core::PathService;
use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Scene, SceneOp};
use fret_core::{
    PathCommand, PathConstraints, PathStyle, StrokeCapV1, StrokeJoinV1, StrokeStyleV2,
};
use fret_render_wgpu::{Renderer, WgpuContext};
use support::{pixel_rgba, render_scene_rgba8};

fn u(v: f32, sf: f32) -> u32 {
    (v * sf).round() as u32
}

#[test]
fn path_stroke_style_v2_join_miter_vs_bevel_has_expected_corner_coverage() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    // An L-shaped polyline with a 90° corner at (50, 10).
    let cmds = [
        PathCommand::MoveTo(Point::new(Px(10.0), Px(10.0))),
        PathCommand::LineTo(Point::new(Px(50.0), Px(10.0))),
        PathCommand::LineTo(Point::new(Px(50.0), Px(50.0))),
    ];

    let constraints = PathConstraints { scale_factor: 1.0 };

    let stroke = |join| StrokeStyleV2 {
        width: Px(20.0),
        join,
        cap: StrokeCapV1::Butt,
        miter_limit: 16.0,
        dash: None,
    };

    let (path_miter, _m) = renderer.prepare(
        &cmds,
        PathStyle::StrokeV2(stroke(StrokeJoinV1::Miter)),
        constraints,
    );
    let (path_bevel, _m) = renderer.prepare(
        &cmds,
        PathStyle::StrokeV2(stroke(StrokeJoinV1::Bevel)),
        constraints,
    );

    let origin_miter = Point::new(Px(0.0), Px(0.0));
    let origin_bevel = Point::new(Px(120.0), Px(0.0));

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(120.0)),
        ),
        background: (fret_core::Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }))
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: (fret_core::Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }))
        .into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let white = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: origin_miter,
        path: path_miter,
        paint: white.into(),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(2),
        origin: origin_bevel,
        path: path_bevel,
        paint: white.into(),
    });

    // Sample a point that lies inside the miter "square corner" but outside the bevel diagonal.
    //
    // For a 90° corner, the miter join outer boundary forms a right-angle "square corner"
    // region. The bevel join replaces that outer corner with a diagonal; the inside region lies
    // on the side of the diagonal closer to the centerline. We pick a point well inside the
    // miter square but on the *outside* of the bevel diagonal.
    //
    // See ADR 0277 for the intended semantics.
    let sample_local = (59.0_f32, 2.0_f32);
    let sample_miter = (
        origin_miter.x.0 + sample_local.0,
        origin_miter.y.0 + sample_local.1,
    );
    let sample_bevel = (
        origin_bevel.x.0 + sample_local.0,
        origin_bevel.y.0 + sample_local.1,
    );

    for sf in [1.0_f32, 1.5_f32, 2.0_f32] {
        let size = (u(300.0, sf), u(120.0, sf));
        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);

        let m = pixel_rgba(
            &pixels,
            size.0,
            u(sample_miter.0, sf),
            u(sample_miter.1, sf),
        );
        let b = pixel_rgba(
            &pixels,
            size.0,
            u(sample_bevel.0, sf),
            u(sample_bevel.1, sf),
        );

        assert!(
            m[3] > 200,
            "expected miter join pixel alpha to be high; got rgba={m:?} sf={sf}"
        );
        assert!(
            b[3] < 40,
            "expected bevel join pixel alpha to be low; got rgba={b:?} sf={sf}"
        );
    }
}

#[test]
fn path_stroke_style_v2_dash_periodicity_and_phase_are_deterministic_across_scale_factors() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let cmds = [
        PathCommand::MoveTo(Point::new(Px(10.0), Px(30.0))),
        PathCommand::LineTo(Point::new(Px(210.0), Px(30.0))),
    ];
    let constraints = PathConstraints { scale_factor: 1.0 };

    let stroke = |phase: f32| StrokeStyleV2 {
        width: Px(12.0),
        join: StrokeJoinV1::Miter,
        cap: StrokeCapV1::Butt,
        miter_limit: 4.0,
        dash: Some(fret_core::scene::DashPatternV1::new(
            Px(20.0),
            Px(20.0),
            Px(phase),
        )),
    };

    let (path_phase0, _m0) = renderer.prepare(&cmds, PathStyle::StrokeV2(stroke(0.0)), constraints);
    let (path_phase10, _m10) =
        renderer.prepare(&cmds, PathStyle::StrokeV2(stroke(10.0)), constraints);

    let origin_phase0 = Point::new(Px(0.0), Px(0.0));
    let origin_phase10 = Point::new(Px(0.0), Px(80.0));

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(160.0)),
        ),
        background: (fret_core::Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }))
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: (fret_core::Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }))
        .into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let white = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: origin_phase0,
        path: path_phase0,
        paint: white.into(),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(2),
        origin: origin_phase10,
        path: path_phase10,
        paint: white.into(),
    });

    let sample_phase0_on = (20.0_f32, 30.0_f32); // distance 10: inside dash
    let sample_phase0_off = (40.0_f32, 30.0_f32); // distance 30: inside gap
    let sample_phase10_on = (15.0_f32, 110.0_f32); // distance 5: inside dash (phase=10)
    let sample_phase10_off = (25.0_f32, 110.0_f32); // distance 15: inside gap (phase=10)

    for sf in [1.0_f32, 1.5_f32, 2.0_f32] {
        let size = (u(260.0, sf), u(160.0, sf));
        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);

        let p0_on = pixel_rgba(
            &pixels,
            size.0,
            u(sample_phase0_on.0, sf),
            u(sample_phase0_on.1, sf),
        );
        let p0_off = pixel_rgba(
            &pixels,
            size.0,
            u(sample_phase0_off.0, sf),
            u(sample_phase0_off.1, sf),
        );
        let p10_on = pixel_rgba(
            &pixels,
            size.0,
            u(sample_phase10_on.0, sf),
            u(sample_phase10_on.1, sf),
        );
        let p10_off = pixel_rgba(
            &pixels,
            size.0,
            u(sample_phase10_off.0, sf),
            u(sample_phase10_off.1, sf),
        );

        assert!(
            p0_on[3] > 200,
            "expected phase=0 dash pixel alpha to be high; got rgba={p0_on:?} sf={sf}"
        );
        assert!(
            p0_off[3] < 40,
            "expected phase=0 gap pixel alpha to be low; got rgba={p0_off:?} sf={sf}"
        );
        assert!(
            p10_on[3] > 200,
            "expected phase=10 dash pixel alpha to be high; got rgba={p10_on:?} sf={sf}"
        );
        assert!(
            p10_off[3] < 40,
            "expected phase=10 gap pixel alpha to be low; got rgba={p10_off:?} sf={sf}"
        );
    }
}

#[test]
fn path_stroke_style_v2_round_cap_extends_coverage_beyond_butt_across_scale_factors() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let cmds = [
        PathCommand::MoveTo(Point::new(Px(20.0), Px(30.0))),
        PathCommand::LineTo(Point::new(Px(80.0), Px(30.0))),
    ];
    let constraints = PathConstraints { scale_factor: 1.0 };

    let stroke = |cap| StrokeStyleV2 {
        width: Px(20.0),
        join: StrokeJoinV1::Miter,
        cap,
        miter_limit: 4.0,
        dash: None,
    };

    let (path_butt, _mb) = renderer.prepare(
        &cmds,
        PathStyle::StrokeV2(stroke(StrokeCapV1::Butt)),
        constraints,
    );
    let (path_round, _mr) = renderer.prepare(
        &cmds,
        PathStyle::StrokeV2(stroke(StrokeCapV1::Round)),
        constraints,
    );

    let origin_butt = Point::new(Px(0.0), Px(0.0));
    let origin_round = Point::new(Px(0.0), Px(80.0));

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(160.0), Px(160.0)),
        ),
        background: (fret_core::Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }))
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: (fret_core::Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }))
        .into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let white = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: origin_butt,
        path: path_butt,
        paint: white.into(),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(2),
        origin: origin_round,
        path: path_round,
        paint: white.into(),
    });

    // The line ends at x=80; width=20 => radius=10. Round cap should cover ~x in (80..90).
    let sample_beyond_end_butt = (88.0_f32, 30.0_f32);
    let sample_beyond_end_round = (88.0_f32, 110.0_f32);

    for sf in [1.0_f32, 1.5_f32, 2.0_f32] {
        let size = (u(160.0, sf), u(160.0, sf));
        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, sf);

        let b = pixel_rgba(
            &pixels,
            size.0,
            u(sample_beyond_end_butt.0, sf),
            u(sample_beyond_end_butt.1, sf),
        );
        let r = pixel_rgba(
            &pixels,
            size.0,
            u(sample_beyond_end_round.0, sf),
            u(sample_beyond_end_round.1, sf),
        );

        assert!(
            b[3] < 40,
            "expected butt cap pixel alpha beyond end to be low; got rgba={b:?} sf={sf}"
        );
        assert!(
            r[3] > 200,
            "expected round cap pixel alpha beyond end to be high; got rgba={r:?} sf={sf}"
        );
    }
}
