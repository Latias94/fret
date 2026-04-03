use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, FontId, Px, Rect, TextStyle, TextWrap, UiServices};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext, WinitHotReloadContext,
    WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::{FontCatalogCache, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::declarative;
use fret_ui_kit::IntoUiElementInExt as _;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
struct CjkCase {
    label: &'static str,
    text: &'static str,
}

const CJK_CASES: &[CjkCase] = &[
    CjkCase {
        label: "Chinese (Simplified)",
        text: "你好，世界！这是一段用于验证排版、换行与标点处理的文本：12345，（）《》“”……",
    },
    CjkCase {
        label: "Japanese",
        text: "日本語のテキストです。禁則処理、句読点、全角スペース、そして長い文章の折り返しを確認します。",
    },
    CjkCase {
        label: "Korean",
        text: "한국어 텍스트입니다. 자간/줄바꿈/문장부호 처리를 확인하기 위한 문장입니다.",
    },
    CjkCase {
        label: "Mixed",
        text: "Text + 中文 + 日本語 + 한국어 + 😀 ✈️ 1️⃣ 🇺🇸 👨‍👩‍👧‍👦",
    },
];

const FALLBACK_SMOKE: &str =
    "Default font fallback: Text + 中文 + 日本語 + 한국어 + 😀 ✈️ 1️⃣ 🇺🇸 👨‍👩‍👧‍👦";

pub struct CjkConformanceWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
pub struct CjkConformanceDriver;

impl CjkConformanceDriver {
    fn render(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut CjkConformanceWindowState,
        bounds: Rect,
    ) {
        let root = declarative::RenderRootContext::new(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
        )
        .render_root("cjk-conformance", |cx| {
            let theme = cx.theme_snapshot();

            let available_fonts = cx
                .app
                .global::<FontCatalogCache>()
                .cloned()
                .unwrap_or_default()
                .families_arc();

            let cjk_family = "Noto Sans CJK SC";
            let cjk_available = available_fonts
                .iter()
                .any(|n| n.as_ref().eq_ignore_ascii_case(cjk_family));

            let status = {
                let text = if cjk_available {
                    format!("{cjk_family}: available (try toggling WASM feature `cjk-lite-fonts`)")
                } else {
                    format!(
                        "{cjk_family}: missing (WASM needs bundled fonts; enable `cjk-lite-fonts`)"
                    )
                };

                cx.text_props(fret_ui::element::TextProps {
                    layout: Default::default(),
                    text: Arc::from(text),
                    style: Some(TextStyle {
                        size: Px(12.0),
                        ..Default::default()
                    }),
                    color: Some(theme.color_token("muted-foreground")),
                    align: fret_core::TextAlign::Start,
                    wrap: TextWrap::Word,
                    overflow: fret_core::TextOverflow::Clip,
                    ink_overflow: Default::default(),
                })
            };

            let label_style = TextStyle {
                size: Px(12.0),
                ..Default::default()
            };
            let default_style = TextStyle {
                size: Px(20.0),
                ..Default::default()
            };
            let sample_style = TextStyle {
                font: FontId::family(cjk_family),
                size: Px(20.0),
                ..Default::default()
            };

            let mut rows: Vec<fret_ui::element::AnyElement> =
                Vec::with_capacity(CJK_CASES.len() * 2 + 2);
            rows.push(status);
            rows.push(cx.text_props(fret_ui::element::TextProps {
                layout: Default::default(),
                text: Arc::from("Fallback smoke (default style):"),
                style: Some(label_style.clone()),
                color: Some(theme.color_token("muted-foreground")),
                align: fret_core::TextAlign::Start,
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            }));
            rows.push(cx.text_props(fret_ui::element::TextProps {
                layout: Default::default(),
                text: Arc::from(FALLBACK_SMOKE),
                style: Some(default_style),
                color: Some(theme.color_token("foreground")),
                align: fret_core::TextAlign::Start,
                wrap: TextWrap::Word,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            }));
            rows.push(cx.text_props(fret_ui::element::TextProps {
                layout: Default::default(),
                text: Arc::from("Fallback smoke (forced CJK family):"),
                style: Some(label_style.clone()),
                color: Some(theme.color_token("muted-foreground")),
                align: fret_core::TextAlign::Start,
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            }));
            rows.push(cx.text_props(fret_ui::element::TextProps {
                layout: Default::default(),
                text: Arc::from(FALLBACK_SMOKE),
                style: Some(sample_style.clone()),
                color: Some(theme.color_token("foreground")),
                align: fret_core::TextAlign::Start,
                wrap: TextWrap::Word,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            }));

            for case in CJK_CASES {
                rows.push(cx.text_props(fret_ui::element::TextProps {
                    layout: Default::default(),
                    text: Arc::from(case.label),
                    style: Some(label_style.clone()),
                    color: Some(theme.color_token("muted-foreground")),
                    align: fret_core::TextAlign::Start,
                    wrap: TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                    ink_overflow: Default::default(),
                }));

                rows.push(cx.text_props(fret_ui::element::TextProps {
                    layout: Default::default(),
                    text: Arc::from(case.text),
                    style: Some(sample_style.clone()),
                    color: Some(theme.color_token("foreground")),
                    align: fret_core::TextAlign::Start,
                    wrap: TextWrap::Word,
                    overflow: fret_core::TextOverflow::Clip,
                    ink_overflow: Default::default(),
                }));
            }

            let header = shadcn::CardHeader::new([shadcn::CardTitle::new(
                "CJK Conformance (bootstrap fonts)",
            )
            .into_element(cx)])
            .into_element(cx);

            let scroll = shadcn::ScrollArea::new([ui::v_flex(|_cx| rows)
                .w_full()
                .gap(Space::N2)
                .items_start()
                .into_element(cx)])
            .ui()
            .w_full()
            .h_full()
            .into_element(cx);

            let content = shadcn::CardContent::new([scroll]).into_element(cx);
            let card = shadcn::Card::new([header, content])
                .ui()
                .w_full()
                .h_full()
                .max_w(Px(960.0))
                .into_element(cx);

            ui::children![cx; cjk_conformance_page(cx, theme, card)]
        });

        state.ui.set_root(root);
        state.root = Some(root);
    }
}

fn cjk_conformance_page<'a, Cx, C>(
    cx: &mut Cx,
    theme: fret_ui::ThemeSnapshot,
    card: C,
) -> impl fret_ui_kit::IntoUiElement<App> + use<Cx, C>
where
    Cx: fret_ui::ElementContextAccess<'a, App>,
    C: fret_ui_kit::IntoUiElement<App>,
{
    ui::container(move |cx| {
        ui::single(
            cx,
            ui::v_flex(move |cx| ui::single(cx, card))
                .w_full()
                .h_full()
                .justify_center()
                .items_center(),
        )
    })
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element_in(cx)
}

fn create_window_state(
    _driver: &mut CjkConformanceDriver,
    _app: &mut App,
    window: AppWindowId,
) -> CjkConformanceWindowState {
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    CjkConformanceWindowState { ui, root: None }
}

fn hot_reload_window(
    _driver: &mut CjkConformanceDriver,
    context: WinitHotReloadContext<'_, CjkConformanceWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_command(
    _driver: &mut CjkConformanceDriver,
    _context: WinitCommandContext<'_, CjkConformanceWindowState>,
    _command: fret_app::CommandId,
) {
}

fn handle_event(
    _driver: &mut CjkConformanceDriver,
    context: WinitEventContext<'_, CjkConformanceWindowState>,
    event: &Event,
) {
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

fn render(
    _driver: &mut CjkConformanceDriver,
    context: WinitRenderContext<'_, CjkConformanceWindowState>,
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

    CjkConformanceDriver::render(app, services, window, state, bounds);

    state.ui.request_semantics_snapshot();
    state.ui.ingest_paint_cache_source(scene);
    scene.clear();
    let mut frame =
        fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
    frame.layout_all();
    frame.paint_all(scene);
}

fn window_create_spec(
    _driver: &mut CjkConformanceDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    _driver: &mut CjkConformanceDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
    _new_window: AppWindowId,
) {
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<CjkConformanceDriver, CjkConformanceWindowState>,
) {
    hooks.hot_reload_window = Some(hot_reload_window);
    hooks.handle_command = Some(handle_command);
    hooks.window_create_spec = Some(window_create_spec);
    hooks.window_created = Some(window_created);
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Zinc,
        shadcn::themes::ShadcnColorScheme::Dark,
    );
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo cjk_conformance_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> FnDriver<CjkConformanceDriver, CjkConformanceWindowState> {
    FnDriver::new(
        CjkConformanceDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
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

    crate::run_native_with_fn_driver_with_hooks(
        config,
        app,
        CjkConformanceDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
    .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
