mod support;

use fret_core::geometry::{Point, Px, Rect, Size, Transform2D};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_core::{FillRule, FillStyle, PathCommand, PathConstraints, PathService, PathStyle};
use fret_render_wgpu::{Renderer, WgpuContext};
use support::{pixel_rgba, render_scene_rgba8};

fn rotation_about(center: Point, radians: f32) -> Transform2D {
    let (sin, cos) = radians.sin_cos();
    let rot = Transform2D {
        a: cos,
        b: sin,
        c: -sin,
        d: cos,
        tx: 0.0,
        ty: 0.0,
    };

    let neg_center = Point::new(Px(-center.x.0), Px(-center.y.0));
    Transform2D::translation(center) * rot * Transform2D::translation(neg_center)
}

fn prepare_fill_path(renderer: &mut Renderer, commands: &[PathCommand]) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::Fill(FillStyle {
            rule: FillRule::NonZero,
        }),
        PathConstraints { scale_factor: 1.0 },
    );
    id
}

#[test]
fn gpu_clip_path_clips_to_shape_not_just_bounds() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let tri = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &tri);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopClip);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let inside = pixel_rgba(&pixels, size.0, 8, 8);
    assert_eq!(
        inside,
        [0, 255, 0, 255],
        "inside clip-path should be visible"
    );

    let outside = pixel_rgba(&pixels, size.0, 48, 48);
    assert_eq!(
        outside,
        [0, 0, 0, 0],
        "outside clip-path should remain clear"
    );
}

#[test]
fn gpu_clip_path_is_captured_at_push_time_and_does_not_follow_later_transforms() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let square = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(16.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(16.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &square);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });

    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(16.0), Px(16.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    scene.push(SceneOp::PushTransform {
        transform: Transform2D::translation(Point::new(Px(32.0), Px(0.0))),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(16.0), Px(16.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopTransform);

    scene.push(SceneOp::PopClip);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let inside = pixel_rgba(&pixels, size.0, 8, 8);
    assert_eq!(
        inside,
        [0, 0, 255, 255],
        "quad drawn inside captured clip-path should be visible"
    );

    let translated = pixel_rgba(&pixels, size.0, 40, 8);
    assert_eq!(
        translated,
        [0, 0, 0, 0],
        "clip-path must not follow transforms pushed after the clip entry"
    );
}

#[test]
fn gpu_clip_path_clip_before_transform_partial_overlap_is_clipped() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let square = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(16.0), Px(16.0))),
        PathCommand::LineTo(Point::new(Px(16.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &square);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(32.0))),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });

    scene.push(SceneOp::PushTransform {
        transform: Transform2D::translation(Point::new(Px(8.0), Px(0.0))),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(16.0), Px(16.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopTransform);

    scene.push(SceneOp::PopClip);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let left = pixel_rgba(&pixels, size.0, 4, 8);
    assert_eq!(
        left,
        [0, 0, 0, 0],
        "quad portion fully outside the clip-path should remain clear"
    );

    let overlap = pixel_rgba(&pixels, size.0, 12, 8);
    assert_eq!(
        overlap,
        [0, 0, 255, 255],
        "quad portion overlapping captured clip-path should be visible"
    );

    let right = pixel_rgba(&pixels, size.0, 20, 8);
    assert_eq!(
        right,
        [0, 0, 0, 0],
        "quad portion outside the clip-path on the far side should remain clear"
    );
}

#[test]
fn gpu_clip_path_under_affine_rotation_clips_in_rotated_space() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let square = [
        PathCommand::MoveTo(Point::new(Px(-12.0), Px(-12.0))),
        PathCommand::LineTo(Point::new(Px(-12.0), Px(12.0))),
        PathCommand::LineTo(Point::new(Px(12.0), Px(12.0))),
        PathCommand::LineTo(Point::new(Px(12.0), Px(-12.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &square);

    let center = Point::new(Px(32.0), Px(32.0));
    let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushTransform { transform });
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: center,
        path,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopClip);
    scene.push(SceneOp::PopTransform);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let inside = pixel_rgba(&pixels, size.0, 32, 32);
    assert_eq!(
        inside,
        [0, 255, 0, 255],
        "center must be inside rotated clip"
    );

    let outside = pixel_rgba(&pixels, size.0, 32, 8);
    assert_eq!(
        outside,
        [0, 0, 0, 0],
        "point outside rotated clip must remain clear"
    );
}

#[test]
fn gpu_rectangular_clip_path_matches_clip_rect_without_transform() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let rect_path = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &rect_path);

    let clip_rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(32.0)));

    let mut rect_scene = Scene::default();
    rect_scene.push(SceneOp::PushClipRect { rect: clip_rect });
    rect_scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    rect_scene.push(SceneOp::PopClip);

    let mut path_scene = Scene::default();
    path_scene.push(SceneOp::PushClipPath {
        bounds: clip_rect,
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });
    path_scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    path_scene.push(SceneOp::PopClip);

    let rect_pixels = render_scene_rgba8(&ctx, &mut renderer, &rect_scene, size, 1.0);
    let path_pixels = render_scene_rgba8(&ctx, &mut renderer, &path_scene, size, 1.0);

    let p_inside = (8u32, 8u32);
    assert_eq!(
        pixel_rgba(&rect_pixels, size.0, p_inside.0, p_inside.1),
        pixel_rgba(&path_pixels, size.0, p_inside.0, p_inside.1),
        "rect clip and rectangular clip-path must agree at an interior sample"
    );

    let p_outside = (40u32, 8u32);
    assert_eq!(
        pixel_rgba(&rect_pixels, size.0, p_outside.0, p_outside.1),
        pixel_rgba(&path_pixels, size.0, p_outside.0, p_outside.1),
        "rect clip and rectangular clip-path must agree at an outside sample"
    );
}

#[test]
fn gpu_nested_clip_path_with_composite_group_clips_group_content() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let tri = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(48.0))),
        PathCommand::LineTo(Point::new(Px(48.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &tri);

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });

    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: Point::new(Px(8.0), Px(8.0)),
        path,
    });

    scene.push(SceneOp::PushCompositeGroup {
        desc: fret_core::CompositeGroupDesc::new(
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            fret_core::BlendMode::Over,
            fret_core::EffectQuality::Auto,
        )
        .with_opacity(1.0),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopCompositeGroup);

    scene.push(SceneOp::PopClip);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let inside = pixel_rgba(&pixels, size.0, 16, 16);
    assert_eq!(
        inside,
        [0, 0, 255, 255],
        "inside clip-path, composite group content should be visible"
    );

    let outside = pixel_rgba(&pixels, size.0, 56, 56);
    assert_eq!(
        outside,
        [255, 0, 0, 255],
        "outside clip-path, background should remain (group must be clipped)"
    );
}

#[test]
fn gpu_nested_clip_rect_then_clip_path_composes() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let size = (64u32, 64u32);

    let tri = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(48.0))),
        PathCommand::LineTo(Point::new(Px(48.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &tri);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipRect {
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(64.0))),
    });
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopClip);
    scene.push(SceneOp::PopClip);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let inside = pixel_rgba(&pixels, size.0, 8, 8);
    assert_eq!(
        inside,
        [0, 255, 0, 255],
        "inside nested clip stack should be visible"
    );

    let clipped_by_rect = pixel_rgba(&pixels, size.0, 48, 8);
    assert_eq!(
        clipped_by_rect,
        [0, 0, 0, 0],
        "outer clip-rect must still clip clip-path content"
    );
}

#[test]
fn gpu_clip_path_degrades_to_scissor_only_under_tight_intermediate_budget() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(1024);

    let size = (64u32, 64u32);

    let tri = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(0.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &tri);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipPath {
        bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: (Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        border: Default::default(),
        border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
        corner_radii: Default::default(),
    });
    scene.push(SceneOp::PopClip);

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let outside = pixel_rgba(&pixels, size.0, 48, 48);
    assert_eq!(
        outside,
        [0, 255, 0, 255],
        "under tight budgets, clip-path should deterministically degrade (scissor-only fallback)"
    );
}
