use fret_app::{App, Effect, WindowRequest};
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Scene, SceneOp,
};
use fret_core::text::{FontWeight, TextConstraints, TextOverflow, TextStyle, TextWrap};
use fret_launch::{WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig};
use fret_render::{Renderer, WgpuContext};
use std::time::{Duration, Instant};

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

fn parse_env_bool(key: &str) -> Option<bool> {
    std::env::var_os(key).and_then(|v| match v.to_string_lossy().trim() {
        "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON" => Some(true),
        "0" | "false" | "FALSE" | "no" | "NO" | "off" | "OFF" => Some(false),
        _ => None,
    })
}

fn parse_env_u32(key: &str) -> Option<u32> {
    std::env::var_os(key).and_then(|v| v.to_string_lossy().trim().parse::<u32>().ok())
}

fn parse_env_quality(key: &str) -> Option<EffectQuality> {
    std::env::var_os(key).and_then(|v| match v.to_string_lossy().trim() {
        "auto" | "Auto" | "AUTO" => Some(EffectQuality::Auto),
        "low" | "Low" | "LOW" => Some(EffectQuality::Low),
        "medium" | "Medium" | "MEDIUM" => Some(EffectQuality::Medium),
        "high" | "High" | "HIGH" => Some(EffectQuality::High),
        _ => None,
    })
}

#[derive(Default)]
struct EffectsDemoDriver;

#[derive(Debug, Clone)]
struct OverlayTextCache {
    last_text: String,
    last_scale_bits: u32,
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<fret_core::text::TextMetrics>,
}

impl Default for OverlayTextCache {
    fn default() -> Self {
        Self {
            last_text: String::new(),
            last_scale_bits: 0,
            blob: None,
            metrics: None,
        }
    }
}

#[derive(Debug, Clone)]
struct EffectsDemoState {
    panel0_enabled: bool,
    panel1_enabled: bool,
    panel2_enabled: bool,

    quality: EffectQuality,

    panel0_blur_radius_px: u32,
    panel0_blur_downsample: u32,

    panel1_pixelate_scale: u32,
    panel2_pixelate_scale: u32,

    show_help: bool,
    overlay_dirty: bool,
    overlay: OverlayTextCache,

    frame: u64,
    exit_after_frames: Option<u64>,
    last_renderer_report: Option<Instant>,
}

impl Default for EffectsDemoState {
    fn default() -> Self {
        let exit_after_frames = std::env::var("FRET_EFFECTS_DEMO_EXIT_AFTER_FRAMES")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&v| v > 0);

        Self {
            panel0_enabled: parse_env_bool("FRET_EFFECTS_PANEL0").unwrap_or(true),
            panel1_enabled: parse_env_bool("FRET_EFFECTS_PANEL1").unwrap_or(true),
            panel2_enabled: parse_env_bool("FRET_EFFECTS_PANEL2").unwrap_or(true),

            quality: parse_env_quality("FRET_EFFECTS_QUALITY").unwrap_or(EffectQuality::Auto),

            panel0_blur_radius_px: parse_env_u32("FRET_EFFECTS_BLUR_RADIUS_PX").unwrap_or(6),
            panel0_blur_downsample: parse_env_u32("FRET_EFFECTS_BLUR_DOWNSAMPLE").unwrap_or(2),

            panel1_pixelate_scale: parse_env_u32("FRET_EFFECTS_P1_PIXELATE_SCALE").unwrap_or(8),
            panel2_pixelate_scale: parse_env_u32("FRET_EFFECTS_P2_PIXELATE_SCALE").unwrap_or(6),

            show_help: true,
            overlay_dirty: true,
            overlay: OverlayTextCache::default(),

            frame: 0,
            exit_after_frames,
            last_renderer_report: None,
        }
    }
}

impl EffectsDemoState {
    fn any_effects_enabled(&self) -> bool {
        self.panel0_enabled || self.panel1_enabled || self.panel2_enabled
    }

    fn cycle_quality(&mut self) {
        self.quality = match self.quality {
            EffectQuality::Auto => EffectQuality::Low,
            EffectQuality::Low => EffectQuality::Medium,
            EffectQuality::Medium => EffectQuality::High,
            EffectQuality::High => EffectQuality::Auto,
        };
    }

    fn overlay_text(&self) -> String {
        if !self.show_help {
            return String::new();
        }

        let quality = match self.quality {
            EffectQuality::Auto => "Auto",
            EffectQuality::Low => "Low",
            EffectQuality::Medium => "Medium",
            EffectQuality::High => "High",
        };

        format!(
            "effects_demo controls\n\
             0: toggle all effects ({})\n\
             1/2/3: toggle panels (p0={}, p1={}, p2={})\n\
             Q: cycle quality (current: {})\n\
             [/]: blur radius (p0={}, downsample={})\n\
             -/=: pixelate scale (p1={})\n\
             Z/X: pixelate scale (p2={})\n\
             H: toggle this help\n\
             Esc: close window",
            self.any_effects_enabled(),
            self.panel0_enabled,
            self.panel1_enabled,
            self.panel2_enabled,
            quality,
            self.panel0_blur_radius_px,
            self.panel0_blur_downsample,
            self.panel1_pixelate_scale,
            self.panel2_pixelate_scale,
        )
    }
}

impl WinitAppDriver for EffectsDemoDriver {
    type WindowState = EffectsDemoState;

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, renderer: &mut Renderer) {
        let perf_enabled = std::env::var_os("FRET_EFFECTS_DEMO_EXIT_AFTER_FRAMES").is_some()
            || std::env::var_os("FRET_EFFECTS_DEMO_PROFILE").is_some()
            || std::env::var_os("FRET_RENDERER_PERF_PIPELINES").is_some();
        if perf_enabled {
            renderer.set_perf_enabled(true);
        }
    }

    fn gpu_frame_prepare(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        _context: &WgpuContext,
        renderer: &mut Renderer,
        _scale_factor: f32,
    ) {
        let profiling = std::env::var_os("FRET_EFFECTS_DEMO_PROFILE").is_some()
            || state.exit_after_frames.is_some();
        if !profiling {
            return;
        }

        state.frame = state.frame.saturating_add(1);

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
            state.last_renderer_report = Some(now);
        }

        if let Some(limit) = state.exit_after_frames {
            if state.frame >= limit {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
        }

        app.request_redraw(window);
    }

    fn create_window_state(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
    ) -> Self::WindowState {
        EffectsDemoState::default()
    }

    fn handle_event(
        &mut self,
        context: WinitEventContext<'_, Self::WindowState>,
        event: &fret_core::Event,
    ) {
        let WinitEventContext {
            app, window, state, ..
        } = context;
        match event {
            fret_core::Event::WindowCloseRequested
            | fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            fret_core::Event::KeyDown {
                key,
                modifiers,
                repeat: false,
            } => {
                let mut changed = false;

                match key {
                    fret_core::KeyCode::Digit0 => {
                        let enable = !state.any_effects_enabled();
                        state.panel0_enabled = enable;
                        state.panel1_enabled = enable;
                        state.panel2_enabled = enable;
                        changed = true;
                    }
                    fret_core::KeyCode::Digit1 => {
                        state.panel0_enabled = !state.panel0_enabled;
                        changed = true;
                    }
                    fret_core::KeyCode::Digit2 => {
                        state.panel1_enabled = !state.panel1_enabled;
                        changed = true;
                    }
                    fret_core::KeyCode::Digit3 => {
                        state.panel2_enabled = !state.panel2_enabled;
                        changed = true;
                    }
                    fret_core::KeyCode::KeyQ => {
                        state.cycle_quality();
                        changed = true;
                    }
                    fret_core::KeyCode::BracketLeft => {
                        let step = if modifiers.shift { 4 } else { 1 };
                        state.panel0_blur_radius_px =
                            state.panel0_blur_radius_px.saturating_sub(step).max(0);
                        changed = true;
                    }
                    fret_core::KeyCode::BracketRight => {
                        let step = if modifiers.shift { 4 } else { 1 };
                        state.panel0_blur_radius_px =
                            (state.panel0_blur_radius_px.saturating_add(step)).min(64);
                        changed = true;
                    }
                    fret_core::KeyCode::Minus => {
                        let step = if modifiers.shift { 4 } else { 1 };
                        state.panel1_pixelate_scale =
                            state.panel1_pixelate_scale.saturating_sub(step).max(1);
                        changed = true;
                    }
                    fret_core::KeyCode::Equal => {
                        let step = if modifiers.shift { 4 } else { 1 };
                        state.panel1_pixelate_scale =
                            (state.panel1_pixelate_scale.saturating_add(step)).min(64);
                        changed = true;
                    }
                    fret_core::KeyCode::KeyZ => {
                        let step = if modifiers.shift { 4 } else { 1 };
                        state.panel2_pixelate_scale =
                            state.panel2_pixelate_scale.saturating_sub(step).max(1);
                        changed = true;
                    }
                    fret_core::KeyCode::KeyX => {
                        let step = if modifiers.shift { 4 } else { 1 };
                        state.panel2_pixelate_scale =
                            (state.panel2_pixelate_scale.saturating_add(step)).min(64);
                        changed = true;
                    }
                    fret_core::KeyCode::KeyH => {
                        state.show_help = !state.show_help;
                        changed = true;
                    }
                    _ => {}
                }

                if changed {
                    state.overlay_dirty = true;
                    app.request_redraw(window);
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            bounds,
            scale_factor,
            scene,
            services,
            state,
            ..
        } = context;
        scene.clear();

        let w = bounds.size.width.0.max(1.0);
        let h = bounds.size.height.0.max(1.0);
        let full = Rect::new(bounds.origin, Size::new(Px(w), Px(h)));

        // Background.
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: full,
            background: fret_core::Paint::Solid(Color {
                r: 0.08,
                g: 0.09,
                b: 0.12,
                a: 1.0,
            }),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(0.0)),
        });

        // Color stripes: higher-frequency signal for blur / pixelate.
        let stripe_w = 10.0_f32.max(2.0);
        let stripe_count = ((w / stripe_w).ceil() as u32).max(1);
        for i in 0..stripe_count {
            let x = bounds.origin.x.0 + stripe_w * i as f32;
            let denom = (stripe_count.saturating_sub(1)).max(1) as f32;
            let t = i as f32 / denom;
            let (r, g, b) = if t < 0.33 {
                (1.0, t / 0.33, 0.0)
            } else if t < 0.66 {
                (1.0 - (t - 0.33) / 0.33, 1.0, (t - 0.33) / 0.33)
            } else {
                (0.0, 1.0 - (t - 0.66) / 0.34, 1.0)
            };
            scene.push(SceneOp::Quad {
                order: DrawOrder(1 + i),
                rect: Rect::new(
                    Point::new(Px(x), bounds.origin.y),
                    Size::new(Px(stripe_w), Px(h)),
                ),
                background: fret_core::Paint::Solid(Color { r, g, b, a: 1.0 }),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(Px(0.0)),
            });
        }

        // Three panels.
        let pad = 24.0;
        let panel_h = (h - pad * 2.0).max(120.0);
        let panel_w = ((w - pad * 4.0) / 3.0).max(120.0);
        let y0 = bounds.origin.y.0 + pad;

        let panel0 = Rect::new(
            Point::new(Px(bounds.origin.x.0 + pad), Px(y0)),
            Size::new(Px(panel_w), Px(panel_h)),
        );
        let panel1 = Rect::new(
            Point::new(Px(bounds.origin.x.0 + pad * 2.0 + panel_w), Px(y0)),
            Size::new(Px(panel_w), Px(panel_h)),
        );
        let panel2 = Rect::new(
            Point::new(Px(bounds.origin.x.0 + pad * 3.0 + panel_w * 2.0), Px(y0)),
            Size::new(Px(panel_w), Px(panel_h)),
        );

        let panel_radii = Corners::all(Px(18.0));

        let panel_border = |scene: &mut Scene, order: u32, rect: Rect, color: Color| {
            scene.push(SceneOp::Quad {
                order: DrawOrder(order),
                rect,
                background: fret_core::Paint::Solid(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }),
                border: Edges::all(Px(2.0)),
                border_paint: fret_core::Paint::Solid(color),

                corner_radii: panel_radii,
            });
        };

        // Panel 0: backdrop blur + slight color adjust (glass-ish).
        scene.push(SceneOp::PushClipRRect {
            rect: panel0,
            corner_radii: panel_radii,
        });
        if state.panel0_enabled && state.panel0_blur_radius_px > 0 {
            scene.push(SceneOp::PushEffect {
                bounds: panel0,
                mode: EffectMode::Backdrop,
                chain: EffectChain::from_steps(&[
                    EffectStep::GaussianBlur {
                        radius_px: Px(state.panel0_blur_radius_px as f32),
                        downsample: state.panel0_blur_downsample.max(2),
                    },
                    EffectStep::ColorAdjust {
                        saturation: 1.2,
                        brightness: 1.02,
                        contrast: 1.02,
                    },
                ]),
                quality: state.quality,
            });
        }
        scene.push(SceneOp::Quad {
            order: DrawOrder(10_000),
            rect: panel0,
            background: fret_core::Paint::Solid(Color {
                r: 0.08,
                g: 0.08,
                b: 0.08,
                a: 0.08,
            }),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: panel_radii,
        });
        panel_border(
            scene,
            10_100,
            panel0,
            Color {
                r: 0.35,
                g: 0.35,
                b: 0.35,
                a: if state.panel0_enabled { 0.35 } else { 0.18 },
            },
        );
        if state.panel0_enabled && state.panel0_blur_radius_px > 0 {
            scene.push(SceneOp::PopEffect);
        }
        scene.push(SceneOp::PopClip);

        // Panel 1: backdrop pixelate.
        scene.push(SceneOp::PushClipRRect {
            rect: panel1,
            corner_radii: panel_radii,
        });
        if state.panel1_enabled {
            scene.push(SceneOp::PushEffect {
                bounds: panel1,
                mode: EffectMode::Backdrop,
                chain: EffectChain::from_steps(&[EffectStep::Pixelate {
                    scale: state.panel1_pixelate_scale.max(1),
                }]),
                quality: state.quality,
            });
        }
        scene.push(SceneOp::Quad {
            order: DrawOrder(11_000),
            rect: panel1,
            background: fret_core::Paint::Solid(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.12,
            }),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: panel_radii,
        });
        panel_border(
            scene,
            11_100,
            panel1,
            Color {
                r: 0.55,
                g: 0.495,
                b: 0.33,
                a: if state.panel1_enabled { 0.55 } else { 0.22 },
            },
        );
        if state.panel1_enabled {
            scene.push(SceneOp::PopEffect);
        }
        scene.push(SceneOp::PopClip);

        // Panel 2: filter-content pixelate applied to a subtree (content drawn inside the group).
        scene.push(SceneOp::PushClipRRect {
            rect: panel2,
            corner_radii: panel_radii,
        });
        if state.panel2_enabled {
            scene.push(SceneOp::PushEffect {
                bounds: panel2,
                mode: EffectMode::FilterContent,
                chain: EffectChain::from_steps(&[EffectStep::Pixelate {
                    scale: state.panel2_pixelate_scale.max(1),
                }]),
                quality: state.quality,
            });
        }
        // High-frequency stripes so pixelation is obvious.
        let stripe_w = 2.0_f32;
        let count = (panel2.size.width.0 / stripe_w).ceil().max(1.0) as u32;
        for i in 0..count {
            let x = panel2.origin.x.0 + stripe_w * i as f32;
            let is_red = (i % 2) == 0;
            let bg = if is_red {
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 1.0,
                }
            };
            scene.push(SceneOp::Quad {
                order: DrawOrder(12_000 + i),
                rect: Rect::new(
                    Point::new(Px(x), panel2.origin.y),
                    Size::new(Px(stripe_w), panel2.size.height),
                ),
                background: fret_core::Paint::Solid(bg),

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: Corners::all(Px(0.0)),
            });
        }
        // Slight tint to keep the panel readable (premultiplied).
        scene.push(SceneOp::Quad {
            order: DrawOrder(12_900),
            rect: panel2,
            background: fret_core::Paint::Solid(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.12,
            }),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: panel_radii,
        });
        if state.panel2_enabled {
            scene.push(SceneOp::PopEffect);
        }
        panel_border(
            scene,
            13_000,
            panel2,
            Color {
                r: 0.495,
                g: 0.495,
                b: 0.5225,
                a: if state.panel2_enabled { 0.55 } else { 0.22 },
            },
        );
        scene.push(SceneOp::PopClip);

        // Foreground marker (ordering sanity).
        scene.push(SceneOp::Quad {
            order: DrawOrder(20_000),
            rect: Rect::new(
                Point::new(
                    Px(bounds.origin.x.0 + w - 120.0),
                    Px(bounds.origin.y.0 + h - 80.0),
                ),
                Size::new(Px(96.0), Px(56.0)),
            ),
            background: fret_core::Paint::Solid(Color {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 0.9,
            }),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT,

            corner_radii: Corners::all(Px(14.0)),
        });

        // Debug overlay: show controls/state without depending on higher-level widgets.
        if state.show_help {
            let overlay_text = state.overlay_text();
            let scale_bits = scale_factor.to_bits();
            if state.overlay_dirty
                || state.overlay.last_scale_bits != scale_bits
                || state.overlay.last_text != overlay_text
            {
                if let Some(blob) = state.overlay.blob.take() {
                    services.text().release(blob);
                }

                let style = TextStyle {
                    font: fret_core::FontId::default(),
                    size: Px(13.0),
                    weight: FontWeight::MEDIUM,
                    slant: fret_core::text::TextSlant::Normal,
                    line_height: Some(Px(16.0)),
                    letter_spacing_em: None,
                };
                let constraints = TextConstraints {
                    max_width: Some(Px(w - pad * 2.0)),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    scale_factor,
                };

                let (blob, metrics) =
                    services
                        .text()
                        .prepare_str(overlay_text.as_str(), &style, constraints);
                state.overlay.last_text = overlay_text;
                state.overlay.last_scale_bits = scale_bits;
                state.overlay.blob = Some(blob);
                state.overlay.metrics = Some(metrics);
                state.overlay_dirty = false;
            }

            if let (Some(blob), Some(metrics)) = (state.overlay.blob, state.overlay.metrics) {
                let pad_px = Px(10.0);
                let bg_rect = Rect::new(
                    Point::new(Px(bounds.origin.x.0 + pad), Px(bounds.origin.y.0 + pad)),
                    Size::new(
                        Px(metrics.size.width.0 + pad_px.0 * 2.0),
                        Px(metrics.size.height.0 + pad_px.0 * 2.0),
                    ),
                );
                scene.push(SceneOp::Quad {
                    order: DrawOrder(30_000),
                    rect: bg_rect,
                    background: fret_core::Paint::Solid(Color {
                        r: 0.06,
                        g: 0.06,
                        b: 0.07,
                        a: 0.72,
                    }),
                    border: Edges::all(Px(1.0)),
                    border_paint: fret_core::Paint::Solid(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.12,
                    }),
                    corner_radii: Corners::all(Px(12.0)),
                });

                scene.push(SceneOp::Text {
                    order: DrawOrder(30_001),
                    origin: Point::new(
                        Px(bg_rect.origin.x.0 + pad_px.0),
                        Px(bg_rect.origin.y.0 + pad_px.0 + metrics.baseline.0),
                    ),
                    text: blob,
                    color: Color {
                        r: 0.95,
                        g: 0.95,
                        b: 0.95,
                        a: 0.95,
                    },
                });
            }
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    crate::run_native_demo(
        WinitRunnerConfig {
            main_window_title: "effects_demo".to_string(),
            main_window_size: winit::dpi::LogicalSize::new(1100.0, 520.0),
            ..Default::default()
        },
        App::new(),
        EffectsDemoDriver::default(),
    )
}
