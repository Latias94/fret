use std::sync::Arc;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Edges, Event, Px, Rect, UiServices};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext,
    WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::action::UiActionHostAdapter;
use fret_ui::declarative;
use fret_ui::element::{CrossAlign, FlexProps, LayoutStyle, MainAlign};
use fret_ui::{Invalidation, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn as shadcn;

struct SonnerDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    last_action: Model<Arc<str>>,
    promise: Option<shadcn::ToastPromise>,
}

#[derive(Default)]
struct SonnerDemoDriver;

impl SonnerDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> SonnerDemoWindowState {
        let last_action = app.models_mut().insert(Arc::<str>::from("<none>"));

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        SonnerDemoWindowState {
            ui,
            root: None,
            last_action,
            promise: None,
        }
    }

    fn render_demo(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut SonnerDemoWindowState,
        bounds: Rect,
    ) {
        OverlayController::begin_frame(app, window);

        let last_action = state.last_action.clone();
        let promise_active = state.promise.is_some();

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("sonner-demo", |cx| {
                    cx.observe_model(&last_action, Invalidation::Layout);
                    let last_action_value = cx
                        .app
                        .models()
                        .get_cloned(&last_action)
                        .unwrap_or_else(|| Arc::<str>::from("<none>"));

                    vec![
                        cx.flex(
                            FlexProps {
                                layout: LayoutStyle::default(),
                                direction: fret_core::Axis::Vertical,
                                gap: fret_ui::element::SpacingLength::Px(Px(12.0)),
                                padding: Edges::all(Px(24.0)).into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                            },
                            |cx| {
                                vec![
                                    cx.text("Sonner (shadcn/ui) demo"),
                                    cx.text(format!(
                                        "promise active: {promise_active} | last action: {last_action_value}"
                                    )),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: fret_ui::element::SpacingLength::Px(Px(8.0)),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: true,
                                        },
                                        |cx| {
                                            vec![
                                                shadcn::Button::new("Default")
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.default",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Description")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.description",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Success")
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.success",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Info")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from("sonner.toast.info"))
                                                    .into_element(cx),
                                                shadcn::Button::new("Warning")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.warning",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Error")
                                                    .variant(shadcn::ButtonVariant::Destructive)
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.error",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Loading (pinned)")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.loading",
                                                    ))
                                                    .into_element(cx),
                                            ]
                                        },
                                    ),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: fret_ui::element::SpacingLength::Px(Px(8.0)),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: true,
                                        },
                                        |cx| {
                                            vec![
                                                shadcn::Button::new("Action + Cancel")
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.action_cancel",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Pinned")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.pinned",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Not dismissible")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from(
                                                        "sonner.toast.not_dismissible",
                                                    ))
                                                    .into_element(cx),
                                            ]
                                        },
                                    ),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: fret_ui::element::SpacingLength::Px(Px(8.0)),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: true,
                                        },
                                        |cx| {
                                            vec![
                                                shadcn::Button::new("Promise: start")
                                                    .on_click(CommandId::from(
                                                        "sonner.promise.start",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Promise: resolve success")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from(
                                                        "sonner.promise.success",
                                                    ))
                                                    .into_element(cx),
                                                shadcn::Button::new("Promise: resolve error")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from(
                                                        "sonner.promise.error",
                                                    ))
                                                    .into_element(cx),
                                            ]
                                        },
                                    ),
                                ]
                            },
                        ),
                        shadcn::Toaster::new().into_element(cx),
                    ]
                });

        state.ui.set_root(root);
        OverlayController::render(&mut state.ui, app, services, window, bounds);
        state.root = Some(root);
    }
}

impl WinitAppDriver for SonnerDemoDriver {
    type WindowState = SonnerDemoWindowState;

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
        state.root = None;
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        let WinitWindowContext { app, state, .. } = context;
        state.ui.propagate_global_changes(app, changed);
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

        let sonner = shadcn::Sonner::global(app);

        match command.as_str() {
            "sonner.toast.default" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Default toast",
                    shadcn::ToastMessageOptions::new(),
                );
            }
            "sonner.toast.description" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Description toast",
                    shadcn::ToastMessageOptions::new().description("This is a description."),
                );
            }
            "sonner.toast.success" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_success_message(
                    &mut host,
                    window,
                    "Success!",
                    shadcn::ToastMessageOptions::new().description("Everything worked."),
                );
            }
            "sonner.toast.info" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_info_message(
                    &mut host,
                    window,
                    "Info",
                    shadcn::ToastMessageOptions::new().description("FYI: this is informational."),
                );
            }
            "sonner.toast.warning" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_warning_message(
                    &mut host,
                    window,
                    "Warning",
                    shadcn::ToastMessageOptions::new().description("Something looks off."),
                );
            }
            "sonner.toast.error" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_error_message(
                    &mut host,
                    window,
                    "Error",
                    shadcn::ToastMessageOptions::new().description("Something failed."),
                );
            }
            "sonner.toast.loading" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_loading_message(
                    &mut host,
                    window,
                    "Loading…",
                    shadcn::ToastMessageOptions::new().description("This is pinned by default."),
                );
            }
            "sonner.toast.action_cancel" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Action toast",
                    shadcn::ToastMessageOptions::new()
                        .description("Try the action/cancel buttons.")
                        .action("Undo", "sonner.toast.action")
                        .cancel("Cancel", "sonner.toast.cancel"),
                );
            }
            "sonner.toast.pinned" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Pinned toast",
                    shadcn::ToastMessageOptions::new()
                        .description("This toast does not auto-close.")
                        .pinned(),
                );
            }
            "sonner.toast.not_dismissible" => {
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Not dismissible",
                    shadcn::ToastMessageOptions::new()
                        .description("Swipe-to-dismiss is disabled.")
                        .dismissible(false)
                        .duration(Duration::from_secs(6)),
                );
            }
            "sonner.promise.start" => {
                let promise = {
                    let mut host = UiActionHostAdapter { app };
                    sonner.toast_promise(&mut host, window, "Working…")
                };
                state.promise = Some(promise);
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("promise.start");
                });
            }
            "sonner.promise.success" => {
                if let Some(promise) = state.promise.take() {
                    {
                        let mut host = UiActionHostAdapter { app };
                        promise.success_with(
                            &mut host,
                            "Done!",
                            shadcn::ToastMessageOptions::new().description("Promise resolved."),
                        );
                    }
                    let _ = app.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("promise.success");
                    });
                } else {
                    let mut host = UiActionHostAdapter { app };
                    sonner.toast_info_message(
                        &mut host,
                        window,
                        "No active promise",
                        shadcn::ToastMessageOptions::new()
                            .description("Click “Promise: start” first."),
                    );
                }
            }
            "sonner.promise.error" => {
                if let Some(promise) = state.promise.take() {
                    {
                        let mut host = UiActionHostAdapter { app };
                        promise.error_with(
                            &mut host,
                            "Failed",
                            shadcn::ToastMessageOptions::new().description("Promise rejected."),
                        );
                    }
                    let _ = app.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("promise.error");
                    });
                } else {
                    let mut host = UiActionHostAdapter { app };
                    sonner.toast_info_message(
                        &mut host,
                        window,
                        "No active promise",
                        shadcn::ToastMessageOptions::new()
                            .description("Click “Promise: start” first."),
                    );
                }
            }
            "sonner.toast.action" => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.action");
                });
            }
            "sonner.toast.cancel" => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.cancel");
                });
            }
            _ => {}
        }

        app.request_redraw(window);
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
        Self::render_demo(app, services, window, state, bounds);

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
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo sonner_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(900.0, 540.0),
        ..Default::default()
    }
}

fn create_window_state(
    driver: &mut SonnerDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> SonnerDemoWindowState {
    <SonnerDemoDriver as WinitAppDriver>::create_window_state(driver, app, window)
}

fn hot_reload_window(
    driver: &mut SonnerDemoDriver,
    context: WinitHotReloadContext<'_, SonnerDemoWindowState>,
) {
    let WinitHotReloadContext {
        app,
        services,
        window,
        state,
    } = context;
    <SonnerDemoDriver as WinitAppDriver>::hot_reload_window(driver, app, services, window, state)
}

fn handle_global_changes(
    driver: &mut SonnerDemoDriver,
    context: WinitWindowContext<'_, SonnerDemoWindowState>,
    changed: &[std::any::TypeId],
) {
    <SonnerDemoDriver as WinitAppDriver>::handle_global_changes(driver, context, changed)
}

fn handle_event(
    driver: &mut SonnerDemoDriver,
    context: WinitEventContext<'_, SonnerDemoWindowState>,
    event: &Event,
) {
    <SonnerDemoDriver as WinitAppDriver>::handle_event(driver, context, event)
}

fn handle_command(
    driver: &mut SonnerDemoDriver,
    context: WinitCommandContext<'_, SonnerDemoWindowState>,
    command: CommandId,
) {
    <SonnerDemoDriver as WinitAppDriver>::handle_command(driver, context, command)
}

fn render(driver: &mut SonnerDemoDriver, context: WinitRenderContext<'_, SonnerDemoWindowState>) {
    <SonnerDemoDriver as WinitAppDriver>::render(driver, context)
}

fn window_create_spec(
    driver: &mut SonnerDemoDriver,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    <SonnerDemoDriver as WinitAppDriver>::window_create_spec(driver, app, request)
}

pub fn build_fn_driver() -> impl WinitAppDriver {
    FnDriver::new(
        SonnerDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(|hooks| {
        hooks.hot_reload_window = Some(hot_reload_window);
        hooks.handle_global_changes = Some(handle_global_changes);
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
    let driver = build_fn_driver();

    fret::run_native_with_compat_driver(config, app, driver).context("run sonner_demo app")
}
