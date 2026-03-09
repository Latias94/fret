use gpui::{
    App, Application, Bounds, Context, Timer, Window, WindowBounds, WindowOptions, div, px, rgb,
    size,
};
use gpui::prelude::*;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareActiveMode {
    Idle,
    RerenderOnly,
    PaintModel,
    LayoutModel,
}

impl CompareActiveMode {
    fn from_env() -> Self {
        let Some(raw) = env_string("GPUI_HELLO_WORLD_COMPARE_ACTIVE_MODE") else {
            return Self::Idle;
        };
        match raw.trim().to_ascii_lowercase().as_str() {
            "idle" => Self::Idle,
            "rerender" | "rerender-only" | "rerender_only" | "render-only" | "render_only" => {
                Self::RerenderOnly
            }
            "paint" | "paint-model" | "paint_model" => Self::PaintModel,
            "layout" | "layout-model" | "layout_model" => Self::LayoutModel,
            other => {
                eprintln!(
                    "gpui_hello_world_compare active_mode_invalid value={other} default=idle"
                );
                Self::Idle
            }
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::RerenderOnly => "rerender-only",
            Self::PaintModel => "paint-model",
            Self::LayoutModel => "layout-model",
        }
    }

    fn is_active(self) -> bool {
        !matches!(self, Self::Idle)
    }

    fn uses_animation_frame_loop(self) -> bool {
        matches!(self, Self::RerenderOnly | Self::PaintModel | Self::LayoutModel)
    }
}

#[derive(Debug, Clone, Copy)]
struct CompareFlags {
    no_text: bool,
    no_swatches: bool,
    active_mode: CompareActiveMode,
}

impl CompareFlags {
    fn from_env() -> Self {
        Self {
            no_text: env_flag("GPUI_HELLO_WORLD_COMPARE_NO_TEXT"),
            no_swatches: env_flag("GPUI_HELLO_WORLD_COMPARE_NO_SWATCHES"),
            active_mode: CompareActiveMode::from_env(),
        }
    }
}

#[derive(Debug, Clone)]
struct CompareConfig {
    flags: CompareFlags,
    width_px: f32,
    height_px: f32,
    sample_offsets: Vec<f64>,
    internal_report_path: Option<PathBuf>,
    exit_after_secs: Option<f64>,
}

impl CompareConfig {
    fn from_env() -> Self {
        Self {
            flags: CompareFlags::from_env(),
            width_px: env_f32("GPUI_HELLO_WORLD_COMPARE_WINDOW_WIDTH").unwrap_or(500.0).max(1.0),
            height_px: env_f32("GPUI_HELLO_WORLD_COMPARE_WINDOW_HEIGHT").unwrap_or(500.0).max(1.0),
            sample_offsets: parse_sample_offsets(
                &env_string("GPUI_HELLO_WORLD_COMPARE_INTERNAL_SAMPLE_AT_SECS")
                    .unwrap_or_else(|| "1,2,6".to_string()),
            ),
            internal_report_path: env_string("GPUI_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH")
                .map(PathBuf::from),
            exit_after_secs: env_f64("GPUI_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS"),
        }
    }
}

#[derive(Debug)]
struct ProbeState {
    launch_at: Instant,
    launch_unix_ms: u64,
    render_count: u64,
    frame_tick: u64,
    samples: Vec<Value>,
}

impl ProbeState {
    fn new() -> Self {
        Self {
            launch_at: Instant::now(),
            launch_unix_ms: unix_ms_now(),
            render_count: 0,
            frame_tick: 0,
            samples: Vec::new(),
        }
    }
}

struct GpuiHelloWorldCompare {
    flags: CompareFlags,
    render_count: u64,
    frame_tick: u64,
    probe: Arc<Mutex<ProbeState>>,
}

impl GpuiHelloWorldCompare {
    fn new(flags: CompareFlags, probe: Arc<Mutex<ProbeState>>) -> Self {
        Self {
            flags,
            render_count: 0,
            frame_tick: 0,
            probe,
        }
    }
}

impl Render for GpuiHelloWorldCompare {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_count = self.render_count.saturating_add(1);
        if self.flags.active_mode.uses_animation_frame_loop() {
            window.request_animation_frame();
            self.frame_tick = self.frame_tick.wrapping_add(1);
        }

        {
            let mut probe = self.probe.lock().unwrap_or_else(|err| err.into_inner());
            probe.render_count = self.render_count;
            probe.frame_tick = self.frame_tick;
        }

        let bg_wave = if matches!(self.flags.active_mode, CompareActiveMode::PaintModel) {
            ((self.frame_tick / 2) % 24) as u32
        } else {
            0
        };
        let panel_bg = ((0x50 + bg_wave) << 16) | ((0x50 + (bg_wave / 2)) << 8) | (0x50 + bg_wave);
        let layout_probe_height_px = if matches!(self.flags.active_mode, CompareActiveMode::LayoutModel) {
            8.0 + (((self.frame_tick / 2) % 5) as f32 * 12.0)
        } else {
            8.0
        };

        let swatch = |fill_rgb: u32, border_rgb: u32| {
            div()
                .size_8()
                .bg(rgb(fill_rgb))
                .border_1()
                .border_dashed()
                .rounded_md()
                .border_color(rgb(border_rgb))
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .bg(rgb(panel_bg))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .w(px(160.0))
                    .h(px(layout_probe_height_px))
                    .bg(rgb(panel_bg))
                    .rounded_md(),
            )
            .when(!self.flags.no_text, |div| div.child("Hello, World!"))
            .when(!self.flags.no_swatches, |root| {
                root.child(
                    div()
                        .flex()
                        .gap_2()
                        .child(swatch(0xff0000, 0xffffff))
                        .child(swatch(0x00ff00, 0xffffff))
                        .child(swatch(0x0000ff, 0xffffff))
                        .child(swatch(0xffff00, 0xffffff))
                        .child(swatch(0x000000, 0xffffff))
                        .child(swatch(0xffffff, 0x000000)),
                )
            })
    }
}

fn install_probe_tasks(
    window: &mut Window,
    cx: &mut App,
    config: CompareConfig,
    probe: Arc<Mutex<ProbeState>>,
) {
    if let Some(report_path) = config.internal_report_path.clone() {
        write_report(&report_path, &config, &probe);

        let sample_offsets = config.sample_offsets.clone();
        let sample_report_path = report_path.clone();
        let sample_config = config.clone();
        let sample_probe = probe.clone();
        window
            .spawn(cx, async move |_cx| {
                for offset_secs in sample_offsets {
                    let wait_secs = {
                        let state = sample_probe.lock().unwrap_or_else(|err| err.into_inner());
                        (offset_secs - state.launch_at.elapsed().as_secs_f64()).max(0.0)
                    };
                    Timer::after(Duration::from_secs_f64(wait_secs)).await;
                    capture_sample(offset_secs, &sample_config, &sample_probe);
                    write_report(&sample_report_path, &sample_config, &sample_probe);
                }
            })
            .detach();
    }

    if let Some(exit_after_secs) = config.exit_after_secs {
        let exit_report_path = config.internal_report_path.clone();
        let exit_config = config.clone();
        let exit_probe = probe.clone();
        window
            .spawn(cx, async move |cx| {
                let wait_secs = {
                    let state = exit_probe.lock().unwrap_or_else(|err| err.into_inner());
                    (exit_after_secs.max(0.0) - state.launch_at.elapsed().as_secs_f64()).max(0.0)
                };
                Timer::after(Duration::from_secs_f64(wait_secs)).await;
                if let Some(report_path) = exit_report_path.as_ref() {
                    write_report(report_path, &exit_config, &exit_probe);
                }
                let _ = cx.update(|_window, cx| {
                    cx.quit();
                });
            })
            .detach();
    }
}

fn capture_sample(offset_secs: f64, config: &CompareConfig, probe: &Arc<Mutex<ProbeState>>) {
    let mut state = probe.lock().unwrap_or_else(|err| err.into_inner());
    let captured_unix_ms = unix_ms_now();
    let launch_unix_ms = state.launch_unix_ms;
    let render_count = state.render_count;
    let frame_tick = state.frame_tick;
    state.samples.push(json!({
        "offset_secs": offset_secs,
        "captured_unix_ms": captured_unix_ms,
        "captured_since_launch_ms": captured_unix_ms.saturating_sub(launch_unix_ms),
        "runtime": {
            "render_count": render_count,
            "frame_tick": frame_tick,
        },
        "scene": {
            "no_text": config.flags.no_text,
            "no_swatches": config.flags.no_swatches,
            "active_mode": config.flags.active_mode.as_str(),
        },
    }));
}

fn write_report(path: &Path, config: &CompareConfig, probe: &Arc<Mutex<ProbeState>>) {
    let state = probe.lock().unwrap_or_else(|err| err.into_inner());
    let report = json!({
        "schema_version": 1,
        "framework": "gpui",
        "requested_runtime": {
            "framework": "gpui",
            "scene": {
                "no_text": config.flags.no_text,
                "no_swatches": config.flags.no_swatches,
                "active_mode": config.flags.active_mode.as_str(),
                "active": config.flags.active_mode.is_active(),
                "animation_frame_loop": config.flags.active_mode.uses_animation_frame_loop(),
            },
            "window": {
                "width_px": config.width_px,
                "height_px": config.height_px,
            },
            "startup": {
                "exit_after_secs": config.exit_after_secs,
                "sample_at_secs": config.sample_offsets,
            },
        },
        "runtime": {
            "render_count": state.render_count,
            "frame_tick": state.frame_tick,
            "elapsed_since_launch_ms": state.launch_at.elapsed().as_millis() as u64,
        },
        "samples": state.samples,
    });
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap_or_else(|_| b"{}".to_vec()));
}

fn parse_sample_offsets(raw: &str) -> Vec<f64> {
    let mut offsets = raw
        .split(',')
        .filter_map(|piece| {
            let piece = piece.trim();
            if piece.is_empty() {
                return None;
            }
            piece.parse::<f64>().ok().filter(|value| *value >= 0.0)
        })
        .collect::<Vec<_>>();
    offsets.sort_by(f64::total_cmp);
    offsets.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON);
    offsets
}

fn env_flag(name: &str) -> bool {
    std::env::var_os(name)
        .and_then(|value| value.into_string().ok())
        .is_some_and(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            !normalized.is_empty() && normalized != "0" && normalized != "false" && normalized != "no"
        })
}

fn env_string(name: &str) -> Option<String> {
    std::env::var_os(name).and_then(|value| value.into_string().ok())
}

fn env_f32(name: &str) -> Option<f32> {
    env_string(name).and_then(|value| value.parse::<f32>().ok())
}

fn env_f64(name: &str) -> Option<f64> {
    env_string(name).and_then(|value| value.parse::<f64>().ok())
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn main() {
    let config = CompareConfig::from_env();
    let probe = Arc::new(Mutex::new(ProbeState::new()));

    Application::new().run({
        let config = config.clone();
        let probe = probe.clone();
        move |cx: &mut App| {
            let window_bounds = WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(config.width_px), px(config.height_px)),
                cx,
            ));
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(window_bounds),
                    ..Default::default()
                },
                {
                    let config = config.clone();
                    let probe = probe.clone();
                    move |window, cx| {
                        install_probe_tasks(window, cx, config.clone(), probe.clone());
                        cx.new(|_| GpuiHelloWorldCompare::new(config.flags, probe.clone()))
                    }
                },
            )
            .unwrap();
            cx.activate(true);
        }
    });
}
