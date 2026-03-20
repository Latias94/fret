use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use fret_render::{RenderError, SurfaceState, WgpuContext};
#[cfg(target_os = "macos")]
use objc2_metal::MTLDevice as _;
use serde_json::json;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

const WINDOW_TITLE: &str = "wgpu_hello_world_control";

#[derive(Debug, Clone, Copy)]
struct WindowConfig {
    width_px: f64,
    height_px: f64,
}

impl WindowConfig {
    fn from_env() -> Self {
        Self {
            width_px: env_f64("FRET_WGPU_HELLO_WORLD_CONTROL_WINDOW_WIDTH")
                .unwrap_or(500.0)
                .max(1.0),
            height_px: env_f64("FRET_WGPU_HELLO_WORLD_CONTROL_WINDOW_HEIGHT")
                .unwrap_or(500.0)
                .max(1.0),
        }
    }
}

#[derive(Debug, Clone)]
struct InternalGpuSamplingConfig {
    report_path: PathBuf,
    sample_at_secs: Vec<f64>,
    top_n: usize,
}

#[derive(Debug, Clone, Default)]
struct RuntimeSampleState {
    redraw_count: u64,
    present_count: u64,
    last_redraw_since_launch_ms: u64,
    last_present_since_launch_ms: u64,
    last_surface_width_px: u32,
    last_surface_height_px: u32,
}

struct GpuWindowState {
    window: Arc<dyn Window>,
    context: WgpuContext,
    surface: SurfaceState<'static>,
}

struct WgpuHelloWorldControlApp {
    window_cfg: WindowConfig,
    launch_at: Instant,
    launch_unix_ms: u64,
    exit_after: Option<Duration>,
    pre_init_sleep: Option<Duration>,
    continuous_redraw_interval: Option<Duration>,
    next_redraw_at: Option<Instant>,
    runtime_state: Arc<Mutex<RuntimeSampleState>>,
    gpu: Option<GpuWindowState>,
    sampling_started: bool,
}

impl WgpuHelloWorldControlApp {
    fn new(window_cfg: WindowConfig) -> Self {
        let continuous_redraw_interval = continuous_redraw_interval();
        Self {
            window_cfg,
            launch_at: Instant::now(),
            launch_unix_ms: process_launch_unix_ms(),
            exit_after: env_f64("FRET_WGPU_HELLO_WORLD_CONTROL_EXIT_AFTER_SECS")
                .filter(|value| *value >= 0.0)
                .map(Duration::from_secs_f64),
            pre_init_sleep: pre_init_sleep_duration(),
            continuous_redraw_interval,
            next_redraw_at: continuous_redraw_interval.map(|interval| Instant::now() + interval),
            runtime_state: Arc::new(Mutex::new(RuntimeSampleState::default())),
            gpu: None,
            sampling_started: false,
        }
    }

    fn init_gpu(&mut self, event_loop: &dyn ActiveEventLoop) -> anyhow::Result<()> {
        if self.gpu.is_some() {
            return Ok(());
        }

        let attrs = WindowAttributes::default()
            .with_title(WINDOW_TITLE)
            .with_surface_size(LogicalSize::new(
                self.window_cfg.width_px,
                self.window_cfg.height_px,
            ));
        let window = Arc::<dyn Window>::from(event_loop.create_window(attrs)?);
        let context = pollster::block_on(WgpuContext::new()).context("WgpuContext::new failed")?;
        let size = window.surface_size();
        let surface = context
            .instance
            .create_surface(window.clone())
            .map_err(|source| RenderError::CreateSurfaceFailed { source })?;
        let surface = SurfaceState::new(
            &context.adapter,
            &context.device,
            surface,
            size.width,
            size.height,
        )?;

        {
            let mut state = self
                .runtime_state
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            state.last_surface_width_px = surface.config.width;
            state.last_surface_height_px = surface.config.height;
        }

        if !self.sampling_started {
            if let Some(config) = internal_gpu_sampling_config() {
                start_internal_sampling_thread(
                    config,
                    self.launch_at,
                    self.launch_unix_ms,
                    &context,
                    &surface,
                    self.window_cfg,
                    self.runtime_state.clone(),
                );
            }
            self.sampling_started = true;
        }

        window.request_redraw();
        self.schedule_next_redraw();
        self.gpu = Some(GpuWindowState {
            window,
            context,
            surface,
        });
        Ok(())
    }

    fn schedule_next_redraw(&mut self) {
        self.next_redraw_at = self
            .continuous_redraw_interval
            .map(|interval| Instant::now() + interval);
    }

    fn request_continuous_redraw_if_due(&mut self) {
        let Some(deadline) = self.next_redraw_at else {
            return;
        };
        if Instant::now() < deadline {
            return;
        }
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
            self.schedule_next_redraw();
        }
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        if size.width == 0 || size.height == 0 {
            return;
        }
        gpu.surface
            .resize(&gpu.context.device, size.width, size.height);
        let mut state = self
            .runtime_state
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        state.last_surface_width_px = size.width;
        state.last_surface_height_px = size.height;
        gpu.window.request_redraw();
    }

    fn render(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };

        {
            let mut state = self
                .runtime_state
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            state.redraw_count = state.redraw_count.saturating_add(1);
            state.last_redraw_since_launch_ms = self.launch_at.elapsed().as_millis() as u64;
        }

        let result = gpu.surface.present_with(&gpu.context.queue, |view| {
            let mut encoder =
                gpu.context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("wgpu_hello_world_control.clear"),
                    });
            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("wgpu_hello_world_control.pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.08,
                                g: 0.08,
                                b: 0.09,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }
            vec![encoder.finish()]
        });

        match result {
            Ok(()) => {
                let mut state = self
                    .runtime_state
                    .lock()
                    .unwrap_or_else(|err| err.into_inner());
                state.present_count = state.present_count.saturating_add(1);
                state.last_present_since_launch_ms = self.launch_at.elapsed().as_millis() as u64;
                drop(state);
                self.schedule_next_redraw();
            }
            Err(RenderError::SurfaceAcquireFailed { source }) => match source {
                fret_render::SurfaceAcquireError::Lost
                | fret_render::SurfaceAcquireError::Outdated => {
                    let size = gpu.window.surface_size();
                    self.resize(size);
                }
                fret_render::SurfaceAcquireError::OutOfMemory => event_loop.exit(),
                fret_render::SurfaceAcquireError::Timeout
                | fret_render::SurfaceAcquireError::Other => {
                    eprintln!("{WINDOW_TITLE}: surface acquire failed: {source:?}");
                }
            },
            Err(err) => {
                eprintln!("{WINDOW_TITLE}: render failed: {err:?}");
                event_loop.exit();
            }
        }
    }
}

impl ApplicationHandler for WgpuHelloWorldControlApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
        if let Some(pre_init_sleep) = self.pre_init_sleep.take() {
            eprintln!(
                "{WINDOW_TITLE}: sleeping {:.3}s before GPU init for diagnostics",
                pre_init_sleep.as_secs_f64()
            );
            std::thread::sleep(pre_init_sleep);
        }
        if let Err(err) = self.init_gpu(event_loop) {
            eprintln!("{WINDOW_TITLE}: init failed: {err:#}");
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        if gpu.window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::SurfaceResized(size) => self.resize(size),
            WindowEvent::RedrawRequested => self.render(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.request_continuous_redraw_if_due();
        if let Some(exit_after) = self.exit_after {
            if self.launch_at.elapsed() >= exit_after {
                event_loop.exit();
                return;
            }
        }
        if let Some(deadline) = self.next_redraw_at {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

fn env_string(name: &str) -> Option<String> {
    std::env::var_os(name)
        .and_then(|value| value.into_string().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_f64(name: &str) -> Option<f64> {
    env_string(name).and_then(|value| value.parse::<f64>().ok())
}

fn env_usize(name: &str) -> Option<usize> {
    env_string(name).and_then(|value| value.parse::<usize>().ok())
}

fn parse_sample_at_secs(raw: &str) -> anyhow::Result<Vec<f64>> {
    let mut out = Vec::new();
    for piece in raw.split(',') {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        let value = piece
            .parse::<f64>()
            .map_err(|err| anyhow::anyhow!("invalid sample offset `{piece}`: {err}"))?;
        if value < 0.0 {
            anyhow::bail!("sample offset must be >= 0, got {value}");
        }
        out.push(value);
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    out.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON);
    if out.is_empty() {
        anyhow::bail!("no sample offsets configured");
    }
    Ok(out)
}

fn pre_init_sleep_duration() -> Option<Duration> {
    env_f64("FRET_WGPU_HELLO_WORLD_CONTROL_PRE_INIT_SLEEP_SECS")
        .filter(|value| *value > 0.0)
        .map(Duration::from_secs_f64)
}

fn continuous_redraw_interval() -> Option<Duration> {
    if let Some(interval_ms) =
        env_f64("FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW_INTERVAL_MS")
    {
        if interval_ms <= 0.0 {
            return None;
        }
        return Some(Duration::from_secs_f64(interval_ms / 1000.0));
    }
    match env_string("FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW")
        .as_deref()
        .map(str::to_ascii_lowercase)
    {
        Some(value) if matches!(value.as_str(), "1" | "true" | "yes" | "on") => {
            Some(Duration::from_millis(16))
        }
        _ => None,
    }
}

fn internal_gpu_sampling_config() -> Option<InternalGpuSamplingConfig> {
    let report_path = env_string("FRET_WGPU_HELLO_WORLD_CONTROL_INTERNAL_REPORT_PATH")?;
    let sample_at_secs_raw = env_string("FRET_WGPU_HELLO_WORLD_CONTROL_INTERNAL_SAMPLE_AT_SECS")
        .unwrap_or_else(|| "2,6,12".to_string());
    let sample_at_secs = match parse_sample_at_secs(&sample_at_secs_raw) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{WINDOW_TITLE}: invalid internal sample config: {err:#}");
            return None;
        }
    };
    Some(InternalGpuSamplingConfig {
        report_path: PathBuf::from(report_path),
        sample_at_secs,
        top_n: env_usize("FRET_WGPU_HELLO_WORLD_CONTROL_INTERNAL_TOP_N").unwrap_or(16),
    })
}

fn start_internal_sampling_thread(
    config: InternalGpuSamplingConfig,
    launch_at: Instant,
    launch_unix_ms: u64,
    context: &WgpuContext,
    surface: &SurfaceState<'static>,
    window_cfg: WindowConfig,
    runtime_state: Arc<Mutex<RuntimeSampleState>>,
) {
    let instance = context.instance.clone();
    let device = context.device.clone();
    let adapter_info = context.adapter.get_info();
    let init_diagnostics = context.init_diagnostics.clone();
    let requested_runtime = json!({
        "window": {
            "width_px": window_cfg.width_px,
            "height_px": window_cfg.height_px,
        },
        "surface": {
            "format": format!("{:?}", surface.config.format),
            "present_mode": format!("{:?}", surface.config.present_mode),
            "desired_maximum_frame_latency": surface.config.desired_maximum_frame_latency,
            "alpha_mode": format!("{:?}", surface.config.alpha_mode),
        },
        "pre_init_sleep_secs": pre_init_sleep_duration()
            .map(|duration| duration.as_secs_f64()),
        "continuous_redraw_interval_ms": continuous_redraw_interval()
            .map(|interval| interval.as_secs_f64() * 1000.0),
    });
    std::thread::spawn(move || {
        let mut samples = Vec::new();
        for &offset_secs in &config.sample_at_secs {
            let deadline = launch_at + Duration::from_secs_f64(offset_secs);
            let now = Instant::now();
            if deadline > now {
                std::thread::sleep(deadline.duration_since(now));
            }
            let sample = capture_internal_gpu_sample(
                &instance,
                &device,
                launch_unix_ms,
                offset_secs,
                config.top_n,
                &runtime_state,
            );
            samples.push(sample);
            let payload = json!({
                "schema_version": 1,
                "kind": "wgpu_hello_world_control_internal_gpu_timeline",
                "process_launch_unix_ms": launch_unix_ms,
                "sample_at_secs": config.sample_at_secs,
                "adapter": {
                    "name": adapter_info.name,
                    "vendor": adapter_info.vendor,
                    "device": adapter_info.device,
                    "device_type": format!("{:?}", adapter_info.device_type),
                    "driver": adapter_info.driver,
                    "driver_info": adapter_info.driver_info,
                    "backend": format!("{:?}", adapter_info.backend),
                },
                "init_diagnostics": init_diagnostics,
                "requested_runtime": requested_runtime,
                "samples": samples,
            });
            write_internal_report(&config.report_path, &payload);
        }
    });
}

fn capture_internal_gpu_sample(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
    launch_unix_ms: u64,
    offset_secs: f64,
    top_n: usize,
    runtime_state: &Arc<Mutex<RuntimeSampleState>>,
) -> serde_json::Value {
    let captured_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let hub_json = if let Some(report) = instance.generate_report() {
        let hub = report.hub_report();
        json!({
            "present": true,
            "adapters": (hub.adapters.num_allocated + hub.adapters.num_kept_from_user) as u64,
            "devices": (hub.devices.num_allocated + hub.devices.num_kept_from_user) as u64,
            "queues": (hub.queues.num_allocated + hub.queues.num_kept_from_user) as u64,
            "command_encoders": (hub.command_encoders.num_allocated + hub.command_encoders.num_kept_from_user) as u64,
            "buffers": (hub.buffers.num_allocated + hub.buffers.num_kept_from_user) as u64,
            "textures": (hub.textures.num_allocated + hub.textures.num_kept_from_user) as u64,
            "texture_views": (hub.texture_views.num_allocated + hub.texture_views.num_kept_from_user) as u64,
            "samplers": (hub.samplers.num_allocated + hub.samplers.num_kept_from_user) as u64,
            "shader_modules": (hub.shader_modules.num_allocated + hub.shader_modules.num_kept_from_user) as u64,
            "render_pipelines": (hub.render_pipelines.num_allocated + hub.render_pipelines.num_kept_from_user) as u64,
            "compute_pipelines": (hub.compute_pipelines.num_allocated + hub.compute_pipelines.num_kept_from_user) as u64,
        })
    } else {
        json!({ "present": false })
    };

    let allocator_report = device.generate_allocator_report();
    let metal_current_allocated_size_bytes = current_metal_allocated_size_bytes(device);
    let allocator_json = if let Some(report) = allocator_report {
        let allocation_count = report.allocations.len() as u64;
        let mut allocations = report.allocations;
        allocations.sort_unstable_by_key(|allocation| std::cmp::Reverse(allocation.size));
        allocations.truncate(top_n);
        let top_allocations = allocations
            .into_iter()
            .map(|allocation| {
                json!({
                    "name": allocation.name,
                    "size": allocation.size,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "present": true,
            "total_allocated_bytes": report.total_allocated_bytes,
            "total_reserved_bytes": report.total_reserved_bytes,
            "blocks": report.blocks.len() as u64,
            "allocations": allocation_count,
            "metal_current_allocated_size_bytes": metal_current_allocated_size_bytes,
            "top_allocations": top_allocations,
        })
    } else {
        json!({
            "present": false,
            "metal_current_allocated_size_bytes": metal_current_allocated_size_bytes,
            "top_allocations": [],
        })
    };

    let runtime = runtime_state
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone();

    json!({
        "offset_secs": offset_secs,
        "captured_unix_ms": captured_unix_ms,
        "captured_since_launch_ms": captured_unix_ms.saturating_sub(launch_unix_ms),
        "hub": hub_json,
        "allocator": allocator_json,
        "runtime": {
            "redraw_count": runtime.redraw_count,
            "present_count": runtime.present_count,
            "last_redraw_since_launch_ms": runtime.last_redraw_since_launch_ms,
            "last_present_since_launch_ms": runtime.last_present_since_launch_ms,
            "last_surface_width_px": runtime.last_surface_width_px,
            "last_surface_height_px": runtime.last_surface_height_px,
        }
    })
}

#[cfg(target_os = "macos")]
fn current_metal_allocated_size_bytes(device: &wgpu::Device) -> Option<u64> {
    unsafe {
        device
            .as_hal::<wgpu::hal::api::Metal>()
            .map(|device| device.raw_device().currentAllocatedSize() as u64)
    }
}

#[cfg(not(target_os = "macos"))]
fn current_metal_allocated_size_bytes(_device: &wgpu::Device) -> Option<u64> {
    None
}

fn write_internal_report(path: &Path, payload: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_vec_pretty(payload) {
        Ok(bytes) => {
            if let Err(err) = std::fs::write(path, bytes) {
                eprintln!(
                    "{WINDOW_TITLE}: failed to write internal report {}: {err}",
                    path.display()
                );
            }
        }
        Err(err) => {
            eprintln!("{WINDOW_TITLE}: failed to serialize internal report: {err}");
        }
    }
}

fn process_launch_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn main() -> anyhow::Result<()> {
    let _ = fret_alloc::allocator_name();
    let event_loop = EventLoop::new()?;
    let app = WgpuHelloWorldControlApp::new(WindowConfig::from_env());
    event_loop.run_app(app)?;
    Ok(())
}
