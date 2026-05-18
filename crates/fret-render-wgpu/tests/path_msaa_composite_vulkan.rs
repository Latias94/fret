use fret_core::geometry::{Point, Px};
use fret_core::geometry::{Rect, Size};
use fret_core::scene::{Color, DrawOrder, Scene, SceneOp};
use fret_core::{PathCommand, PathConstraints, PathService, PathStyle, StrokeStyle};
use fret_render_wgpu::{ClearColor, RenderSceneParams, Renderer, WgpuContext};

mod support;

use support::{pixel_rgba as pixel_bgra, read_texture_rgba8};

#[test]
fn gpu_path_msaa_composite_vulkan_smoke() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    if ctx.adapter.get_info().backend != wgpu::Backend::Vulkan {
        // This test targets the Vulkan backend specifically.
        return;
    }

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_path_msaa_samples(4);

    let size = (256u32, 256u32);
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("path_msaa_composite_vulkan output"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let (path_top, _metrics) = renderer.prepare(
        &[
            PathCommand::MoveTo(Point::new(Px(16.0), Px(80.0))),
            PathCommand::LineTo(Point::new(Px(240.0), Px(80.0))),
        ],
        PathStyle::Stroke(StrokeStyle { width: Px(6.0) }),
        PathConstraints { scale_factor: 1.0 },
    );
    let (path_bottom, _metrics) = renderer.prepare(
        &[
            PathCommand::MoveTo(Point::new(Px(16.0), Px(160.0))),
            PathCommand::LineTo(Point::new(Px(240.0), Px(160.0))),
        ],
        PathStyle::Stroke(StrokeStyle { width: Px(6.0) }),
        PathConstraints { scale_factor: 1.0 },
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipRect {
        rect: Rect::new(
            Point::new(Px(64.0), Px(64.0)),
            Size::new(Px(128.0), Px(64.0)),
        ),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(0.0)),
        path: path_top,
        paint: Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
        .into(),
    });
    scene.push(SceneOp::PopClip);

    // Ensure we generate a second `PathMsaaBatch` pass by changing the clip stack and thus the
    // path draw's uniform index. The old (buggy) renderer wrote the composite quad vertices into
    // a shared buffer at offset 0 for each pass, which meant only the final write was observed by
    // all passes in the same submission.
    scene.push(SceneOp::PushClipRect {
        rect: Rect::new(
            Point::new(Px(64.0), Px(144.0)),
            Size::new(Px(128.0), Px(64.0)),
        ),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(0.0)),
        path: path_bottom,
        paint: Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
        .into(),
    });
    scene.push(SceneOp::PopClip);

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &view,
            scene: &scene,
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

    let pixels = read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size);
    // BGRA order for Bgra8 outputs.
    let top_line = pixel_bgra(&pixels, size.0, 128, 80);
    let bottom_line = pixel_bgra(&pixels, size.0, 128, 160);

    // Expect the red stroke to be visible at y=80 and the green stroke at y=160.
    assert!(
        top_line[3] > 32 && top_line[2] > 32,
        "expected a visible red pixel at (128, 80), got BGRA={top_line:?}"
    );
    assert!(
        bottom_line[3] > 32 && bottom_line[1] > 32,
        "expected a visible green pixel at (128, 160), got BGRA={bottom_line:?}"
    );
}
