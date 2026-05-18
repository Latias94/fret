mod support;

use fret_core::geometry::{Point, Px, Rect, Size, Transform2D};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_core::{FillRule, FillStyle, PathCommand, PathConstraints, PathService, PathStyle};
use fret_render_wgpu::{Renderer, WgpuContext};
use support::{pixel_rgba, render_scene_rgba8};

fn white() -> Color {
    Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    }
}

fn prepare_fill_path(
    renderer: &mut Renderer,
    commands: &[PathCommand],
    rule: FillRule,
) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::Fill(FillStyle { rule }),
        PathConstraints { scale_factor: 1.0 },
    );
    id
}

fn intersecting_same_winding_regions() -> [PathCommand; 10] {
    [
        PathCommand::MoveTo(Point::new(Px(8.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(56.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(56.0), Px(64.0))),
        PathCommand::LineTo(Point::new(Px(8.0), Px(64.0))),
        PathCommand::Close,
        PathCommand::MoveTo(Point::new(Px(32.0), Px(8.0))),
        PathCommand::LineTo(Point::new(Px(64.0), Px(40.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(72.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(40.0))),
        PathCommand::Close,
    ]
}

fn centered_square_path(half: f32) -> [PathCommand; 5] {
    [
        PathCommand::MoveTo(Point::new(Px(-half), Px(-half))),
        PathCommand::LineTo(Point::new(Px(half), Px(-half))),
        PathCommand::LineTo(Point::new(Px(half), Px(half))),
        PathCommand::LineTo(Point::new(Px(-half), Px(half))),
        PathCommand::Close,
    ]
}

#[test]
fn gpu_path_fill_rules_distinguish_overlapping_winding_regions() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let commands = intersecting_same_winding_regions();

    let non_zero = prepare_fill_path(&mut renderer, &commands, FillRule::NonZero);
    let even_odd = prepare_fill_path(&mut renderer, &commands, FillRule::EvenOdd);

    let white = Paint::Solid(white());
    let mut scene = Scene::default();
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(0.0)),
        path: non_zero,
        paint: white.into(),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: Point::new(Px(0.0), Px(84.0)),
        path: even_odd,
        paint: white.into(),
    });

    let size = (72, 168);
    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let non_zero_overlap = pixel_rgba(&pixels, size.0, 32, 40);
    assert!(
        non_zero_overlap[3] > 200,
        "non-zero fill should cover same-winding intersecting overlap; got {non_zero_overlap:?}"
    );

    let even_odd_single_winding = pixel_rgba(&pixels, size.0, 12, 104);
    assert!(
        even_odd_single_winding[3] > 200,
        "even-odd fill should cover a single-winding lobe; got {even_odd_single_winding:?}"
    );

    let even_odd_overlap = pixel_rgba(&pixels, size.0, 32, 124);
    assert!(
        even_odd_overlap[3] < 40,
        "even-odd fill should clear the intersecting double-winding overlap; got {even_odd_overlap:?}"
    );
}

#[test]
fn gpu_path_transform_and_clip_compose_for_rotated_paths() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let square = centered_square_path(18.0);
    let path = prepare_fill_path(&mut renderer, &square, FillRule::NonZero);

    let center = Point::new(Px(48.0), Px(48.0));
    let transform = Transform2D::rotation_about_radians(std::f32::consts::FRAC_PI_4, center);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipRect {
        rect: Rect::new(
            Point::new(Px(24.0), Px(24.0)),
            Size::new(Px(48.0), Px(36.0)),
        ),
    });
    scene.push(SceneOp::PushTransform { transform });
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: center,
        path,
        paint: (Paint::Solid(white())).into(),
    });
    scene.push(SceneOp::PopTransform);
    scene.push(SceneOp::PopClip);

    let size = (96, 96);
    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let rotated_only_region = pixel_rgba(&pixels, size.0, 48, 27);
    assert!(
        rotated_only_region[3] > 200,
        "rotated path should cover a point outside the unrotated square but inside the clip; got {rotated_only_region:?}"
    );

    let center_pixel = pixel_rgba(&pixels, size.0, 48, 48);
    assert!(
        center_pixel[3] > 200,
        "rotated path center should remain visible; got {center_pixel:?}"
    );

    let clipped_region = pixel_rgba(&pixels, size.0, 48, 69);
    assert!(
        clipped_region[3] < 40,
        "clip rect should clip the transformed path; got {clipped_region:?}"
    );
}
