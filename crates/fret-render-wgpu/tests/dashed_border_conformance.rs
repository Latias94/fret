use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DashPatternV1, DrawOrder, Paint, Scene, SceneOp, StrokeStyleV1};
use fret_render_wgpu::{Renderer, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn dashed_border_conformance_stroke_rrect_masks_border_coverage() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let rect_origin = (20.0_f32, 20.0_f32);
    let rect_size = (200.0_f32, 80.0_f32);
    let radius = 16.0_f32;
    let stroke_w = 4.0_f32;

    let dash = DashPatternV1::new(Px(10.0), Px(6.0), Px(0.0));
    let style = StrokeStyleV1 { dash: Some(dash) };

    let mut scene = Scene::default();
    scene.push(SceneOp::StrokeRRect {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(rect_origin.0), Px(rect_origin.1)),
            Size::new(Px(rect_size.0), Px(rect_size.1)),
        ),
        stroke: Edges::all(Px(stroke_w)),
        stroke_paint: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })
        .into(),
        corner_radii: Corners::all(Px(radius)),
        style,
    });

    fn u(v: f32, sf: f32) -> u32 {
        (v * sf).round() as u32
    }

    for scale_factor in [1.0_f32, 1.5_f32, 2.0_f32] {
        let size = (u(256.0, scale_factor), u(128.0, scale_factor));
        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, scale_factor);

        let start_x = rect_origin.0 + radius;
        let y_top = rect_origin.1 + stroke_w * 0.5;

        // Dash-on sample: s=3 (inside [0..dash)).
        let on_x = start_x + 3.0;
        let on = pixel_rgba(
            &pixels,
            size.0,
            u(on_x, scale_factor),
            u(y_top, scale_factor),
        );

        // Dash-off sample: s=13 (inside (dash..dash+gap)).
        let off_x = start_x + 13.0;
        let off = pixel_rgba(
            &pixels,
            size.0,
            u(off_x, scale_factor),
            u(y_top, scale_factor),
        );

        // Interior sample: should remain transparent.
        let mid_x = rect_origin.0 + rect_size.0 * 0.5;
        let mid_y = rect_origin.1 + rect_size.1 * 0.5;
        let mid = pixel_rgba(
            &pixels,
            size.0,
            u(mid_x, scale_factor),
            u(mid_y, scale_factor),
        );

        assert!(
            on[3] > 200,
            "expected dash-on pixel alpha to be high; got rgba={on:?} sf={scale_factor}"
        );
        assert!(
            off[3] < 40,
            "expected dash-off pixel alpha to be low; got rgba={off:?} sf={scale_factor}"
        );
        assert!(
            mid[3] < 5,
            "expected interior pixel alpha to remain transparent; got rgba={mid:?} sf={scale_factor}"
        );
    }
}
