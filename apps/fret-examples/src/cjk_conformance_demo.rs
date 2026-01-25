#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
#[cfg(not(target_arch = "wasm32"))]
use fret_bootstrap::BootstrapBuilder;
use fret_core::{AppWindowId, Event, FontId, Px, Rect, TextStyle, TextWrap, UiServices};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig,
};
use fret_runtime::{FontCatalogCache, PlatformCapabilities};
use fret_ui::declarative;
use fret_ui::{Theme, UiTree};
use fret_ui_shadcn::{self as shadcn, prelude::*};
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

struct CjkConformanceWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct CjkConformanceDriver;

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
            let theme = Theme::global(&*cx.app).clone();

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
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::Word,
                    overflow: fret_core::TextOverflow::Clip,
                })
            };

            let label_style = TextStyle {
                size: Px(12.0),
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

            for case in CJK_CASES {
                rows.push(cx.text_props(fret_ui::element::TextProps {
                    layout: Default::default(),
                    text: Arc::from(case.label),
                    style: Some(label_style.clone()),
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                }));

                rows.push(cx.text_props(fret_ui::element::TextProps {
                    layout: Default::default(),
                    text: Arc::from(case.text),
                    style: Some(sample_style.clone()),
                    color: Some(theme.color_required("foreground")),
                    wrap: TextWrap::Word,
                    overflow: fret_core::TextOverflow::Clip,
                }));
            }

            let header = shadcn::CardHeader::new([shadcn::CardTitle::new(
                "CJK Conformance (bootstrap fonts)",
            )
            .into_element(cx)])
            .into_element(cx);

            let scroll = shadcn::ScrollArea::new([ui::v_flex(cx, |_cx| rows)
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
                .max_w(MetricRef::Px(Px(960.0)))
                .into_element(cx);

            let page = ui::container(cx, |cx| {
                [ui::v_flex(cx, |_cx| [card])
                    .w_full()
                    .h_full()
                    .justify_center()
                    .items_center()
                    .into_element(cx)]
            })
            .bg(ColorRef::Color(theme.color_required("muted")))
            .p(Space::N6)
            .w_full()
            .h_full()
            .into_element(cx);

            vec![page]
        });

        state.ui.set_root(root);
        state.root = Some(root);
    }
}

impl WinitAppDriver for CjkConformanceDriver {
    type WindowState = CjkConformanceWindowState;

    fn create_window_state(&mut self, _app: &mut App, window: AppWindowId) -> Self::WindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        CjkConformanceWindowState { ui, root: None }
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
        _context: WinitCommandContext<'_, Self::WindowState>,
        _command: fret_app::CommandId,
    ) {
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
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
        shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo cjk_conformance_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    CjkConformanceDriver::default()
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

    BootstrapBuilder::new(app, CjkConformanceDriver)
        .configure(move |c| {
            *c = config;
        })
        .with_default_config_files()
        .context("load layered config files (settings/keymap)")?
        .with_lucide_icons()
        .run()
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
