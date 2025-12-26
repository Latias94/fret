use fret_app::{App, Effect, WindowRequest};
use fret_core::{
    AppWindowId, Color, Event, KeyCode, PlatformCapabilities, Point, Px, Rect, Scene, Size,
    SvgFit, UiServices,
};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use std::time::{Duration, Instant};
use winit::event_loop::EventLoop;

const SVG_SQUARE: &str = r#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <path d="M12 2l3 7 7 3-7 3-3 7-3-7-7-3 7-3 3-7z"/>
</svg>
"#;

const SVG_WIDE: &str = r#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 16">
  <path d="M2 8c6-8 22-8 28 0-6 8-22 8-28 0z"/>
</svg>
"#;

#[derive(Debug, Clone, Copy)]
struct Rgb {
    r: f32,
    g: f32,
    b: f32,
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Rgb {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i as u32 {
        0 => Rgb { r: v, g: t, b: p },
        1 => Rgb { r: q, g: v, b: p },
        2 => Rgb { r: p, g: v, b: t },
        3 => Rgb { r: p, g: q, b: v },
        4 => Rgb { r: t, g: p, b: v },
        _ => Rgb { r: v, g: p, b: q },
    }
}

#[derive(Debug, Clone, Copy)]
enum FitMode {
    Contain,
    Width,
    Stretch,
    Mixed,
}

impl FitMode {
    fn next(self) -> Self {
        match self {
            Self::Contain => Self::Width,
            Self::Width => Self::Stretch,
            Self::Stretch => Self::Mixed,
            Self::Mixed => Self::Contain,
        }
    }

    fn to_fit(self, idx: usize) -> SvgFit {
        match self {
            Self::Contain => SvgFit::Contain,
            Self::Width => SvgFit::Width,
            Self::Stretch => SvgFit::Stretch,
            Self::Mixed => match idx % 3 {
                0 => SvgFit::Contain,
                1 => SvgFit::Width,
                _ => SvgFit::Stretch,
            },
        }
    }
}

#[derive(Debug)]
struct SvgAtlasStressState {
    svg_square: Option<fret_core::SvgId>,
    svg_wide: Option<fret_core::SvgId>,
    frame: u64,
    start: Option<Instant>,
    last_report: Option<Instant>,
    render_time_accum: Duration,
    render_frames_accum: u64,
    phase: bool,
    auto_phase: bool,
    fit_mode: FitMode,
    budget_presets: Vec<u64>,
    budget_index: usize,
    budget_applied: u64,
    max_frames: Option<u64>,
}

impl Default for SvgAtlasStressState {
    fn default() -> Self {
        Self {
            svg_square: None,
            svg_wide: None,
            frame: 0,
            start: None,
            last_report: None,
            render_time_accum: Duration::ZERO,
            render_frames_accum: 0,
            phase: false,
            auto_phase: true,
            fit_mode: FitMode::Mixed,
            budget_presets: vec![256 * 1024, 1024 * 1024, 4 * 1024 * 1024, 64 * 1024 * 1024],
            budget_index: 1,
            budget_applied: 0,
            max_frames: None,
        }
    }
}

#[derive(Default)]
struct SvgAtlasStressDriver {
    max_frames: Option<u64>,
}

impl SvgAtlasStressDriver {
    fn print_help() {
        println!("svg_atlas_stress controls:");
        println!("  Space: toggle phase (A/B)");
        println!("  T: toggle auto phase flip");
        println!("  F: cycle SvgFit mode");
        println!("  B: cycle svg_raster_budget_bytes preset");
        println!("  H: print this help");
    }

    fn print_state(state: &SvgAtlasStressState) {
        let budget = state.budget_presets[state.budget_index];
        println!(
            "phase={} auto_phase={} fit={:?} svg_raster_budget_bytes={}KB",
            if state.phase { "B" } else { "A" },
            state.auto_phase,
            state.fit_mode,
            budget / 1024
        );
    }
}

impl WinitDriver for SvgAtlasStressDriver {
    type WindowState = SvgAtlasStressState;

    fn create_window_state(&mut self, _app: &mut App, _window: AppWindowId) -> Self::WindowState {
        Self::print_help();
        let mut st = SvgAtlasStressState::default();
        st.max_frames = self.max_frames;
        Self::print_state(&st);
        st
    }

    fn gpu_frame_prepare(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        _context: &fret_render::WgpuContext,
        renderer: &mut fret_render::Renderer,
        _scale_factor: f32,
    ) {
        let desired = state.budget_presets[state.budget_index];
        if state.budget_applied != desired {
            renderer.set_svg_raster_budget_bytes(desired);
            state.budget_applied = desired;
        }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        _services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
        event: &Event,
    ) {
        let Event::KeyDown { key, repeat, .. } = event else {
            return;
        };
        if *repeat {
            return;
        }

        match *key {
            KeyCode::Space => {
                state.phase = !state.phase;
                Self::print_state(state);
            }
            KeyCode::KeyT => {
                state.auto_phase = !state.auto_phase;
                Self::print_state(state);
            }
            KeyCode::KeyF => {
                state.fit_mode = state.fit_mode.next();
                Self::print_state(state);
            }
            KeyCode::KeyB => {
                state.budget_index = (state.budget_index + 1) % state.budget_presets.len();
                Self::print_state(state);
            }
            KeyCode::KeyH => Self::print_help(),
            _ => {}
        }

        app.request_redraw(window);
    }

    fn render(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        _scale_factor: f32,
        services: &mut dyn fret_core::UiServices,
        scene: &mut Scene,
    ) {
        let render_start = Instant::now();

        if state.start.is_none() {
            let now = Instant::now();
            state.start = Some(now);
            state.last_report = Some(now);
        }

        state.frame = state.frame.wrapping_add(1);
        if state.auto_phase && state.frame % 180 == 0 {
            state.phase = !state.phase;
        }

        if state.svg_square.is_none() {
            state.svg_square = Some(services.svg().register_svg(SVG_SQUARE.as_bytes()));
        }
        if state.svg_wide.is_none() {
            state.svg_wide = Some(services.svg().register_svg(SVG_WIDE.as_bytes()));
        }
        let svg_square = state.svg_square.expect("svg registered");
        let svg_wide = state.svg_wide.expect("svg registered");

        scene.clear();

        scene.push(fret_core::SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect: bounds,
            background: Color {
                r: 0.06,
                g: 0.07,
                b: 0.08,
                a: 1.0,
            },
            border: fret_core::Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let margin = Px(12.0);
        let cell = Px(40.0);
        let gap = Px(2.0);
        let start = Point::new(bounds.origin.x + margin, bounds.origin.y + margin);
        let usable_w = (bounds.size.width.0 - margin.0 * 2.0).max(0.0);
        let usable_h = (bounds.size.height.0 - margin.0 * 2.0).max(0.0);

        let step = cell.0 + gap.0;
        let cols = (usable_w / step).floor().max(1.0) as usize;
        let rows = (usable_h / step).floor().max(1.0) as usize;

        let mut icons_emitted = 0usize;
        for row in 0..rows {
            for col in 0..cols {
                let parity = (row + col) % 2 == 0;
                if parity != state.phase {
                    continue;
                }

                let idx = row * cols + col;
                let w = 10.0 + (idx % 31) as f32;
                let h = 10.0 + ((idx / 31) % 31) as f32;
                let size = Size::new(Px(w), Px(h));

                let cell_origin = Point::new(
                    start.x + Px(col as f32 * step),
                    start.y + Px(row as f32 * step),
                );
                let x = cell_origin.x + Px((cell.0 - size.width.0).max(0.0) * 0.5);
                let y = cell_origin.y + Px((cell.0 - size.height.0).max(0.0) * 0.5);
                let rect = Rect::new(Point::new(x, y), size);

                let fit = state.fit_mode.to_fit(idx);
                let svg = if idx % 5 == 0 { svg_wide } else { svg_square };

                let rgb = hsv_to_rgb((idx as f32 * 0.031).rem_euclid(1.0), 0.75, 0.95);
                let color = Color {
                    r: rgb.r,
                    g: rgb.g,
                    b: rgb.b,
                    a: 1.0,
                };

                scene.push(fret_core::SceneOp::SvgMaskIcon {
                    order: fret_core::DrawOrder(1),
                    rect,
                    svg,
                    fit,
                    color,
                    opacity: 1.0,
                });
                icons_emitted += 1;
            }
        }

        let elapsed = render_start.elapsed();
        state.render_time_accum += elapsed;
        state.render_frames_accum = state.render_frames_accum.saturating_add(1);

        let report_every = Duration::from_secs(1);
        if let Some(last) = state.last_report
            && last.elapsed() >= report_every
        {
            let avg_us = if state.render_frames_accum == 0 {
                0.0
            } else {
                state.render_time_accum.as_secs_f64() * 1_000_000.0
                    / state.render_frames_accum as f64
            };
            println!(
                "frames={} icons/frame={} avg_driver_render={:.1}us budget={}KB fit={:?} phase={}",
                state.frame,
                icons_emitted,
                avg_us,
                state.budget_presets[state.budget_index] / 1024,
                state.fit_mode,
                if state.phase { "B" } else { "A" }
            );
            state.last_report = Some(Instant::now());
            state.render_time_accum = Duration::ZERO;
            state.render_frames_accum = 0;
        }

        if let Some(max) = state.max_frames
            && state.frame >= max
        {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        app.request_redraw(window);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
}

fn main() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_runner_winit_wgpu=info".parse().unwrap()),
        )
        .try_init();

    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());

    let config = WinitRunnerConfig {
        main_window_title: "fret-svg-atlas-stress".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        svg_raster_budget_bytes: 1024 * 1024,
        ..Default::default()
    };

    let mut max_frames: Option<u64> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--frames requires a value");
                };
                max_frames = Some(value.parse()?);
            }
            "--help" | "-h" => {
                println!("Usage: fret-svg-atlas-stress [--frames N]");
                return Ok(());
            }
            other => anyhow::bail!("unknown arg: {other}"),
        }
    }

    let driver = SvgAtlasStressDriver { max_frames };
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
