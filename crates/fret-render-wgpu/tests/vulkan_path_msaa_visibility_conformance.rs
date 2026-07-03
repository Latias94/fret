use fret_core::PathService as _;
use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_core::{FillStyle, PathCommand, PathConstraints, PathStyle};
use fret_render_wgpu::{
    ClearColor, RenderSceneParams, RenderSceneSourceSelection, Renderer, WgpuContext,
};
use std::ffi::OsString;
use std::sync::{Mutex, OnceLock};

#[path = "support/readback.rs"]
mod support;

use support::{pixel_rgba, read_texture_rgba8};

struct EnvVarGuard {
    key: &'static str,
    prev: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let prev = std::env::var_os(key);
        // Safety: this is a test-only opt-in knob. `std::env::set_var` is process-global and
        // considered unsafe under Rust's data-race model; we keep the mutation scoped to this
        // test process via a guard.
        unsafe { std::env::set_var(key, value) };
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => unsafe { std::env::set_var(self.key, v) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

fn u(v: f32, sf: f32) -> u32 {
    (v * sf).round() as u32
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("ENV_LOCK poisoned")
}

#[test]
fn vulkan_path_msaa_pipeline_is_visible_by_default() {
    let _lock = env_lock();
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };
    if ctx.adapter.get_info().backend != wgpu::Backend::Vulkan {
        return;
    }

    // If the opt-out env var is set (e.g. debugging a driver issue), skip to avoid a false
    // failure.
    if std::env::var_os("FRET_DISABLE_VULKAN_PATH_MSAA").is_some() {
        return;
    }

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let features = ctx.adapter.get_texture_format_features(format);
    if !features
        .allowed_usages
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        || !features
            .allowed_usages
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
        || !features
            .flags
            .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        || !features.flags.sample_count_supported(4)
    {
        return;
    }

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);
    renderer.set_path_msaa_samples(4);

    let viewport_size = (256u32, 256u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("vulkan_path_msaa_visibility_conformance output"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let cmds = [
        PathCommand::MoveTo(Point::new(Px(32.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(224.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(224.0), Px(224.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(224.0))),
        PathCommand::Close,
    ];
    let constraints = PathConstraints { scale_factor: 1.0 };
    let (path, _metrics) =
        renderer.prepare(&cmds, PathStyle::Fill(FillStyle::default()), constraints);

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(256.0), Px(256.0)),
        ),
        background: (Paint::TRANSPARENT).into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
        paint: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
    });

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &target_view,
            source: RenderSceneSourceSelection::flat_compat(&scene),
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

    let snap = renderer
        .take_last_frame_perf_snapshot()
        .expect("perf snapshot");
    assert_eq!(snap.path_msaa_samples_requested, 4);
    assert!(
        snap.pipeline_switches_path_msaa > 0,
        "expected Vulkan path MSAA pipeline to be enabled by default; got pipeline_switches_path_msaa=0"
    );
    assert_eq!(snap.path_msaa_samples_effective, 4);
    assert_eq!(snap.path_msaa_vulkan_safety_valve_degradations, 0);

    let pixels = read_texture_rgba8(&ctx.device, &ctx.queue, &target, viewport_size);
    let sample = pixel_rgba(&pixels, viewport_size.0, u(128.0, 1.0), u(128.0, 1.0));
    assert!(
        sample[3] > 200,
        "expected visible output alpha; got rgba={sample:?}"
    );
}

#[test]
fn vulkan_path_msaa_can_be_disabled_via_env() {
    let _lock = env_lock();
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };
    if ctx.adapter.get_info().backend != wgpu::Backend::Vulkan {
        return;
    }

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let features = ctx.adapter.get_texture_format_features(format);
    if !features
        .allowed_usages
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        || !features
            .allowed_usages
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
        || !features
            .flags
            .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        || !features.flags.sample_count_supported(4)
    {
        return;
    }

    let _env = EnvVarGuard::set("FRET_DISABLE_VULKAN_PATH_MSAA", "1");

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);
    renderer.set_path_msaa_samples(4);

    let viewport_size = (256u32, 256u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("vulkan_path_msaa_visibility_conformance output (forced)"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let cmds = [
        PathCommand::MoveTo(Point::new(Px(32.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(224.0), Px(32.0))),
        PathCommand::LineTo(Point::new(Px(224.0), Px(224.0))),
        PathCommand::LineTo(Point::new(Px(32.0), Px(224.0))),
        PathCommand::Close,
    ];
    let constraints = PathConstraints { scale_factor: 1.0 };
    let (path, _metrics) =
        renderer.prepare(&cmds, PathStyle::Fill(FillStyle::default()), constraints);

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(256.0), Px(256.0)),
        ),
        background: (Paint::TRANSPARENT).into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
        paint: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
    });

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &target_view,
            source: RenderSceneSourceSelection::flat_compat(&scene),
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

    let snap = renderer
        .take_last_frame_perf_snapshot()
        .expect("perf snapshot");
    assert!(
        snap.pipeline_switches_path_msaa == 0,
        "expected Vulkan path MSAA pipeline to be disabled via env; got pipeline_switches_path_msaa={}",
        snap.pipeline_switches_path_msaa
    );
    assert_eq!(snap.path_msaa_samples_effective, 1);
    assert!(
        snap.path_msaa_vulkan_safety_valve_degradations >= 1,
        "expected Vulkan MSAA opt-out to be observed; got path_msaa_vulkan_safety_valve_degradations={}",
        snap.path_msaa_vulkan_safety_valve_degradations
    );

    let pixels = read_texture_rgba8(&ctx.device, &ctx.queue, &target, viewport_size);
    let sample = pixel_rgba(&pixels, viewport_size.0, u(128.0, 1.0), u(128.0, 1.0));
    assert!(
        sample[3] > 200,
        "expected visible output alpha; got rgba={sample:?}"
    );
}
