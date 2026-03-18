use fret::advanced::prelude::{LocalState, TrackedStateExt as _};
use fret::advanced::view::{AppUiRenderRootState, render_root_with_app_ui};
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, FontId, Px, Rect, TextStyle, TextWrap, UiServices};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext, WinitHotReloadContext,
    WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::{FontCatalogCache, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::declarative;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::primitives::separator::Separator;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
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

pub struct EmojiConformanceWindowState {
    ui: UiTree<App>,
    app_ui_root: AppUiRenderRootState,
    emoji_font_override: LocalState<Option<Arc<str>>>,
    emoji_font_override_open: LocalState<bool>,
}

#[derive(Default)]
pub struct EmojiConformanceDriver;

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

        let root = render_root_with_app_ui(
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds),
            "emoji-conformance",
            &mut state.app_ui_root,
            |cx| {
                let theme = cx.theme_snapshot();

                let available_fonts = cx
                    .app
                    .global::<FontCatalogCache>()
                    .cloned()
                    .unwrap_or_default()
                    .families_arc();

                let selected_emoji_font = emoji_font_override.layout(cx).value_or_default();

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
                    let has_known_color_emoji_font =
                        ["Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji"]
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
                        color: Some(theme.color_token("muted-foreground")),
                        align: fret_core::TextAlign::Start,
                        wrap: TextWrap::Word,
                        overflow: fret_core::TextOverflow::Clip,
                        ink_overflow: Default::default(),
                    })
                };

                let controls = ui::h_flex(|cx| {
                    [
                        shadcn::Select::new(
                            emoji_font_override.clone_model(),
                            emoji_font_override_open.clone_model(),
                        )
                        .value(
                            shadcn::SelectValue::new().placeholder("Force emoji font (optional)"),
                        )
                        .items(items)
                        .ui()
                        .w_px(Px(280.0))
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
                        color: Some(theme.color_token("muted-foreground")),
                        align: fret_core::TextAlign::Start,
                        wrap: TextWrap::None,
                        overflow: fret_core::TextOverflow::Clip,
                        ink_overflow: Default::default(),
                    }));

                    rows.push(cx.text_props(fret_ui::element::TextProps {
                        layout: Default::default(),
                        text: Arc::from(case.text),
                        style: Some(emoji_style.clone()),
                        color: Some(theme.color_token("foreground")),
                        align: fret_core::TextAlign::Start,
                        wrap: TextWrap::Word,
                        overflow: fret_core::TextOverflow::Clip,
                        ink_overflow: Default::default(),
                    }));
                }

                if let Some(name) = selected_emoji_font.as_deref() {
                    let forced = format!("(forced) {name}: 😀 😺 🧑‍💻 ❤️ 👨‍👩‍👧‍👦 🇯🇵 🏳️‍🌈");
                    rows.push(Separator::new().into_element(cx));
                    rows.push(cx.text_props(fret_ui::element::TextProps {
                        layout: Default::default(),
                        text: Arc::from("Emoji-only line forced to the selected family:"),
                        style: Some(label_style.clone()),
                        color: Some(theme.color_token("muted-foreground")),
                        align: fret_core::TextAlign::Start,
                        wrap: TextWrap::None,
                        overflow: fret_core::TextOverflow::Clip,
                        ink_overflow: Default::default(),
                    }));
                    rows.push(cx.text_props(fret_ui::element::TextProps {
                        layout: Default::default(),
                        text: Arc::from(forced),
                        style: Some(TextStyle {
                            font: FontId::family(name),
                            ..emoji_style
                        }),
                        color: Some(theme.color_token("foreground")),
                        align: fret_core::TextAlign::Start,
                        wrap: TextWrap::Word,
                        overflow: fret_core::TextOverflow::Clip,
                        ink_overflow: Default::default(),
                    }));
                }

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

                ui::children![cx; emoji_conformance_page(cx, theme, card)].into()
            },
        );

        state.ui.set_root(root);
    }
}

fn emoji_conformance_page<C>(
    cx: &mut fret_ui::ElementContext<'_, App>,
    theme: fret_ui::ThemeSnapshot,
    card: C,
) -> impl fret_ui_kit::IntoUiElement<App> + use<C>
where
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
    .into_element(cx)
}

fn create_window_state(
    _driver: &mut EmojiConformanceDriver,
    app: &mut App,
    window: AppWindowId,
) -> EmojiConformanceWindowState {
    let emoji_font_override = LocalState::from_model(app.models_mut().insert(None::<Arc<str>>));
    let emoji_font_override_open = LocalState::from_model(app.models_mut().insert(false));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    EmojiConformanceWindowState {
        ui,
        app_ui_root: AppUiRenderRootState::default(),
        emoji_font_override,
        emoji_font_override_open,
    }
}

fn hot_reload_window(
    _driver: &mut EmojiConformanceDriver,
    context: WinitHotReloadContext<'_, EmojiConformanceWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
}

fn handle_command(
    _driver: &mut EmojiConformanceDriver,
    context: WinitCommandContext<'_, EmojiConformanceWindowState>,
    command: CommandId,
) {
    let WinitCommandContext { app, state, .. } = context;

    if command.as_str() == CMD_EMOJI_FONT_RESET {
        let _ = state.emoji_font_override.set_in(app.models_mut(), None);
    }
}

fn handle_event(
    _driver: &mut EmojiConformanceDriver,
    context: WinitEventContext<'_, EmojiConformanceWindowState>,
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
    _driver: &mut EmojiConformanceDriver,
    context: WinitRenderContext<'_, EmojiConformanceWindowState>,
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
    _driver: &mut EmojiConformanceDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    _driver: &mut EmojiConformanceDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
    _new_window: AppWindowId,
) {
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<EmojiConformanceDriver, EmojiConformanceWindowState>,
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
        main_window_title: "fret-demo emoji_conformance_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> FnDriver<EmojiConformanceDriver, EmojiConformanceWindowState> {
    FnDriver::new(
        EmojiConformanceDriver::default(),
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
        EmojiConformanceDriver::default(),
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
