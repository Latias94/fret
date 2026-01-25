#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
#[cfg(not(target_arch = "wasm32"))]
use fret_bootstrap::BootstrapBuilder;
use fret_core::{AppWindowId, Event, FontId, Px, Rect, TextStyle, TextWrap, UiServices};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig,
};
use fret_runtime::{FontCatalogCache, PlatformCapabilities};
use fret_ui::declarative;
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_kit::primitives::separator::Separator;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::collections::HashSet;
use std::sync::Arc;

const CMD_EMOJI_FONT_RESET: &str = "emoji_conformance.emoji_font.reset";

#[derive(Debug, Clone, Copy)]
struct EmojiCase {
    label: &'static str,
    text: &'static str,
}

const EMOJI_CASES: &[EmojiCase] = &[
    EmojiCase {
        label: "Basic",
        text: "😀 😺 🎉 ✅ ❌",
    },
    EmojiCase {
        label: "ZWJ (profession)",
        text: "🧑‍💻 👩‍🚀 🧑‍🍳 👨‍🔧",
    },
    EmojiCase {
        label: "Family (ZWJ)",
        text: "👨‍👩‍👧‍👦 👩‍👩‍👦 👨‍👨‍👧‍👧",
    },
    EmojiCase {
        label: "Flags (regional indicators)",
        text: "🇯🇵 🇺🇸 🇪🇺 🇨🇳",
    },
    EmojiCase {
        label: "Variation selectors",
        text: "❤︎ ❤️ ☺︎ ☺️",
    },
    EmojiCase {
        label: "Skin tone modifiers",
        text: "👍 👍🏽 👋🏿 🤝🏽",
    },
    EmojiCase {
        label: "Keycaps",
        text: "1️⃣ 2️⃣ #️⃣ *️⃣",
    },
    EmojiCase {
        label: "ZWJ flag",
        text: "🏳️‍🌈 🏴‍☠️",
    },
    EmojiCase {
        label: "Mixed scripts",
        text: "Text + emoji 😀 + 日本語 + 😀 + العربية + 😀",
    },
];

struct EmojiConformanceWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    emoji_font_override: Model<Option<Arc<str>>>,
    emoji_font_override_open: Model<bool>,
}

#[derive(Default)]
struct EmojiConformanceDriver;

impl EmojiConformanceDriver {
    fn render(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut EmojiConformanceWindowState,
        bounds: Rect,
    ) {
        let emoji_font_override = state.emoji_font_override.clone();
        let emoji_font_override_open = state.emoji_font_override_open.clone();

        let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
            .render_root("emoji-conformance", |cx| {
                cx.observe_model(&emoji_font_override, Invalidation::Layout);
                cx.observe_model(&emoji_font_override_open, Invalidation::Layout);

                let theme = Theme::global(&*cx.app).clone();

                let available_fonts = cx
                    .app
                    .global::<FontCatalogCache>()
                    .cloned()
                    .unwrap_or_default()
                    .families_arc();

                let selected_emoji_font = cx
                    .app
                    .models()
                    .read(&emoji_font_override, |v| v.clone())
                    .ok()
                    .flatten();

                let mut items: Vec<shadcn::SelectItem> = Vec::new();
                let mut seen: HashSet<Arc<str>> = HashSet::new();

                for preferred in ["Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji"] {
                    let Some(found) = available_fonts
                        .iter()
                        .find(|n| n.as_ref().eq_ignore_ascii_case(preferred))
                    else {
                        continue;
                    };
                    let name: Arc<str> = found.clone();
                    if seen.insert(name.clone()) {
                        items.push(shadcn::SelectItem::new(name.clone(), name));
                    }
                }

                let title = shadcn::CardTitle::new("Emoji conformance").into_element(cx);
                let subtitle = shadcn::CardDescription::new(
                    "Verify color emoji rendering, ZWJ sequences, VS16, keycaps, and flags.",
                )
                .into_element(cx);

                let header = shadcn::CardHeader::new([title, subtitle]).into_element(cx);

                let status_line = {
                    let has_known_color_emoji_font = ["Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji"]
                        .iter()
                        .any(|n| {
                            available_fonts
                                .iter()
                                .any(|f| f.as_ref().eq_ignore_ascii_case(n))
                        });

                    let msg: Arc<str> = if has_known_color_emoji_font {
                        Arc::from("Status: emoji color font detected in catalog.")
                    } else {
                        Arc::from(
                            "Status: no known color emoji font in catalog (you may see tofu or monochrome).",
                        )
                    };

                    cx.text_props(fret_ui::element::TextProps {
                        layout: Default::default(),
                        text: msg,
                        style: None,
                        color: Some(theme.color_required("muted-foreground")),
                        wrap: TextWrap::Word,
                        overflow: fret_core::TextOverflow::Clip,
                    })
                };

                let controls = ui::h_flex(cx, |cx| {
                    [
                        shadcn::Select::new(
                            emoji_font_override.clone(),
                            emoji_font_override_open.clone(),
                        )
                        .placeholder("Force emoji font (optional)")
                        .items(items)
                        .ui()
                        .w_px(MetricRef::Px(Px(280.0)))
                        .into_element(cx),
                        shadcn::Button::new("Reset")
                            .variant(shadcn::ButtonVariant::Outline)
                            .on_click(CommandId::new(CMD_EMOJI_FONT_RESET))
                            .into_element(cx),
                    ]
                })
                .w_full()
                .gap(Space::N3)
                .items_center()
                .into_element(cx);

                let label_style = TextStyle {
                    size: Px(12.0),
                    ..Default::default()
                };
                let emoji_style = TextStyle {
                    size: Px(28.0),
                    ..Default::default()
                };

                let mut rows: Vec<AnyElement> = Vec::with_capacity(EMOJI_CASES.len() * 2 + 6);

                rows.push(status_line);
                rows.push(controls);
                rows.push(Separator::new().into_element(cx));

                for case in EMOJI_CASES {
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
                        style: Some(emoji_style.clone()),
                        color: Some(theme.color_required("foreground")),
                        wrap: TextWrap::Word,
                        overflow: fret_core::TextOverflow::Clip,
                    }));
                }

                if let Some(name) = selected_emoji_font.as_deref() {
                    let forced = format!("(forced) {name}: 😀 😺 🧑‍💻 ❤️ 👨‍👩‍👧‍👦 🇯🇵 🏳️‍🌈");
                    rows.push(Separator::new().into_element(cx));
                    rows.push(cx.text_props(fret_ui::element::TextProps {
                        layout: Default::default(),
                        text: Arc::from("Emoji-only line forced to the selected family:"),
                        style: Some(label_style.clone()),
                        color: Some(theme.color_required("muted-foreground")),
                        wrap: TextWrap::None,
                        overflow: fret_core::TextOverflow::Clip,
                    }));
                    rows.push(cx.text_props(fret_ui::element::TextProps {
                        layout: Default::default(),
                        text: Arc::from(forced),
                        style: Some(TextStyle {
                            font: FontId::family(name),
                            ..emoji_style
                        }),
                        color: Some(theme.color_required("foreground")),
                        wrap: TextWrap::Word,
                        overflow: fret_core::TextOverflow::Clip,
                    }));
                }

                let scroll = shadcn::ScrollArea::new([
                    ui::v_flex(cx, |_cx| rows)
                        .w_full()
                        .gap(Space::N2)
                        .items_start()
                        .into_element(cx),
                ])
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

impl WinitAppDriver for EmojiConformanceDriver {
    type WindowState = EmojiConformanceWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let emoji_font_override = app.models_mut().insert(None::<Arc<str>>);
        let emoji_font_override_open = app.models_mut().insert(false);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        EmojiConformanceWindowState {
            ui,
            root: None,
            emoji_font_override,
            emoji_font_override_open,
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
        let WinitCommandContext { app, state, .. } = context;

        if command.as_str() == CMD_EMOJI_FONT_RESET {
            let _ = app
                .models_mut()
                .update(&state.emoji_font_override, |v| *v = None);
        }
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

        EmojiConformanceDriver::render(app, services, window, state, bounds);

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
        main_window_title: "fret-demo emoji_conformance_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    EmojiConformanceDriver::default()
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

    BootstrapBuilder::new(app, EmojiConformanceDriver)
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
