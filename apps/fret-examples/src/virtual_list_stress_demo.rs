use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, Px};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    VirtualListOptions,
};
use fret_ui::{Invalidation, UiTree, VirtualListScrollHandle};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use std::sync::Arc;
use std::time::{Duration, Instant};

const LIST_LEN: usize = 100_000;

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

fn parse_env_u64(key: &str) -> Option<u64> {
    std::env::var_os(key).and_then(|v| v.to_string_lossy().parse::<u64>().ok())
}

fn parse_env_bool(key: &str) -> bool {
    let Some(raw) = std::env::var_os(key) else {
        return false;
    };
    let value = raw.to_string_lossy().trim().to_ascii_lowercase();
    matches!(value.as_str(), "1" | "true" | "yes" | "on")
}

struct VirtualListStressWindowState {
    ui: UiTree<App>,
    scroll_handle: VirtualListScrollHandle,
    tall_rows_enabled: fret_app::Model<bool>,
    reversed: fret_app::Model<bool>,
    items_revision: fret_app::Model<u64>,
    frame: u64,
    exit_after_frames: Option<u64>,
    auto_scroll: bool,
    last_renderer_report: Option<Instant>,
}

#[derive(Default)]
struct VirtualListStressDriver;

impl VirtualListStressDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> VirtualListStressWindowState {
        let tall_rows_enabled = app.models_mut().insert(false);
        let reversed = app.models_mut().insert(false);
        let items_revision = app.models_mut().insert(0u64);
        let exit_after_frames = parse_env_u64("FRET_VLIST_STRESS_EXIT_AFTER_FRAMES");
        let auto_scroll = parse_env_bool("FRET_VLIST_STRESS_AUTO_SCROLL");

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        VirtualListStressWindowState {
            ui,
            scroll_handle: VirtualListScrollHandle::new(),
            tall_rows_enabled,
            reversed,
            items_revision,
            frame: 0,
            exit_after_frames,
            auto_scroll,
            last_renderer_report: None,
        }
    }
}

impl WinitAppDriver for VirtualListStressDriver {
    type WindowState = VirtualListStressWindowState;

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, renderer: &mut Renderer) {
        renderer.set_perf_enabled(true);
    }

    fn gpu_frame_prepare(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        _context: &WgpuContext,
        renderer: &mut Renderer,
        _scale_factor: f32,
    ) {
        if !state.auto_scroll && state.exit_after_frames.is_none() {
            return;
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
            state.last_renderer_report = Some(now);
        }
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
        } = context;

        if state.ui.dispatch_command(app, services, &command) {
            return;
        }

        if command.as_str() == "virtual_list_stress_demo.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;

        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if let Event::KeyDown { key, modifiers, .. } = event {
            if modifiers.ctrl || modifiers.alt || modifiers.shift || modifiers.meta {
                state.ui.dispatch_event(app, services, event);
                return;
            }

            match *key {
                fret_core::KeyCode::Escape => {
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                    return;
                }
                fret_core::KeyCode::Space => {
                    let _ = app
                        .models_mut()
                        .update(&state.tall_rows_enabled, |v| *v = !*v);
                    app.request_redraw(window);
                }
                fret_core::KeyCode::KeyR => {
                    let _ = app.models_mut().update(&state.reversed, |v| *v = !*v);
                    let _ = app
                        .models_mut()
                        .update(&state.items_revision, |v| *v = v.wrapping_add(1));
                    app.request_redraw(window);
                }
                fret_core::KeyCode::Home => {
                    state
                        .scroll_handle
                        .scroll_to_item(0, fret_ui::ScrollStrategy::Start);
                    app.request_redraw(window);
                }
                fret_core::KeyCode::End => {
                    state.scroll_handle.scroll_to_bottom();
                    app.request_redraw(window);
                }
                fret_core::KeyCode::KeyG => {
                    state
                        .scroll_handle
                        .scroll_to_item(LIST_LEN / 2, fret_ui::ScrollStrategy::Center);
                    app.request_redraw(window);
                }
                _ => {}
            }
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        state.frame = state.frame.wrapping_add(1);

        let tall_rows_enabled = app
            .models()
            .read(&state.tall_rows_enabled, |v| *v)
            .unwrap_or(false);
        let reversed = app.models().read(&state.reversed, |v| *v).unwrap_or(false);
        let items_revision = app
            .models()
            .read(&state.items_revision, |v| *v)
            .unwrap_or(0);

        let scroll_handle = state.scroll_handle.clone();
        let offset_y = scroll_handle.offset().y;

        let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
            .render_root("virtual-list-stress", |cx| {
                cx.observe_model(&state.tall_rows_enabled, Invalidation::Layout);
                cx.observe_model(&state.reversed, Invalidation::Layout);
                cx.observe_model(&state.items_revision, Invalidation::Layout);

                let theme = cx.theme_snapshot();
                let padding = theme.metric_required("metric.padding.md");

                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;

                let header: Arc<str> = Arc::from(format!(
                    "VirtualList stress demo | rows={LIST_LEN} | offset_y={:.1} | tall={} | reversed={} | [Space]=toggle tall | [R]=reverse | [G]=go mid | [Home]/[End] | [Esc]=close",
                    offset_y.0, tall_rows_enabled, reversed
                ));

                let mut list_slot = LayoutStyle::default();
                list_slot.size.width = Length::Fill;
                list_slot.size.height = Length::Fill;
                list_slot.flex.grow = 1.0;
                list_slot.flex.basis = Length::Px(Px(0.0));
                list_slot.overflow = Overflow::Clip;

                let mut options = VirtualListOptions::new(Px(18.0), 10);
                options.items_revision = items_revision;
                options.gap = Px(2.0);

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        background: Some(theme.color_required("background")),
                        ..Default::default()
                    },
                    |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: root_layout,
                                        direction: fret_core::Axis::Vertical,
                                        gap: Px(8.0),
                                        padding: fret_core::Edges::all(padding),
                                        justify: MainAlign::Start,
                                        align: CrossAlign::Stretch,
                                        wrap: false,
                                    },
                            |cx| {
                                vec![
                                    cx.text(header),
                                    cx.container(
                                        ContainerProps {
                                            layout: list_slot,
                                            background: Some(theme.color_required("card")),
                                            border: fret_core::Edges::all(Px(1.0)),
                                            border_color: Some(theme.color_required("border")),
                                            corner_radii: fret_core::Corners::all(Px(8.0)),
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![cx.virtual_list_keyed_with_layout(
                                                {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout.size.height = Length::Fill;
                                                    layout
                                                },
                                                LIST_LEN,
                                                options,
                                                &scroll_handle,
                                                move |i| {
                                                    if reversed {
                                                        (LIST_LEN - 1 - i) as fret_ui::ItemKey
                                                    } else {
                                                        i as fret_ui::ItemKey
                                                    }
                                                },
                                                |cx, index| {
                                                    let id = if reversed {
                                                        LIST_LEN - 1 - index
                                                    } else {
                                                        index
                                                    };

                                                    let mut row_layout = LayoutStyle::default();
                                                    row_layout.size.width = Length::Fill;
                                                    row_layout.size.height = Length::Px(if tall_rows_enabled
                                                        && (id % 15 == 0 || id % 17 == 0)
                                                    {
                                                        Px(72.0)
                                                    } else {
                                                        Px(18.0)
                                                    });

                                                    let bg = if id % 2 == 0 {
                                                        theme.color_required("background")
                                                    } else {
                                                        theme.color_required("card")
                                                    };

                                                    cx.container(
                                                        ContainerProps {
                                                            layout: row_layout,
                                                            background: Some(bg),
                                                            padding: fret_core::Edges::symmetric(
                                                                theme.metric_required(
                                                                    "metric.padding.md",
                                                                ),
                                                                Px(0.0),
                                                            ),
                                                            ..Default::default()
                                                        },
                                                        |cx| {
                                                            if id % 37 == 0 {
                                                                vec![cx.text(Arc::<str>::from(
                                                                    format!("Row {id} (tall={tall_rows_enabled})"),
                                                                ))]
                                                            } else {
                                                                Vec::new()
                                                            }
                                                        },
                                                    )
                                                },
                                            )]
                                        },
                                    ),
                                ]
                            },
                        )]
                    },
                )]
            });

        state.ui.set_root(root);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);

        if let Some(limit) = state.exit_after_frames
            && state.frame >= limit
        {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if state.auto_scroll {
            let index = ((state.frame as usize).saturating_mul(37)) % LIST_LEN;
            state
                .scroll_handle
                .scroll_to_item(index, fret_ui::ScrollStrategy::Start);
            app.request_redraw(window);
        } else if state.exit_after_frames.is_some() {
            app.request_redraw(window);
        }
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo virtual_list_stress_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    crate::run_native_demo(config, app, VirtualListStressDriver::default())
        .context("run virtual_list_stress_demo app")
}
