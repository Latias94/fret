use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size, Transform2D};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint, PaintBindingV1,
    PaintEvalSpaceV1, Scene, SceneOp, TileMode,
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

#[test]
fn quad_paint_viewport_px_differs_from_local_px() {
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
        let size = (u(256.0, sf), u(128.0, sf));
        let rect = Rect::new(
            Point::new(Px(0.0), Px(10.0)),
            Size::new(Px(100.0), Px(60.0)),
        );
        let transform = Transform2D::translation(Point::new(Px(50.0), Px(0.0)));
        let gradient = LinearGradient {
            start: Point::new(Px(0.0), Px(0.0)),
            end: Point::new(Px(100.0), Px(0.0)),
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count,
            stops,
        };
        let paint = Paint::LinearGradient(gradient);

        let mut local_scene = Scene::default();
        local_scene.push(SceneOp::PushTransform { transform });
        local_scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect,
            background: PaintBindingV1::with_eval_space(paint, PaintEvalSpaceV1::LocalPx),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        local_scene.push(SceneOp::PopTransform);

        let mut viewport_scene = Scene::default();
        viewport_scene.push(SceneOp::PushTransform { transform });
        viewport_scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect,
            background: PaintBindingV1::with_eval_space(paint, PaintEvalSpaceV1::ViewportPx),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        viewport_scene.push(SceneOp::PopTransform);

        let local_pixels = render_scene_rgba8(&ctx, &mut renderer, &local_scene, size, sf);
        let viewport_pixels = render_scene_rgba8(&ctx, &mut renderer, &viewport_scene, size, sf);

        // Sample at the same pixel coordinate inside the quad. The transform shifts the quad by +50
        // in viewport pixels without changing its local scene coordinates. LocalPx evaluates at
        // x=50 within the quad, while ViewportPx evaluates at x=100 in the viewport.
        let sample_x = 100.0;
        let sample_y = rect.origin.y.0 + 30.0;
        let local = pixel_rgba(&local_pixels, size.0, u(sample_x, sf), u(sample_y, sf));
        let viewport = pixel_rgba(&viewport_pixels, size.0, u(sample_x, sf), u(sample_y, sf));

        assert!(
            local[3] > 240 && viewport[3] > 240,
            "expected opaque alpha (sf={sf}): local={local:?} viewport={viewport:?}"
        );
        assert!(
            local[0] < viewport[0],
            "expected viewport-space to be brighter at x=100 (sf={sf}): local={local:?} viewport={viewport:?}"
        );
        assert!(
            viewport[0] > 240,
            "expected near-white in viewport-space at x=100 (sf={sf}): viewport={viewport:?}"
        );
        assert!(
            local[0] > 40 && local[0] < 220,
            "expected mid-gray in local-space at x=50 (sf={sf}): local={local:?}"
        );
    }
}
