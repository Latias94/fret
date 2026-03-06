use std::sync::Arc;

use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Event, Px, Rect, UiServices};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext,
    WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::Invalidation;
use fret_ui::UiTree;
use fret_ui::declarative;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::{ColorRef, Radius, Space};
use fret_ui_shadcn::{self as shadcn, prelude::*};

const TEST_ID_INPUT: &str = "simple-todo.input";
const TEST_ID_ADD: &str = "simple-todo.add";
const TEST_ID_CLEAR_DONE: &str = "simple-todo.clear-done";

const CMD_ADD: &str = "simple_todo.add";
const CMD_CLEAR_DONE: &str = "simple_todo.clear_done";
const CMD_REMOVE_PREFIX: &str = "simple_todo.remove.";

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct SimpleTodoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    next_id: u64,
}

#[derive(Default)]
struct SimpleTodoDriver;

impl SimpleTodoDriver {
    fn render(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut SimpleTodoWindowState,
        bounds: Rect,
    ) {
        let todos_model = state.todos.clone();
        let draft_model = state.draft.clone();

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("simple-todo-demo", |cx| {
                    // Keep invalidation explicit and boring for onboarding: the todo list affects layout,
                    // the draft affects button enabled state (paint), and each row has a done model.
                    cx.observe_model(&todos_model, Invalidation::Layout);
                    cx.observe_model(&draft_model, Invalidation::Paint);

                    let theme = cx.theme_snapshot();

                    let todos = cx.watch_model(&todos_model).layout().cloned_or_default();
                    let draft_value = cx.watch_model(&draft_model).paint().cloned_or_default();

                    let mut done_count = 0usize;
                    for t in &todos {
                        if cx.watch_model(&t.done).paint().copied_or_default() {
                            done_count += 1;
                        }
                    }
                    let total_count = todos.len();

                    let add_enabled = !draft_value.trim().is_empty();
                    let add_cmd = CommandId::new(CMD_ADD);
                    let clear_done_cmd = CommandId::new(CMD_CLEAR_DONE);

                    let progress = shadcn::Badge::new(format!("{done_count}/{total_count} done"))
                        .variant(shadcn::BadgeVariant::Secondary);

                    let clear_done_btn = shadcn::Button::new("Clear done")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .disabled(done_count == 0)
                        .on_click(clear_done_cmd)
                        .test_id(TEST_ID_CLEAR_DONE)
                        .ui()
                        .rounded_md();

                    let header_actions =
                        ui::h_flex(|cx| ui::children![cx; progress, clear_done_btn])
                            .gap(Space::N2)
                            .items_center()
                            .into_element(cx);

                    let add_btn = shadcn::Button::new("Add")
                        .disabled(!add_enabled)
                        .on_click(add_cmd.clone())
                        .test_id(TEST_ID_ADD)
                        .ui()
                        .rounded_md();

                    let input = shadcn::Input::new(draft_model.clone())
                        .placeholder("Add a task…")
                        .submit_command(add_cmd)
                        .into_element(cx)
                        .test_id(TEST_ID_INPUT);

                    let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
                        .gap(Space::N2)
                        .items_center()
                        .w_full()
                        .into_element(cx);

                    let rows = ui::v_flex_build(|cx, out| {
                        for t in &todos {
                            let remove_cmd = CommandId::new(format!("{CMD_REMOVE_PREFIX}{}", t.id));
                            out.push(
                                cx.keyed(t.id, |cx| todo_row(cx, theme.clone(), t, remove_cmd)),
                            );
                        }
                    })
                    .gap(Space::N3)
                    .w_full()
                    .into_element(cx);

                    let content = ui::v_flex(|cx| ui::children![cx; input_row, rows])
                        .gap(Space::N4)
                        .w_full()
                        .into_element(cx);

                    let card = shadcn::Card::new(ui::children![
                        cx;
                        shadcn::CardHeader::new(ui::children![
                            cx;
                            shadcn::CardTitle::new("Simple Todo"),
                            shadcn::CardDescription::new(
                                "A small baseline: Model + keyed lists (no selector/query).",
                            ),
                            header_actions,
                        ]),
                        shadcn::CardContent::new(ui::children![cx; content]),
                    ])
                    .ui()
                    .bg(ColorRef::Color(theme.color_token("background")))
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_token("border")))
                    .w_full()
                    .max_w(Px(520.0));

                    let page = ui::container(|cx| {
                        let centered = ui::v_flex(|cx| ui::children![cx; card])
                            .w_full()
                            .h_full()
                            .justify_center()
                            .items_center()
                            .into_element(cx);
                        ui::children![cx; centered]
                    })
                    .bg(ColorRef::Color(theme.color_token("muted")))
                    .p(Space::N6)
                    .w_full()
                    .h_full();

                    ui::children![cx; page.into_element(cx)]
                });

        state.ui.set_root(root);
        state.root = Some(root);
    }

    fn handle_command(app: &mut App, state: &mut SimpleTodoWindowState, command: &CommandId) {
        match command.as_str() {
            CMD_ADD => {
                let text = app
                    .models()
                    .read(&state.draft, |v| v.trim().to_string())
                    .ok()
                    .unwrap_or_default();
                if text.is_empty() {
                    return;
                }

                let done = app.models_mut().insert(false);
                let id = state.next_id;
                state.next_id = state.next_id.saturating_add(1).max(1);

                let _ = app.models_mut().update(&state.todos, |todos| {
                    todos.push(TodoItem {
                        id,
                        done,
                        text: Arc::from(text),
                    });
                });
                let _ = app.models_mut().update(&state.draft, |v| v.clear());
            }
            CMD_CLEAR_DONE => {
                let todos = app
                    .models()
                    .read(&state.todos, |v| v.clone())
                    .ok()
                    .unwrap_or_default();
                if todos.is_empty() {
                    return;
                }

                let mut remove_ids = Vec::new();
                for t in &todos {
                    let done = app.models().read(&t.done, |v| *v).ok().unwrap_or(false);
                    if done {
                        remove_ids.push(t.id);
                    }
                }
                if remove_ids.is_empty() {
                    return;
                }

                let _ = app.models_mut().update(&state.todos, |todos| {
                    todos.retain(|t| !remove_ids.contains(&t.id));
                });
            }
            other => {
                let Some(rest) = other.strip_prefix(CMD_REMOVE_PREFIX) else {
                    return;
                };
                let Ok(id) = rest.parse::<u64>() else {
                    return;
                };

                let _ = app.models_mut().update(&state.todos, |todos| {
                    todos.retain(|t| t.id != id);
                });
            }
        }
    }
}

fn todo_row(
    cx: &mut fret_ui::ElementContext<'_, App>,
    theme: fret_ui::ThemeSnapshot,
    item: &TodoItem,
    remove_cmd: CommandId,
) -> fret_ui::element::AnyElement {
    let done = cx.watch_model(&item.done).paint().copied_or_default();

    let checkbox = shadcn::Checkbox::new(item.done.clone());

    let text = ui::text(item.text.clone())
        .truncate()
        .text_sm()
        .text_color(ColorRef::Color(if done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    let remove_btn = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .ui()
        .rounded_md();

    ui::h_flex(|cx| ui::children![cx; checkbox, text, remove_btn])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx)
}

impl WinitAppDriver for SimpleTodoDriver {
    type WindowState = SimpleTodoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let done_1 = app.models_mut().insert(false);
        let done_2 = app.models_mut().insert(true);
        let todos = app.models_mut().insert(vec![
            TodoItem {
                id: 1,
                done: done_1,
                text: Arc::from("Use keyed rows for dynamic lists"),
            },
            TodoItem {
                id: 2,
                done: done_2,
                text: Arc::from("Try the shadcn theme tokens"),
            },
        ]);
        let draft = app.models_mut().insert(String::new());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        SimpleTodoWindowState {
            ui,
            root: None,
            todos,
            draft,
            next_id: 3,
        }
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
        state.root = None;
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app, state, window, ..
        } = context;
        Self::handle_command(app, state, &command);
        app.request_redraw(window);
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;

        match event {
            Event::WindowCloseRequested
            | Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
            _ => {}
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

        Self::render(app, services, window, state, bounds);

        if let Some(root) = state.root {
            state.ui.set_root(root);
        }
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
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

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo simple-todo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(560.0, 520.0),
        ..Default::default()
    }
}

fn create_window_state(
    driver: &mut SimpleTodoDriver,
    app: &mut App,
    window: AppWindowId,
) -> SimpleTodoWindowState {
    <SimpleTodoDriver as WinitAppDriver>::create_window_state(driver, app, window)
}

fn hot_reload_window(
    driver: &mut SimpleTodoDriver,
    context: WinitHotReloadContext<'_, SimpleTodoWindowState>,
) {
    let WinitHotReloadContext {
        app,
        services,
        window,
        state,
    } = context;
    <SimpleTodoDriver as WinitAppDriver>::hot_reload_window(driver, app, services, window, state)
}

fn handle_command(
    driver: &mut SimpleTodoDriver,
    context: WinitCommandContext<'_, SimpleTodoWindowState>,
    command: CommandId,
) {
    <SimpleTodoDriver as WinitAppDriver>::handle_command(driver, context, command)
}

fn handle_event(
    driver: &mut SimpleTodoDriver,
    context: WinitEventContext<'_, SimpleTodoWindowState>,
    event: &Event,
) {
    <SimpleTodoDriver as WinitAppDriver>::handle_event(driver, context, event)
}

fn render(driver: &mut SimpleTodoDriver, context: WinitRenderContext<'_, SimpleTodoWindowState>) {
    <SimpleTodoDriver as WinitAppDriver>::render(driver, context)
}

fn window_create_spec(
    driver: &mut SimpleTodoDriver,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    <SimpleTodoDriver as WinitAppDriver>::window_create_spec(driver, app, request)
}

pub fn build_driver() -> impl WinitAppDriver {
    FnDriver::new(
        SimpleTodoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(|hooks| {
        hooks.hot_reload_window = Some(hot_reload_window);
        hooks.handle_command = Some(handle_command);
        hooks.window_create_spec = Some(window_create_spec);
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();
    crate::run_native_with_compat_driver(config, app, driver)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
