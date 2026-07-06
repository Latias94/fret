use anyhow::Context as _;
use fret::advanced::view::AppRenderDataExt as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Event, Px};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext, WinitHotReloadContext,
    WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::{ModelStore, PlatformCapabilities};
use fret_ui::declarative;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    VirtualListOptions,
};
use fret_ui::{ElementContext, Invalidation, UiTree, VirtualListScrollHandle};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::text as decl_text;
use std::sync::Arc;
use std::time::{Duration, Instant};

const LIST_LEN: usize = 100_000;

fn virtual_list_stress_readout_text<H: fret_ui::UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_control_readout(cx, text)
}

fn virtual_list_stress_row_label_text<H: fret_ui::UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_list_row_label(cx, text)
}

struct VirtualListStressControls {
    tall_rows_enabled: Model<bool>,
    reversed: Model<bool>,
    items_revision: Model<u64>,
}

struct VirtualListStressSnapshot {
    tall_rows_enabled: bool,
    reversed: bool,
    items_revision: u64,
}

impl VirtualListStressControls {
    fn new(models: &mut ModelStore) -> Self {
        Self {
            tall_rows_enabled: models.insert(false),
            reversed: models.insert(false),
            items_revision: models.insert(0u64),
        }
    }

    fn toggle_rows_enabled(&self, models: &mut ModelStore) -> bool {
        Self::toggle_bool(models, &self.tall_rows_enabled)
    }

    fn toggle_reversed_and_bump_revision(&self, models: &mut ModelStore) -> bool {
        let toggled = Self::toggle_bool(models, &self.reversed);
        let bumped = self.bump_items_revision(models);
        toggled || bumped
    }

    fn layout_snapshot(&self, cx: &mut ElementContext<'_, App>) -> VirtualListStressSnapshot {
        let (tall_rows_enabled, reversed, items_revision): (bool, bool, u64) =
            cx.data().selector_model_layout(
                (
                    &self.tall_rows_enabled,
                    &self.reversed,
                    &self.items_revision,
                ),
                |(tall_rows_enabled, reversed, items_revision)| {
                    (tall_rows_enabled, reversed, items_revision)
                },
            );

        VirtualListStressSnapshot {
            tall_rows_enabled,
            reversed,
            items_revision,
        }
    }

    fn bump_items_revision(&self, models: &mut ModelStore) -> bool {
        models
            .update(&self.items_revision, |value| {
                *value = value.wrapping_add(1);
                true
            })
            .unwrap_or(false)
    }

    fn toggle_bool(models: &mut ModelStore, model: &Model<bool>) -> bool {
        models
            .update(model, |value| {
                *value = !*value;
                true
            })
            .unwrap_or(false)
    }
}

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

pub struct VirtualListStressWindowState {
    ui: UiTree<App>,
    scroll_handle: VirtualListScrollHandle,
    controls: VirtualListStressControls,
    frame: u64,
    exit_after_frames: Option<u64>,
    auto_scroll: bool,
    last_renderer_report: Option<Instant>,
}

#[derive(Default)]
pub struct VirtualListStressDriver;

impl VirtualListStressDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> VirtualListStressWindowState {
        let controls = VirtualListStressControls::new(app.models_mut());
        let exit_after_frames = parse_env_u64("FRET_VLIST_STRESS_EXIT_AFTER_FRAMES");
        let auto_scroll = parse_env_bool("FRET_VLIST_STRESS_AUTO_SCROLL");

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        VirtualListStressWindowState {
            ui,
            scroll_handle: VirtualListScrollHandle::new(),
            controls,
            frame: 0,
            exit_after_frames,
            auto_scroll,
            last_renderer_report: None,
        }
    }
}

fn gpu_ready(
    driver: &mut VirtualListStressDriver,
    app: &mut App,
    context: &WgpuContext,
    renderer: &mut Renderer,
) {
    renderer.set_perf_enabled(true);
}

fn gpu_frame_prepare(
    driver: &mut VirtualListStressDriver,
    app: &mut App,
    window: fret_core::AppWindowId,
    state: &mut VirtualListStressWindowState,
    context: &WgpuContext,
    renderer: &mut Renderer,
    scale_factor: f32,
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
                let pipeline_breakdown = std::env::var_os("FRET_RENDERER_PERF_PIPELINES").is_some();
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

fn create_window_state(
    _driver: &mut VirtualListStressDriver,
    app: &mut App,
    window: AppWindowId,
) -> VirtualListStressWindowState {
    VirtualListStressDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut VirtualListStressDriver,
    context: WinitHotReloadContext<'_, VirtualListStressWindowState>,
) {
    let WinitHotReloadContext {
        app,
        services: _,
        window,
        state,
    } = context;
    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
}

fn handle_model_changes(
    _driver: &mut VirtualListStressDriver,
    context: WinitWindowContext<'_, VirtualListStressWindowState>,
    changed: &[fret_app::ModelId],
) {
    context
        .state
        .ui
        .propagate_model_changes(context.app, changed);
}

fn handle_global_changes(
    _driver: &mut VirtualListStressDriver,
    context: WinitWindowContext<'_, VirtualListStressWindowState>,
    changed: &[std::any::TypeId],
) {
    context
        .state
        .ui
        .propagate_global_changes(context.app, changed);
}

fn handle_command(
    _driver: &mut VirtualListStressDriver,
    context: WinitCommandContext<'_, VirtualListStressWindowState>,
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

fn handle_event(
    _driver: &mut VirtualListStressDriver,
    context: WinitEventContext<'_, VirtualListStressWindowState>,
    event: &Event,
) {
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
                let _ = state.controls.toggle_rows_enabled(app.models_mut());
                app.request_redraw(window);
            }
            fret_core::KeyCode::KeyR => {
                let _ = state
                    .controls
                    .toggle_reversed_and_bump_revision(app.models_mut());
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

fn render(
    _driver: &mut VirtualListStressDriver,
    context: WinitRenderContext<'_, VirtualListStressWindowState>,
) {
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

    let scroll_handle = state.scroll_handle.clone();
    let offset_y = scroll_handle.offset().y;

    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
            .render_root("virtual-list-stress", |cx| {
                let controls = state.controls.layout_snapshot(cx);
                let tall_rows_enabled = controls.tall_rows_enabled;
                let reversed = controls.reversed;

                let theme = cx.theme_snapshot();
                let padding = theme.metric_token("metric.padding.md");

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
                options.items_revision = controls.items_revision;
                options.gap = Px(2.0);

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        background: Some(theme.color_token("background")),
                        ..Default::default()
                    },
                    |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: root_layout,
                                        direction: fret_core::Axis::Vertical,
                                        gap: fret_ui::element::SpacingLength::Px(Px(8.0)),
                                        padding: fret_core::Edges::all(padding).into(),
                                        justify: MainAlign::Start,
                                        align: CrossAlign::Stretch,
                                        wrap: false,
                                    },
                            |cx| {
                                vec![
                                    virtual_list_stress_readout_text(cx, header),
                                    cx.container(
                                        ContainerProps {
                                            layout: list_slot,
                                            background: Some(theme.color_token("card")),
                                            border: fret_core::Edges::all(Px(1.0)),
                                            border_color: Some(theme.color_token("border")),
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
                                                        theme.color_token("background")
                                                    } else {
                                                        theme.color_token("card")
                                                    };

                                                    cx.container(
                                                        ContainerProps {
                                                            layout: row_layout,
                                                            background: Some(bg),
                                                            padding: fret_core::Edges::symmetric(
                                                                theme.metric_token(
                                                                    "metric.padding.md",
                                                                ),
                                                                Px(0.0),
                                                            )
                                                            .into(),
                                                            ..Default::default()
                                                        },
                                                        |cx| {
                                                            if id % 37 == 0 {
                                                                let label = Arc::<str>::from(format!(
                                                                    "Row {id} (tall={tall_rows_enabled})"
                                                                ));
                                                                vec![virtual_list_stress_row_label_text(cx, label)]
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
    _driver: &mut VirtualListStressDriver,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<VirtualListStressDriver, VirtualListStressWindowState>,
) {
    hooks.gpu_ready = Some(gpu_ready);
    hooks.hot_reload_window = Some(hot_reload_window);
    hooks.gpu_frame_prepare = Some(gpu_frame_prepare);
    hooks.handle_model_changes = Some(handle_model_changes);
    hooks.handle_global_changes = Some(handle_global_changes);
    hooks.handle_command = Some(handle_command);
    hooks.window_create_spec = Some(window_create_spec);
}

pub fn build_fn_driver() -> FnDriver<VirtualListStressDriver, VirtualListStressWindowState> {
    FnDriver::new(
        VirtualListStressDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
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
        main_window_size: fret_launch::WindowLogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    crate::run_native_with_fn_driver_with_hooks(
        config,
        app,
        VirtualListStressDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
    .context("run virtual_list_stress_demo app")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_list_stress_controls_preserve_command_state_transitions() {
        let mut models = ModelStore::default();
        let controls = VirtualListStressControls::new(&mut models);

        assert!(controls.toggle_rows_enabled(&mut models));
        assert_eq!(models.get_copied(&controls.tall_rows_enabled), Some(true));

        assert!(controls.toggle_reversed_and_bump_revision(&mut models));
        assert_eq!(models.get_copied(&controls.reversed), Some(true));
        assert_eq!(models.get_copied(&controls.items_revision), Some(1));

        models
            .update(&controls.items_revision, |revision| *revision = u64::MAX)
            .expect("items_revision model should exist");
        assert!(controls.toggle_reversed_and_bump_revision(&mut models));
        assert_eq!(models.get_copied(&controls.reversed), Some(false));
        assert_eq!(models.get_copied(&controls.items_revision), Some(0));
    }
}
