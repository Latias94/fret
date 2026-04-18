use anyhow::Context as _;
use fret::advanced::prelude::LocalState;
use fret::advanced::view::{AppUiRenderRootState, render_root_with_app_ui};
use fret::app::RenderContextAccess as _;
use fret_app::{App, CommandId, Effect};
use fret_core::{AppWindowId, Event, Px, Rect, UiServices};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext, WinitHotReloadContext,
    WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::{
    BindingV1, KeySpecV1, Keymap, KeymapFileV1, KeymapService, PlatformCapabilities,
};
use fret_ui::UiTree;
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;
pub struct ImeSmokeWindowState {
    ui: UiTree<App>,
    app_ui_root: AppUiRenderRootState,
    locals: Option<ImeSmokeLocals>,
}

#[derive(Clone)]
struct ImeSmokeLocals {
    input_single: LocalState<String>,
    input_multi: LocalState<String>,
    last_ime: LocalState<Arc<str>>,
}

impl ImeSmokeLocals {
    fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {
        Self {
            input_single: cx.state().local::<String>(),
            input_multi: cx.state().local::<String>(),
            last_ime: cx.state().local_init(|| Arc::<str>::from("IME: <none>")),
        }
    }
}

#[derive(Default)]
pub struct ImeSmokeDriver;

impl ImeSmokeDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> ImeSmokeWindowState {
        let mut ui = UiTree::new();
        ui.set_window(window);

        ImeSmokeWindowState {
            ui,
            app_ui_root: AppUiRenderRootState::default(),
            locals: None,
        }
    }

    fn render(
        app: &mut App,
        ui: &mut UiTree<App>,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        app_ui_root: &mut AppUiRenderRootState,
        locals: &mut Option<ImeSmokeLocals>,
    ) {
        let root = render_root_with_app_ui(
            declarative::RenderRootContext::new(ui, app, services, window, bounds),
            "ime-smoke",
            app_ui_root,
            |cx| {
                if locals.is_none() {
                    *locals = Some(ImeSmokeLocals::new(cx));
                }
                let ImeSmokeLocals {
                    input_single,
                    input_multi,
                    last_ime,
                } = locals.as_ref().expect("IME locals should exist").clone();
                let theme = cx.theme_snapshot();
                let last = last_ime.paint_value(cx);
                let cx = cx.elements();

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
                                vec![
                                    cx.text("IME smoke (Chinese IME)"),
                                    cx.text("Target: Windows + Microsoft Pinyin (微软拼音)"),
                                    cx.text("Type `nihao` while IME is active and verify inline preedit + candidate window positioning."),
                                    cx.text(last),
                                    cx.text("Single-line input"),
                                    shadcn::Input::new(&input_single)
                                        .a11y_label("IME single-line input")
                                        .into_element(cx),
                                    cx.text("Multiline textarea"),
                                    shadcn::Textarea::new(&input_multi)
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
                .into()
            },
        );

        ui.set_root(root);
    }
}

fn create_window_state(
    _driver: &mut ImeSmokeDriver,
    app: &mut App,
    window: AppWindowId,
) -> ImeSmokeWindowState {
    ImeSmokeDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut ImeSmokeDriver,
    context: WinitHotReloadContext<'_, ImeSmokeWindowState>,
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
    _driver: &mut ImeSmokeDriver,
    context: WinitWindowContext<'_, ImeSmokeWindowState>,
    changed: &[fret_app::ModelId],
) {
    context
        .state
        .ui
        .propagate_model_changes(context.app, changed);
}

fn handle_global_changes(
    _driver: &mut ImeSmokeDriver,
    context: WinitWindowContext<'_, ImeSmokeWindowState>,
    changed: &[std::any::TypeId],
) {
    context
        .state
        .ui
        .propagate_global_changes(context.app, changed);
}

fn handle_command(
    _driver: &mut ImeSmokeDriver,
    context: WinitCommandContext<'_, ImeSmokeWindowState>,
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

fn handle_event(
    _driver: &mut ImeSmokeDriver,
    context: WinitEventContext<'_, ImeSmokeWindowState>,
    event: &Event,
) {
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
        if let Some(locals) = state.locals.as_ref() {
            let _ = locals.last_ime.set_in(app.models_mut(), msg);
        }
    }

    state.ui.dispatch_event(app, services, event);
}

fn render(_driver: &mut ImeSmokeDriver, context: WinitRenderContext<'_, ImeSmokeWindowState>) {
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
        &mut state.app_ui_root,
        &mut state.locals,
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
    _driver: &mut ImeSmokeDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    _driver: &mut ImeSmokeDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
    _new_window: AppWindowId,
) {
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<ImeSmokeDriver, ImeSmokeWindowState>,
) {
    hooks.hot_reload_window = Some(hot_reload_window);
    hooks.handle_model_changes = Some(handle_model_changes);
    hooks.handle_global_changes = Some(handle_global_changes);
    hooks.handle_command = Some(handle_command);
    hooks.window_create_spec = Some(window_create_spec);
    hooks.window_created = Some(window_created);
}

pub fn build_fn_driver() -> FnDriver<ImeSmokeDriver, ImeSmokeWindowState> {
    FnDriver::new(
        ImeSmokeDriver::default(),
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

    crate::run_native_with_fn_driver_with_hooks(
        config,
        app,
        ImeSmokeDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
    .context("run ime_smoke_demo app")
}
