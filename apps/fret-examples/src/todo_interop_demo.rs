use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, Px, Rect, UiServices, ViewportFit, ViewportInputEvent};
use fret_launch::{
    EngineFrameUpdate, ViewportRenderTarget, WinitAppDriver, WinitCommandContext,
    WinitEventContext, WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{AnyElement, AnyElementIterExt as _};
use fret_ui::{Theme, UiFrameCx, UiTree};
use fret_ui_kit::declarative as kit_decl;
use fret_ui_kit::prelude::*;
use fret_ui_shadcn as shadcn;
use std::sync::Arc;

const CMD_ADD: &str = "todo-interop.add";
const CMD_CLEAR_DONE: &str = "todo-interop.clear_done";
const CMD_REMOVE_PREFIX: &str = "todo-interop.remove.";

#[derive(Debug, Clone)]
struct ExternalViewportStubState {
    target: fret_core::RenderTargetId,
    clicks: u32,
    last_input: Arc<str>,
}

impl Default for ExternalViewportStubState {
    fn default() -> Self {
        Self {
            target: fret_core::RenderTargetId::default(),
            clicks: 0,
            last_input: Arc::from("Click inside the viewport surface."),
        }
    }
}

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoInteropWindowState {
    ui: UiTree<App>,
    draft: Model<String>,
    todos: Model<Vec<TodoItem>>,
    next_id: u64,

    // "Foreign runtime" stub: an app-owned offscreen render target embedded via ViewportSurface.
    external_target: ViewportRenderTarget,
    external_target_px_size: (u32, u32),
}

#[derive(Default)]
struct TodoInteropDriver;

impl TodoInteropDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> TodoInteropWindowState {
        let draft = app.models_mut().insert(String::new());
        let done_1 = app.models_mut().insert(false);
        let done_2 = app.models_mut().insert(true);
        let todos = app.models_mut().insert(vec![
            TodoItem {
                id: 1,
                done: done_1,
                text: Arc::from("Add an item"),
            },
            TodoItem {
                id: 2,
                done: done_2,
                text: Arc::from("Click the embedded viewport"),
            },
        ]);

        let mut ui = UiTree::new();
        ui.set_window(window);

        TodoInteropWindowState {
            ui,
            draft,
            todos,
            next_id: 3,
            external_target: ViewportRenderTarget::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                RenderTargetColorSpace::Srgb,
            ),
            external_target_px_size: (640, 360),
        }
    }

    fn on_add(app: &mut App, state: &mut TodoInteropWindowState) {
        let draft = app
            .models()
            .read(&state.draft, |s| s.trim().to_string())
            .unwrap_or_default();
        if draft.is_empty() {
            return;
        }

        let id = state.next_id;
        state.next_id += 1;
        let done = app.models_mut().insert(false);
        let item = TodoItem {
            id,
            done,
            text: Arc::from(draft),
        };

        let _ = app.models_mut().update(&state.todos, |v| v.push(item));
        let _ = app.models_mut().update(&state.draft, |s| s.clear());
    }

    fn on_clear_done(app: &mut App, state: &mut TodoInteropWindowState) {
        let done_ids: std::collections::HashSet<u64> = app
            .models()
            .read(&state.todos, |items| {
                items
                    .iter()
                    .filter_map(|t| {
                        app.models()
                            .read(&t.done, |b| *b)
                            .ok()
                            .and_then(|done| done.then_some(t.id))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let _ = app
            .models_mut()
            .update(&state.todos, |v| v.retain(|t| !done_ids.contains(&t.id)));
    }

    fn on_remove(app: &mut App, state: &mut TodoInteropWindowState, id: u64) {
        let _ = app
            .models_mut()
            .update(&state.todos, |v| v.retain(|t| t.id != id));
    }

    fn render(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        state: &mut TodoInteropWindowState,
    ) {
        let draft = state.draft.clone();
        let todos_model = state.todos.clone();
        let ext_target = state.external_target.id();
        let ext_size = state.external_target_px_size;

        let ui = &mut state.ui;
        let root = declarative::RenderRootContext::new(ui, app, services, window, bounds)
            .render_root("todo-interop", |cx| {
                let theme = Theme::global(&*cx.app).clone();
                let ext = cx
                    .watch_global::<ExternalViewportStubState>()
                    .paint()
                    .cloned()
                    .unwrap_or_default();

                cx.watch_model(&draft).layout().observe();
                let todos = cx
                    .watch_model(&todos_model)
                    .layout()
                    .cloned()
                    .unwrap_or_default();
                for t in &todos {
                    cx.watch_model(&t.done).layout().observe();
                }

                let root_el = ui::container(cx, |cx| {
                    let left = todo_panel(cx, &theme, draft.clone(), &todos);
                    let right = external_panel(
                        cx,
                        &theme,
                        ext_target,
                        ext_size,
                        ext.clicks,
                        ext.last_input,
                    );

                    [ui::h_flex(cx, move |_cx| [left, right])
                        .gap(Space::N4)
                        .p(Space::N4)
                        .w_full()
                        .h_full()
                        .into_element(cx)]
                })
                .bg(ColorRef::Color(theme.color_required("background")))
                .w_full()
                .h_full()
                .into_element(cx);

                std::iter::once(root_el)
            });

        ui.set_root(root);
    }
}

impl WinitAppDriver for TodoInteropDriver {
    type WindowState = TodoInteropWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn UiServices,
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

        match command.as_str() {
            CMD_ADD => Self::on_add(app, state),
            CMD_CLEAR_DONE => Self::on_clear_done(app, state),
            "window.close" => app.push_effect(Effect::Window(WindowRequest::Close(window))),
            _ => {
                if let Some(rest) = command.as_str().strip_prefix(CMD_REMOVE_PREFIX) {
                    if let Ok(id) = rest.parse::<u64>() {
                        Self::on_remove(app, state, id);
                    }
                }
            }
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

        state.ui.dispatch_event(app, services, event);
    }

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        app.with_global_mut(ExternalViewportStubState::default, |st, app| {
            if st.target == fret_core::RenderTargetId::default() || st.target != event.target {
                return;
            }

            if matches!(event.kind, fret_core::ViewportInputKind::PointerDown { .. }) {
                st.clicks = st.clicks.saturating_add(1);
            }

            st.last_input = Arc::from(format!(
                "kind={:?} uv=({:.3},{:.3}) target_px={:?}",
                event.kind, event.uv.0, event.uv.1, event.target_px
            ));
            app.request_redraw(event.window);
        });
    }

    fn record_engine_frame(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: fret_runtime::TickId,
        frame_id: fret_runtime::FrameId,
    ) -> EngineFrameUpdate {
        let (id, view) = state.external_target.ensure_size_owned_view(
            context,
            renderer,
            state.external_target_px_size,
            Some("todo-interop external viewport"),
        );

        app.with_global_mut(ExternalViewportStubState::default, |st, _app| {
            st.target = id;
        });

        let clicks = app
            .global::<ExternalViewportStubState>()
            .map(|s| s.clicks)
            .unwrap_or(0);

        let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
        let f = ((clicks % 8) as f32) / 8.0;
        let clear = wgpu::Color {
            r: (0.08 + 0.30 * t) as f64,
            g: (0.08 + 0.25 * (1.0 - t)) as f64,
            b: (0.10 + 0.35 * f) as f64,
            a: 1.0,
        };

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("todo-interop viewport encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("todo-interop viewport clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        EngineFrameUpdate {
            target_updates: Vec::new(),
            command_buffers: vec![encoder.finish()],
        }
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

        Self::render(app, services, window, bounds, state);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        let mut frame = UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
    }
}

fn todo_panel(
    cx: &mut fret_ui::ElementContext<'_, App>,
    theme: &Theme,
    draft_model: Model<String>,
    todos: &[TodoItem],
) -> AnyElement {
    let draft = shadcn::Input::new(draft_model)
        .a11y_label("Todo")
        .placeholder("New task...")
        .submit_command(CommandId::new(CMD_ADD))
        .into_element(cx);

    let add = shadcn::Button::new("Add")
        .on_click(CMD_ADD)
        .into_element(cx);

    let clear_done = shadcn::Button::new("Clear done")
        .variant(shadcn::ButtonVariant::Outline)
        .on_click(CMD_CLEAR_DONE)
        .into_element(cx);

    let list = todos.iter().map(|t| todo_row(cx, theme, t)).elements();

    let header =
        shadcn::CardHeader::new([shadcn::CardTitle::new("Todo").into_element(cx)]).into_element(cx);

    let body = shadcn::CardContent::new([
        ui::h_flex(cx, move |_cx| [draft, add, clear_done])
            .gap(Space::N2)
            .items_center()
            .w_full()
            .into_element(cx),
        cx.column(fret_ui::element::ColumnProps::default(), move |_cx| list),
    ])
    .into_element(cx);

    shadcn::Card::new([header, body])
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(360.0)))
                .h_full(),
        )
        .ui()
        .p_3()
        .into_element(cx)
}

fn todo_row(
    cx: &mut fret_ui::ElementContext<'_, App>,
    _theme: &Theme,
    item: &TodoItem,
) -> AnyElement {
    let remove = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
        .on_click(CommandId::new(format!("{CMD_REMOVE_PREFIX}{}", item.id)))
        .into_element(cx);

    let checkbox = shadcn::Checkbox::new(item.done.clone())
        .a11y_label("Done")
        .into_element(cx);

    let text = cx.text(item.text.clone());

    let row = ui::h_flex(cx, move |_cx| [checkbox, text, remove])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx);

    ui::container(cx, move |_cx| [row])
        .border_1()
        .rounded(Radius::Md)
        .p(Space::N2)
        .w_full()
        .into_element(cx)
}

fn external_panel(
    cx: &mut fret_ui::ElementContext<'_, App>,
    _theme: &Theme,
    target: fret_core::RenderTargetId,
    target_px_size: (u32, u32),
    clicks: u32,
    last_input: Arc<str>,
) -> AnyElement {
    let title = shadcn::CardTitle::new("External viewport (stub)").into_element(cx);
    let desc = shadcn::CardDescription::new(format!(
        "Clicks: {clicks} | Target: {target:?} | {}x{}",
        target_px_size.0, target_px_size.1
    ))
    .into_element(cx);

    let last = shadcn::Badge::new(last_input)
        .variant(shadcn::BadgeVariant::Secondary)
        .into_element(cx);

    let header = shadcn::CardHeader::new([title, desc, last]).into_element(cx);

    let viewport = ui::container(cx, move |cx| {
        let props = kit_decl::viewport_surface::ViewportSurfacePanelProps {
            target,
            target_px_size,
            fit: ViewportFit::Contain,
            opacity: 1.0,
            forward_input: true,
        };
        [kit_decl::viewport_surface::viewport_surface_panel(
            cx, props,
        )]
    })
    .border_1()
    .rounded(Radius::Md)
    .w_full()
    .h_px(MetricRef::Px(Px(360.0)))
    .into_element(cx);

    let body = shadcn::CardContent::new([viewport]).into_element(cx);

    shadcn::Card::new([header, body])
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo todo_interop_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    TodoInteropDriver
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

    crate::run_native_demo(build_runner_config(), build_app(), build_driver())
        .context("run todo_interop_demo app")
}
