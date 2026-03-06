use anyhow::Context as _;
use fret_app::{App, CommandId, Effect};
use fret_core::{AppWindowId, Event, Px, Rect, UiServices};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext,
    WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::{
    BindingV1, KeySpecV1, Keymap, KeymapFileV1, KeymapService, Model, PlatformCapabilities,
};
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_shadcn as shadcn;
use std::sync::Arc;
struct ImeSmokeWindowState {
    ui: UiTree<App>,
    input_single: Model<String>,
    input_multi: Model<String>,
    last_ime: Model<Arc<str>>,
}

#[derive(Default)]
struct ImeSmokeDriver;

impl ImeSmokeDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> ImeSmokeWindowState {
        let input_single = app.models_mut().insert(String::new());
        let input_multi = app.models_mut().insert(String::new());
        let last_ime = app.models_mut().insert(Arc::<str>::from("IME: <none>"));

        let mut ui = UiTree::new();
        ui.set_window(window);

        ImeSmokeWindowState {
            ui,
            input_single,
            input_multi,
            last_ime,
        }
    }

    fn render(
        app: &mut App,
        ui: &mut UiTree<App>,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        input_single: Model<String>,
        input_multi: Model<String>,
        last_ime: Model<Arc<str>>,
    ) {
        let root = declarative::RenderRootContext::new(ui, app, services, window, bounds).render_root(
            "ime-smoke",
            |cx| {
             cx.observe_model(&input_single, Invalidation::Layout);
             cx.observe_model(&input_multi, Invalidation::Layout);
             cx.observe_model(&last_ime, Invalidation::Paint);

            let theme = Theme::global(&*cx.app).snapshot();

            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;

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
                            gap: fret_ui::element::SpacingLength::Px(Px(12.0)),
                            padding: fret_core::Edges::all(
                                theme.metric_token("metric.padding.md"),
                            )
                            .into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        |cx| {
                            let last = cx
                                .app
                                .models()
                                .read(&last_ime, |v| v.clone())
                                .unwrap_or_else(|_| Arc::<str>::from("IME: <error>"));

                            vec![
                                cx.text("IME smoke (Chinese IME)"),
                                cx.text("Target: Windows + Microsoft Pinyin (微软拼音)"),
                                cx.text("Type `nihao` while IME is active and verify inline preedit + candidate window positioning."),
                                cx.text(last),
                                cx.text("Single-line input"),
                                shadcn::Input::new(input_single)
                                    .a11y_label("IME single-line input")
                                    .into_element(cx),
                                cx.text("Multiline textarea"),
                                shadcn::Textarea::new(input_multi)
                                    .a11y_label("IME multiline textarea")
                                    .min_height(Px(160.0))
                                    .into_element(cx),
                                cx.text("Tips: While composing, Tab/Enter/Space/Escape should not trigger app shortcuts or focus traversal."),
                                cx.text("After commit/cancel, Tab focus traversal should work again."),
                            ]
                        },
                    )]
                },
            )]
            },
        );

        ui.set_root(root);
    }
}

impl WinitAppDriver for ImeSmokeDriver {
    type WindowState = ImeSmokeWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        ImeSmokeDriver::build_ui(app, window)
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
        if command.as_str() == "window.close" {
            app.push_effect(Effect::Window(fret_app::WindowRequest::Close(window)));
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
            app.push_effect(Effect::Window(fret_app::WindowRequest::Close(window)));
            return;
        }

        if let Event::Ime(ime) = event {
            let msg: Arc<str> = match ime {
                fret_core::ImeEvent::Enabled => Arc::from("IME: Enabled"),
                fret_core::ImeEvent::Disabled => Arc::from("IME: Disabled"),
                fret_core::ImeEvent::Commit(text) => Arc::from(format!("IME: Commit({text:?})")),
                fret_core::ImeEvent::Preedit { text, cursor } => {
                    Arc::from(format!("IME: Preedit(text={text:?}, cursor={cursor:?})"))
                }
                fret_core::ImeEvent::DeleteSurrounding {
                    before_bytes,
                    after_bytes,
                } => Arc::from(format!(
                    "IME: DeleteSurrounding(before_bytes={before_bytes}, after_bytes={after_bytes})"
                )),
            };
            let _ = app.models_mut().update(&state.last_ime, |v| *v = msg);
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
        ImeSmokeDriver::render(
            app,
            &mut state.ui,
            services,
            window,
            bounds,
            state.input_single.clone(),
            state.input_multi.clone(),
            state.last_ime.clone(),
        );

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
    ) -> Option<fret_launch::WindowCreateSpec> {
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

fn create_window_state(
    driver: &mut ImeSmokeDriver,
    app: &mut App,
    window: AppWindowId,
) -> ImeSmokeWindowState {
    <ImeSmokeDriver as WinitAppDriver>::create_window_state(driver, app, window)
}

fn hot_reload_window(
    driver: &mut ImeSmokeDriver,
    context: WinitHotReloadContext<'_, ImeSmokeWindowState>,
) {
    let WinitHotReloadContext {
        app,
        services,
        window,
        state,
    } = context;
    <ImeSmokeDriver as WinitAppDriver>::hot_reload_window(driver, app, services, window, state)
}

fn handle_model_changes(
    driver: &mut ImeSmokeDriver,
    context: WinitWindowContext<'_, ImeSmokeWindowState>,
    changed: &[fret_app::ModelId],
) {
    <ImeSmokeDriver as WinitAppDriver>::handle_model_changes(driver, context, changed)
}

fn handle_global_changes(
    driver: &mut ImeSmokeDriver,
    context: WinitWindowContext<'_, ImeSmokeWindowState>,
    changed: &[std::any::TypeId],
) {
    <ImeSmokeDriver as WinitAppDriver>::handle_global_changes(driver, context, changed)
}

fn handle_command(
    driver: &mut ImeSmokeDriver,
    context: WinitCommandContext<'_, ImeSmokeWindowState>,
    command: CommandId,
) {
    <ImeSmokeDriver as WinitAppDriver>::handle_command(driver, context, command)
}

fn handle_event(
    driver: &mut ImeSmokeDriver,
    context: WinitEventContext<'_, ImeSmokeWindowState>,
    event: &Event,
) {
    <ImeSmokeDriver as WinitAppDriver>::handle_event(driver, context, event)
}

fn render(driver: &mut ImeSmokeDriver, context: WinitRenderContext<'_, ImeSmokeWindowState>) {
    <ImeSmokeDriver as WinitAppDriver>::render(driver, context)
}

fn window_create_spec(
    driver: &mut ImeSmokeDriver,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    <ImeSmokeDriver as WinitAppDriver>::window_create_spec(driver, app, request)
}

fn window_created(
    driver: &mut ImeSmokeDriver,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
    new_window: AppWindowId,
) {
    <ImeSmokeDriver as WinitAppDriver>::window_created(driver, app, request, new_window)
}

fn build_fn_driver() -> FnDriver<ImeSmokeDriver, ImeSmokeWindowState> {
    FnDriver::new(
        ImeSmokeDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(|hooks| {
        hooks.hot_reload_window = Some(hot_reload_window);
        hooks.handle_model_changes = Some(handle_model_changes);
        hooks.handle_global_changes = Some(handle_global_changes);
        hooks.handle_command = Some(handle_command);
        hooks.window_create_spec = Some(window_create_spec);
        hooks.window_created = Some(window_created);
    })
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

    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![
                BindingV1 {
                    command: Some("focus.next".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("focus.previous".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("window.close".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyW".into(),
                    },
                },
            ],
        })
        .expect("valid keymap"),
    });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo ime_smoke_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(900.0, 720.0),
        ..Default::default()
    };

    let driver = build_fn_driver();
    crate::run_native_with_compat_driver(config, app, driver).context("run ime_smoke_demo app")
}
