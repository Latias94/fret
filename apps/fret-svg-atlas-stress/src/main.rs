use fret_app::{App, Effect, WindowRequest};
use fret_core::SvgService as _;
use fret_core::{AppWindowId, Color, Event, KeyCode, Point, Px, Rect, Scene, Size, SvgFit};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    run_app_with_event_loop,
};
use fret_render::{ClearColor, RenderSceneParams, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use std::time::{Duration, Instant};
use winit::event_loop::EventLoop;

fn try_println(args: std::fmt::Arguments<'_>) {
    use std::io::Write as _;
    let mut out = std::io::stdout().lock();
    let _ = out.write_fmt(args);
    let _ = out.write_all(b"\n");
}

macro_rules! try_println {
    ($($tt:tt)*) => {
        try_println(format_args!($($tt)*))
    };
}

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
    last_renderer_report: Option<Instant>,
    clear_requested: bool,
    clear_atlas_requested: bool,
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
            last_renderer_report: None,
            clear_requested: false,
            clear_atlas_requested: false,
        }
    }
}

#[derive(Default)]
struct SvgAtlasStressDriver {
    max_frames: Option<u64>,
}

impl SvgAtlasStressDriver {
    fn print_help() {
        try_println!("svg_atlas_stress controls:");
        try_println!("  Space: toggle phase (A/B)");
        try_println!("  T: toggle auto phase flip");
        try_println!("  F: cycle SvgFit mode");
        try_println!("  B: cycle svg_raster_budget_bytes preset (standalone rasters only)");
        try_println!("  C: clear svg raster cache");
        try_println!("  M: clear svg mask atlas cache");
        try_println!("  H: print this help");
    }

    fn print_state(state: &SvgAtlasStressState) {
        let budget = state.budget_presets[state.budget_index];
        try_println!(
            "phase={} auto_phase={} fit={:?} svg_raster_budget_bytes={}KB (standalone only)",
            if state.phase { "B" } else { "A" },
            state.auto_phase,
            state.fit_mode,
            budget / 1024
        );
    }
}

fn record_scene(
    scene: &mut Scene,
    bounds: Rect,
    phase: bool,
    fit_mode: FitMode,
    svg_square: fret_core::SvgId,
    svg_wide: fret_core::SvgId,
) -> usize {
    scene.clear();

    scene.push(fret_core::SceneOp::Quad {
        order: fret_core::DrawOrder(0),
        rect: bounds,
        background: fret_core::Paint::Solid(Color {
            r: 0.06,
            g: 0.07,
            b: 0.08,
            a: 1.0,
        }),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT,
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
            if parity != phase {
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

            let fit = fit_mode.to_fit(idx);
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

    icons_emitted
}

fn run_headless(
    frames: u64,
    budget_bytes: u64,
    wait_gpu: bool,
    wait_every: u64,
    churn: bool,
    churn_every: u64,
    churn_drop: usize,
) -> anyhow::Result<()> {
    let ctx = pollster::block_on(WgpuContext::new())?;
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_svg_raster_budget_bytes(budget_bytes);
    renderer.set_svg_perf_enabled(true);
    renderer.set_perf_enabled(true);

    let svg_square = renderer.register_svg(SVG_SQUARE.as_bytes());
    let svg_wide = renderer.register_svg(SVG_WIDE.as_bytes());

    #[derive(Clone, Copy)]
    struct SvgVariant {
        square: fret_core::SvgId,
        wide: fret_core::SvgId,
    }

    fn register_variant(renderer: &mut Renderer, tag: u32) -> SvgVariant {
        let mut square = String::from(SVG_SQUARE);
        square.push_str(&format!("<!--variant:{tag}-->"));
        let mut wide = String::from(SVG_WIDE);
        wide.push_str(&format!("<!--variant:{tag}-->"));
        let square = renderer.register_svg(square.as_bytes());
        let wide = renderer.register_svg(wide.as_bytes());
        SvgVariant { square, wide }
    }

    fn record_scene_churn(
        scene: &mut Scene,
        bounds: Rect,
        phase: bool,
        fit_mode: FitMode,
        variants: &[SvgVariant],
    ) -> usize {
        if variants.is_empty() {
            scene.clear();
            return 0;
        }
        scene.clear();
        scene.push(fret_core::SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect: bounds,
            background: fret_core::Paint::Solid(Color {
                r: 0.06,
                g: 0.07,
                b: 0.08,
                a: 1.0,
            }),
            border: fret_core::Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let cols = 22u32;
        let rows = 14u32;
        let pad = Px(10.0);
        let cell = Px(40.0);
        let step = cell.0 + pad.0;
        let start = Point::new(Px(18.0), Px(18.0));

        let mut icons_emitted = 0usize;
        for row in 0..rows {
            for col in 0..cols {
                let idx = (row * cols + col) as usize;
                let size = if phase {
                    Size::new(Px(cell.0), Px(cell.0))
                } else {
                    Size::new(Px(cell.0 * 0.8), Px(cell.0 * 0.8))
                };
                let cell_origin = Point::new(
                    start.x + Px(col as f32 * step),
                    start.y + Px(row as f32 * step),
                );
                let x = cell_origin.x + Px((cell.0 - size.width.0).max(0.0) * 0.5);
                let y = cell_origin.y + Px((cell.0 - size.height.0).max(0.0) * 0.5);
                let rect = Rect::new(Point::new(x, y), size);

                let fit = fit_mode.to_fit(idx);
                let v = variants[idx % variants.len()];
                let svg = if idx % 5 == 0 { v.wide } else { v.square };

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

        icons_emitted
    }

    let viewport_size = (980u32, 720u32);
    let scale_factor = 2.0f32;
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fret headless target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(
            Px(viewport_size.0 as f32 / scale_factor),
            Px(viewport_size.1 as f32 / scale_factor),
        ),
    );

    let mut scene = Scene::default();
    let mut phase = false;
    let fit_mode = FitMode::Mixed;
    let mut next_tag: u32 = 0;
    let mut variants: Vec<SvgVariant> = Vec::new();
    if churn {
        // Pre-fill with a small working set; churn will swap entries and create holes that the
        // append-only atlas cannot reuse, surfacing fragmentation via growing pages + falling fill%.
        for _ in 0..(churn_drop.max(1) * 2) {
            variants.push(register_variant(&mut renderer, next_tag));
            next_tag = next_tag.wrapping_add(1);
        }
    }

    let wait_every = wait_every.max(1);
    let churn_every = churn_every.max(1);
    let start = Instant::now();
    for frame in 0..frames {
        if frame != 0 && frame % 180 == 0 {
            phase = !phase;
        }
        if churn && frame != 0 && frame % churn_every == 0 {
            let drop_n = churn_drop.min(variants.len());
            for v in variants.drain(0..drop_n) {
                let _ = renderer.unregister_svg(v.square);
                let _ = renderer.unregister_svg(v.wide);
            }
            for _ in 0..drop_n {
                variants.push(register_variant(&mut renderer, next_tag));
                next_tag = next_tag.wrapping_add(1);
            }
        }

        let _icons = if churn {
            record_scene_churn(&mut scene, bounds, phase, fit_mode, &variants)
        } else {
            record_scene(&mut scene, bounds, phase, fit_mode, svg_square, svg_wide)
        };
        let cmd = renderer.render_scene(
            &ctx.device,
            &ctx.queue,
            RenderSceneParams {
                format,
                target_view: &view,
                scene: &scene,
                clear: ClearColor::default(),
                scale_factor,
                viewport_size,
            },
        );
        ctx.queue.submit([cmd]);
        if wait_gpu && (frame % wait_every == 0) {
            // Block until the GPU has completed submitted work. This makes the benchmark closer
            // to end-to-end cost (rather than submit-only).
            let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
        }
        if frame % 60 == 0 {
            let _ = ctx.device.poll(wgpu::PollType::Poll);
            try_println!("headless progress: {frame}/{frames}");
        }
    }
    let _ = ctx.device.poll(if wait_gpu {
        wgpu::PollType::wait_indefinitely()
    } else {
        wgpu::PollType::Poll
    });
    let elapsed = start.elapsed();

    if let Some(snap) = renderer.take_svg_perf_snapshot() {
        let fill_pct = if snap.svg_mask_atlas_capacity_px == 0 {
            0.0
        } else {
            (snap.svg_mask_atlas_used_px as f64 / snap.svg_mask_atlas_capacity_px as f64) * 100.0
        };
        try_println!(
            "headless: frames={} wall={:.2}s prepare={:.2}ms hits={} misses={} alpha_raster={} ({:.2}ms) atlas_inserts={} atlas_write={:.2}ms pages={} rasters={} standalone={}KB atlas={}KB fill={:.1}% wait_gpu={} wait_every={} churn={} churn_every={} churn_drop={}",
            frames,
            elapsed.as_secs_f64(),
            snap.prepare_svg_ops_us as f64 / 1000.0,
            snap.cache_hits,
            snap.cache_misses,
            snap.alpha_raster_count,
            snap.alpha_raster_us as f64 / 1000.0,
            snap.alpha_atlas_inserts,
            snap.alpha_atlas_write_us as f64 / 1000.0,
            snap.atlas_pages_live,
            snap.svg_rasters_live,
            snap.svg_standalone_bytes_live / 1024,
            snap.svg_mask_atlas_bytes_live / 1024,
            fill_pct,
            wait_gpu,
            wait_every,
            churn,
            churn_every,
            churn_drop
        );
    }

    if let Some(snap) = renderer.take_perf_snapshot() {
        let pipeline_breakdown = std::env::var_os("FRET_RENDERER_PERF_PIPELINES").is_some();
        try_println!(
            "headless_renderer_perf: frames={} encode={:.2}ms prepare_svg={:.2}ms prepare_text={:.2}ms draws={} (quad={} viewport={} image={} text={} path={} mask={} fs={} clipmask={}) pipelines={} binds={} (ubinds={} tbinds={}) scissor={} uniform={}KB instance={}KB vertex={}KB cache_hits={} cache_misses={}",
            snap.frames,
            snap.encode_scene_us as f64 / 1000.0,
            snap.prepare_svg_us as f64 / 1000.0,
            snap.prepare_text_us as f64 / 1000.0,
            snap.draw_calls,
            snap.quad_draw_calls,
            snap.viewport_draw_calls,
            snap.image_draw_calls,
            snap.text_draw_calls,
            snap.path_draw_calls,
            snap.mask_draw_calls,
            snap.fullscreen_draw_calls,
            snap.clip_mask_draw_calls,
            snap.pipeline_switches,
            snap.bind_group_switches,
            snap.uniform_bind_group_switches,
            snap.texture_bind_group_switches,
            snap.scissor_sets,
            snap.uniform_bytes / 1024,
            snap.instance_bytes / 1024,
            snap.vertex_bytes / 1024,
            snap.scene_encoding_cache_hits,
            snap.scene_encoding_cache_misses
        );
        if pipeline_breakdown {
            try_println!(
                "headless_renderer_perf_pipelines: quad={} viewport={} mask={} text_mask={} text_color={} path={} path_msaa={} composite={} fullscreen={} clip_mask={}",
                snap.pipeline_switches_quad,
                snap.pipeline_switches_viewport,
                snap.pipeline_switches_mask,
                snap.pipeline_switches_text_mask,
                snap.pipeline_switches_text_color,
                snap.pipeline_switches_path,
                snap.pipeline_switches_path_msaa,
                snap.pipeline_switches_composite,
                snap.pipeline_switches_fullscreen,
                snap.pipeline_switches_clip_mask,
            );
        }
    }

    Ok(())
}

impl WinitAppDriver for SvgAtlasStressDriver {
    type WindowState = SvgAtlasStressState;

    fn gpu_ready(
        &mut self,
        _app: &mut App,
        _context: &fret_render::WgpuContext,
        renderer: &mut fret_render::Renderer,
    ) {
        renderer.set_svg_perf_enabled(true);
        renderer.set_perf_enabled(true);
    }

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
        if state.clear_requested {
            renderer.clear_svg_raster_cache();
            state.clear_requested = false;
        }
        if state.clear_atlas_requested {
            renderer.clear_svg_mask_atlas_cache();
            state.clear_atlas_requested = false;
        }

        let desired = state.budget_presets[state.budget_index];
        if state.budget_applied != desired {
            renderer.set_svg_raster_budget_bytes(desired);
            state.budget_applied = desired;
        }

        let now = Instant::now();
        let should_report = match state.last_renderer_report {
            None => true,
            Some(last) => now.duration_since(last) >= Duration::from_secs(1),
        };
        if should_report {
            if let Some(snap) = renderer.take_perf_snapshot() {
                if snap.frames != 0 {
                    let pipeline_breakdown =
                        std::env::var_os("FRET_RENDERER_PERF_PIPELINES").is_some();
                    try_println!(
                        "renderer_perf: frames={} encode={:.2}ms prepare_svg={:.2}ms prepare_text={:.2}ms draws={} (quad={} viewport={} image={} text={} path={} mask={} fs={} clipmask={}) pipelines={} binds={} (ubinds={} tbinds={}) scissor={} uniform={}KB instance={}KB vertex={}KB cache_hits={} cache_misses={}",
                        snap.frames,
                        snap.encode_scene_us as f64 / 1000.0,
                        snap.prepare_svg_us as f64 / 1000.0,
                        snap.prepare_text_us as f64 / 1000.0,
                        snap.draw_calls,
                        snap.quad_draw_calls,
                        snap.viewport_draw_calls,
                        snap.image_draw_calls,
                        snap.text_draw_calls,
                        snap.path_draw_calls,
                        snap.mask_draw_calls,
                        snap.fullscreen_draw_calls,
                        snap.clip_mask_draw_calls,
                        snap.pipeline_switches,
                        snap.bind_group_switches,
                        snap.uniform_bind_group_switches,
                        snap.texture_bind_group_switches,
                        snap.scissor_sets,
                        snap.uniform_bytes / 1024,
                        snap.instance_bytes / 1024,
                        snap.vertex_bytes / 1024,
                        snap.scene_encoding_cache_hits,
                        snap.scene_encoding_cache_misses
                    );
                    if pipeline_breakdown {
                        try_println!(
                            "renderer_perf_pipelines: quad={} viewport={} mask={} text_mask={} text_color={} path={} path_msaa={} composite={} fullscreen={} clip_mask={}",
                            snap.pipeline_switches_quad,
                            snap.pipeline_switches_viewport,
                            snap.pipeline_switches_mask,
                            snap.pipeline_switches_text_mask,
                            snap.pipeline_switches_text_color,
                            snap.pipeline_switches_path,
                            snap.pipeline_switches_path_msaa,
                            snap.pipeline_switches_composite,
                            snap.pipeline_switches_fullscreen,
                            snap.pipeline_switches_clip_mask,
                        );
                    }
                }
            }
            if let Some(snap) = renderer.take_svg_perf_snapshot() {
                if snap.frames == 0 {
                    state.last_renderer_report = Some(now);
                    return;
                }
                let fill_pct = if snap.svg_mask_atlas_capacity_px == 0 {
                    0.0
                } else {
                    (snap.svg_mask_atlas_used_px as f64 / snap.svg_mask_atlas_capacity_px as f64)
                        * 100.0
                };
                try_println!(
                    "renderer_svg: frames={} prepare={:.2}ms hits={} misses={} alpha_raster={} ({:.2}ms) rgba_raster={} ({:.2}ms) atlas_inserts={} atlas_write={:.2}ms pages={} rasters={} standalone={}KB atlas={}KB fill={:.1}%",
                    snap.frames,
                    snap.prepare_svg_ops_us as f64 / 1000.0,
                    snap.cache_hits,
                    snap.cache_misses,
                    snap.alpha_raster_count,
                    snap.alpha_raster_us as f64 / 1000.0,
                    snap.rgba_raster_count,
                    snap.rgba_raster_us as f64 / 1000.0,
                    snap.alpha_atlas_inserts,
                    snap.alpha_atlas_write_us as f64 / 1000.0,
                    snap.atlas_pages_live,
                    snap.svg_rasters_live,
                    snap.svg_standalone_bytes_live / 1024,
                    snap.svg_mask_atlas_bytes_live / 1024,
                    fill_pct
                );
            }
            state.last_renderer_report = Some(now);
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app, window, state, ..
        } = context;
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
            KeyCode::KeyC => {
                state.clear_requested = true;
                try_println!("svg_atlas_stress: clear svg raster cache requested");
            }
            KeyCode::KeyM => {
                state.clear_atlas_requested = true;
                try_println!("svg_atlas_stress: clear svg mask atlas cache requested");
            }
            KeyCode::KeyH => Self::print_help(),
            _ => {}
        }

        app.request_redraw(window);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            window,
            state,
            bounds,
            services,
            scene,
            ..
        } = context;
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

        let icons_emitted = record_scene(
            scene,
            bounds,
            state.phase,
            state.fit_mode,
            svg_square,
            svg_wide,
        );

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
            try_println!(
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
                .add_directive("fret_launch=info".parse().unwrap()),
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
    let mut headless = false;
    let mut budget_kb: Option<u64> = None;
    let mut wait_gpu = false;
    let mut wait_every: u64 = 1;
    let mut churn = false;
    let mut churn_every: u64 = 180;
    let mut churn_drop: usize = 64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--headless" => headless = true,
            "--frames" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--frames requires a value");
                };
                max_frames = Some(value.parse()?);
            }
            "--budget-kb" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--budget-kb requires a value");
                };
                budget_kb = Some(value.parse()?);
            }
            "--wait-gpu" => wait_gpu = true,
            "--wait-every" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--wait-every requires a value");
                };
                wait_every = value.parse()?;
            }
            "--churn" => churn = true,
            "--churn-every" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--churn-every requires a value");
                };
                churn_every = value.parse()?;
            }
            "--churn-drop" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--churn-drop requires a value");
                };
                churn_drop = value.parse()?;
            }
            "--help" | "-h" => {
                try_println!(
                    "Usage: fret-svg-atlas-stress [--frames N] [--headless] [--budget-kb KB] [--wait-gpu] [--wait-every N] [--churn] [--churn-every N] [--churn-drop N]"
                );
                return Ok(());
            }
            other => anyhow::bail!("unknown arg: {other}"),
        }
    }

    if headless {
        let frames = max_frames.unwrap_or(600);
        let budget = budget_kb.map(|kb| kb * 1024).unwrap_or(1024 * 1024);
        return run_headless(
            frames,
            budget,
            wait_gpu,
            wait_every,
            churn,
            churn_every,
            churn_drop,
        );
    }

    let driver = SvgAtlasStressDriver { max_frames };
    run_app_with_event_loop(event_loop, config, app, driver)?;
    Ok(())
}
