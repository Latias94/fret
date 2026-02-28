use fret_core::PathService as _;
use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp};
use fret_core::{FillStyle, PathCommand, PathConstraints, PathStyle};
use fret_render_wgpu::{ClearColor, RenderSceneParams, Renderer, WgpuContext};
use std::ffi::OsString;
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};

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

fn read_texture_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> Vec<u8> {
    let (width, height) = size;
    let bytes_per_pixel: u32 = 4;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(256) * 256;
    let buffer_size = padded_bytes_per_row as u64 * height as u64;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vulkan_path_msaa_visibility_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("vulkan_path_msaa_visibility_conformance readback encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("map_async channel closed").unwrap();

    let mapped = slice.get_mapped_range();
    let mut pixels = vec![0u8; (unpadded_bytes_per_row * height) as usize];
    for row in 0..height as usize {
        let src = row * padded_bytes_per_row as usize;
        let dst = row * unpadded_bytes_per_row as usize;
        pixels[dst..dst + unpadded_bytes_per_row as usize]
            .copy_from_slice(&mapped[src..src + unpadded_bytes_per_row as usize]);
    }
    drop(mapped);
    buffer.unmap();
    pixels
}

fn pixel_rgba(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [
        pixels[idx],
        pixels[idx + 1],
        pixels[idx + 2],
        pixels[idx + 3],
    ]
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
        background: Paint::TRANSPARENT,
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
        paint: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
    });

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &target_view,
            scene: &scene,
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
        background: Paint::TRANSPARENT,
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Path {
        order: DrawOrder(1),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
        paint: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
    });

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &target_view,
            scene: &scene,
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
