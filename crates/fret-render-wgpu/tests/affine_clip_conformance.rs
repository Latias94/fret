mod support;

use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size, Transform2D};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_render_wgpu::{
    ClearColor, RenderSceneParams, RenderTargetColorSpace, RenderTargetDescriptor, Renderer,
    WgpuContext,
};
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

fn sample_point(t: Transform2D, p: Point, size: (u32, u32)) -> (u32, u32) {
    let p = t.apply_point(p);
    let x = p.x.0.round().clamp(0.0, (size.0 - 1) as f32) as u32;
    let y = p.y.0.round().clamp(0.0, (size.1 - 1) as f32) as u32;
    (x, y)
}

#[test]
fn gpu_offscreen_identity_blit_matches_direct() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(4.0), Px(6.0)), Size::new(Px(56.0), Px(52.0))),
        background: (Paint::Solid(Color {
            r: 0.2,
            g: 0.4,
            b: 0.8,
            a: 0.75,
        }))
        .into(),
        border: Edges::all(Px(1.0)),
        border_paint: (Paint::Solid(Color {
            r: 1.0,
            g: 0.5,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        corner_radii: Corners::all(Px(8.0)),
    });

    let direct = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    renderer.set_debug_offscreen_blit_enabled(true);
    let offscreen = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    renderer.set_debug_offscreen_blit_enabled(false);

    assert_eq!(direct, offscreen, "offscreen blit output must match direct");
}

#[test]
fn gpu_affine_clip_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let center = Point::new(Px(32.0), Px(32.0));

    // 1) Clip rect is evaluated in clip-local space under affine transform.
    {
        let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
        let mut scene = Scene::default();
        scene.push(SceneOp::PushTransform { transform });
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopClip);
        scene.push(SceneOp::PopTransform);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let outside = pixel_rgba(&pixels, size.0, 16, 16);
        let inside = pixel_rgba(&pixels, size.0, 32, 32);

        assert!(
            inside[3] > 200,
            "clip_rect: expected inside pixel to be opaque, got {inside:?}"
        );
        assert!(
            outside[3] < 20,
            "clip_rect: expected outside pixel to be transparent, got {outside:?}"
        );
    }

    // 2) Clip rect is captured at push time (clip-before-transform must remain fixed).
    {
        let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
        let mut scene = Scene::default();
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
        });
        scene.push(SceneOp::PushTransform { transform });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopTransform);
        scene.push(SceneOp::PopClip);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let inside = pixel_rgba(&pixels, size.0, 17, 17);
        let outside = pixel_rgba(&pixels, size.0, 8, 8);

        assert!(
            inside[3] > 200,
            "clip_capture: expected inside pixel to be opaque, got {inside:?}"
        );
        assert!(
            outside[3] < 20,
            "clip_capture: expected outside pixel to be transparent, got {outside:?}"
        );
    }

    // 3) Rounded clip corners must be enforced via shader clip (scissor alone is insufficient).
    {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushClipRRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
            corner_radii: Corners::all(Px(8.0)),
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopClip);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let outside = pixel_rgba(&pixels, size.0, 17, 17);
        let inside = pixel_rgba(&pixels, size.0, 24, 24);

        assert!(
            inside[3] > 200,
            "clip_rrect: expected inside pixel to be opaque, got {inside:?}"
        );
        assert!(
            outside[3] < 20,
            "clip_rrect: expected corner pixel to be clipped, got {outside:?}"
        );
    }

    // 4) Rounded clip must also be evaluated in clip-local space under affine transforms.
    {
        let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
        let mut scene = Scene::default();
        scene.push(SceneOp::PushTransform { transform });
        scene.push(SceneOp::PushClipRRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
            corner_radii: Corners::all(Px(8.0)),
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopClip);
        scene.push(SceneOp::PopTransform);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let outside = pixel_rgba(&pixels, size.0, 16, 16);
        let inside = pixel_rgba(&pixels, size.0, 32, 32);

        assert!(
            inside[3] > 200,
            "clip_rrect_affine: expected inside pixel to be opaque, got {inside:?}"
        );
        assert!(
            outside[3] < 20,
            "clip_rrect_affine: expected outside pixel to be transparent, got {outside:?}"
        );
    }

    // 5) clip-before-transform should keep the clip fixed while content moves under it (scrolling).
    {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
        });
        scene.push(SceneOp::PushTransform {
            transform: Transform2D::translation(Point::new(Px(24.0), Px(0.0))),
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopTransform);
        scene.push(SceneOp::PopClip);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

        let inside_clip_but_empty = pixel_rgba(&pixels, size.0, 18, 32);
        let inside_clip_and_filled = pixel_rgba(&pixels, size.0, 30, 32);

        assert!(
            inside_clip_but_empty[3] < 20,
            "scrolling: expected inside-clip pixel to be transparent, got {inside_clip_but_empty:?}"
        );
        assert!(
            inside_clip_and_filled[3] > 200,
            "scrolling: expected inside-clip pixel to be opaque, got {inside_clip_and_filled:?}"
        );
    }

    // 6) ViewportSurface must also respect the affine clip model (not just scissor bounds).
    {
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let source_size = (16u32, 16u32);
        let source_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("affine_clip_conformance viewport source"),
            size: wgpu::Extent3d {
                width: source_size.0,
                height: source_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let source_render_view =
            source_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut source_scene = Scene::default();
        source_scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(16.0), Px(16.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });

        let source_cb = renderer.render_scene(
            &ctx.device,
            &ctx.queue,
            RenderSceneParams {
                format,
                target_view: &source_render_view,
                scene: &source_scene,
                scene_chunks: None,
                clear: ClearColor(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }),
                scale_factor: 1.0,
                viewport_size: source_size,
            },
        );
        ctx.queue.submit([source_cb]);
        let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

        let source_sample_view =
            source_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let target = renderer.register_render_target(RenderTargetDescriptor {
            view: source_sample_view,
            size: source_size,
            format,
            color_space: RenderTargetColorSpace::Linear,
            metadata: fret_render_wgpu::RenderTargetMetadata::default(),
        });

        let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
        let mut scene = Scene::default();
        scene.push(SceneOp::PushTransform { transform });
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
        });
        scene.push(SceneOp::ViewportSurface {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            target,
            opacity: 1.0,
        });
        scene.push(SceneOp::PopClip);
        scene.push(SceneOp::PopTransform);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let outside = pixel_rgba(&pixels, size.0, 16, 16);
        let inside = pixel_rgba(&pixels, size.0, 32, 32);

        assert!(
            inside[3] > 200,
            "viewport: expected inside pixel to be opaque, got {inside:?}"
        );
        assert!(
            outside[3] < 20,
            "viewport: expected outside pixel to be transparent, got {outside:?}"
        );
    }

    // 7) Nested clips must intersect (not replace).
    {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
        });
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(24.0), Px(24.0)),
                Size::new(Px(16.0), Px(16.0)),
            ),
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopClip);
        scene.push(SceneOp::PopClip);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let inside_inner = pixel_rgba(&pixels, size.0, 32, 32);
        let inside_outer_only = pixel_rgba(&pixels, size.0, 20, 20);
        let outside_all = pixel_rgba(&pixels, size.0, 8, 8);

        assert!(
            inside_inner[3] > 200,
            "nested_clips: expected inner intersection pixel to be opaque, got {inside_inner:?}"
        );
        assert!(
            inside_outer_only[3] < 20,
            "nested_clips: expected outer-only pixel to be clipped, got {inside_outer_only:?}"
        );
        assert!(
            outside_all[3] < 20,
            "nested_clips: expected outside pixel to be transparent, got {outside_all:?}"
        );
    }

    // 8) Rounded clips are captured at push time (clip-before-transform stays fixed).
    {
        let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
        let mut scene = Scene::default();
        scene.push(SceneOp::PushClipRRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
            corner_radii: Corners::all(Px(8.0)),
        });
        scene.push(SceneOp::PushTransform { transform });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopTransform);
        scene.push(SceneOp::PopClip);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let inside_axis_aligned = pixel_rgba(&pixels, size.0, 20, 20);
        let outside_axis_aligned = pixel_rgba(&pixels, size.0, 8, 8);

        assert!(
            inside_axis_aligned[3] > 200,
            "clip_rrect_capture: expected inside pixel to be opaque, got {inside_axis_aligned:?}"
        );
        assert!(
            outside_axis_aligned[3] < 20,
            "clip_rrect_capture: expected outside pixel to be transparent, got {outside_axis_aligned:?}"
        );
    }

    // 9) Deep clip stacks must work (exceeds legacy uniform MAX_CLIPS designs).
    {
        let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
        let mut scene = Scene::default();
        scene.push(SceneOp::PushTransform { transform });

        for _ in 0..64 {
            scene.push(SceneOp::PushClipRect {
                rect: Rect::new(
                    Point::new(Px(16.0), Px(16.0)),
                    Size::new(Px(32.0), Px(32.0)),
                ),
            });
        }

        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });

        for _ in 0..64 {
            scene.push(SceneOp::PopClip);
        }
        scene.push(SceneOp::PopTransform);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let outside = pixel_rgba(&pixels, size.0, 16, 16);
        let inside = pixel_rgba(&pixels, size.0, 32, 32);

        assert!(
            inside[3] > 200,
            "deep_clip_stack: expected inside pixel to be opaque, got {inside:?}"
        );
        assert!(
            outside[3] < 20,
            "deep_clip_stack: expected outside pixel to be transparent, got {outside:?}"
        );
    }

    // 10) Deep rounded clip stacks must also work (shader clip, not scissor).
    {
        let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
        let mut scene = Scene::default();
        scene.push(SceneOp::PushTransform { transform });

        for _ in 0..64 {
            scene.push(SceneOp::PushClipRRect {
                rect: Rect::new(
                    Point::new(Px(16.0), Px(16.0)),
                    Size::new(Px(32.0), Px(32.0)),
                ),
                corner_radii: Corners::all(Px(8.0)),
            });
        }

        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });

        for _ in 0..64 {
            scene.push(SceneOp::PopClip);
        }
        scene.push(SceneOp::PopTransform);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
        let (inside_x, inside_y) = sample_point(transform, Point::new(Px(32.0), Px(32.0)), size);
        let (outside_x, outside_y) = sample_point(transform, Point::new(Px(4.0), Px(4.0)), size);
        let inside = pixel_rgba(&pixels, size.0, inside_x, inside_y);
        let outside = pixel_rgba(&pixels, size.0, outside_x, outside_y);

        assert!(
            inside[3] > 200,
            "deep_rrect_stack: expected inside pixel to be opaque, got {inside:?}"
        );
        assert!(
            outside[3] < 20,
            "deep_rrect_stack: expected outside pixel to be transparent, got {outside:?}"
        );
    }

    // 11) Nested mixed clips must intersect correctly under affine transforms.
    {
        let transform = rotation_about(center, std::f32::consts::FRAC_PI_4);
        let mut scene = Scene::default();
        scene.push(SceneOp::PushTransform { transform });
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(
                Point::new(Px(12.0), Px(12.0)),
                Size::new(Px(40.0), Px(40.0)),
            ),
        });
        scene.push(SceneOp::PushClipRRect {
            rect: Rect::new(
                Point::new(Px(16.0), Px(16.0)),
                Size::new(Px(32.0), Px(32.0)),
            ),
            corner_radii: Corners::all(Px(8.0)),
        });
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
            background: (Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }))
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: (Paint::Solid(Color::TRANSPARENT)).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        scene.push(SceneOp::PopClip);
        scene.push(SceneOp::PopClip);
        scene.push(SceneOp::PopTransform);

        let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

        let (inside_x, inside_y) = sample_point(transform, Point::new(Px(32.0), Px(32.0)), size);
        let (outer_only_x, outer_only_y) =
            sample_point(transform, Point::new(Px(14.0), Px(32.0)), size);
        let (outside_x, outside_y) = sample_point(transform, Point::new(Px(4.0), Px(4.0)), size);

        let inside = pixel_rgba(&pixels, size.0, inside_x, inside_y);
        let outer_only = pixel_rgba(&pixels, size.0, outer_only_x, outer_only_y);
        let outside = pixel_rgba(&pixels, size.0, outside_x, outside_y);

        assert!(
            inside[3] > 200,
            "nested_mixed_clips: expected intersection pixel to be opaque, got {inside:?}"
        );
        assert!(
            outer_only[3] < 20,
            "nested_mixed_clips: expected outer-only pixel to be clipped, got {outer_only:?}"
        );
        assert!(
            outside[3] < 20,
            "nested_mixed_clips: expected outside pixel to be transparent, got {outside:?}"
        );
    }
}
