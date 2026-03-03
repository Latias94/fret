use std::sync::Arc;

use fret::prelude::*;
use fret_app::{CommandId, CommandMeta, Effect, WindowRequest};
use fret_bootstrap::ui_app_with_hooks;
use fret_core::{AppWindowId, Color, Px};
use fret_render::WgpuAdapterSelectionSnapshot;
use fret_runtime::{
    PlatformCapabilities, RunnerWindowStyleDiagnosticsStore, WindowBackgroundMaterialRequest,
    WindowDecorationsRequest, WindowStyleRequest,
};

const CMD_TO_NONE: &str = "cookbook.utility_window_materials.to_none.v1";
const CMD_TO_MICA: &str = "cookbook.utility_window_materials.to_mica.v1";
const CMD_TO_ACRYLIC: &str = "cookbook.utility_window_materials.to_acrylic.v1";
const CMD_TO_VIBRANCY: &str = "cookbook.utility_window_materials.to_vibrancy.v1";
const CMD_QUIT: &str = "cookbook.utility_window_materials.quit.v1";

// Intentionally matches the workstream gate test IDs (so scripts can be reused).
const TEST_ID_ROOT: &str = "utility-window.materials.root";
const TEST_ID_TO_NONE: &str = "utility-window.materials.to_none";
const TEST_ID_TO_MICA: &str = "utility-window.materials.to_mica";
const TEST_ID_TO_ACRYLIC: &str = "utility-window.materials.to_acrylic";
const TEST_ID_TO_VIBRANCY: &str = "utility-window.materials.to_vibrancy";
const TEST_ID_STYLE_TEXT: &str = "utility-window.materials.style_effective";
const TEST_ID_PLATFORM_TEXT: &str = "utility-window.materials.platform";

#[derive(Debug)]
struct State {
    window: AppWindowId,
    status: fret_app::Model<Arc<str>>,
}

fn init_window(app: &mut fret_app::App, window: AppWindowId) -> State {
    State {
        window,
        status: app.models_mut().insert(Arc::from("Idle")),
    }
}

fn configure_driver(
    driver: fret_bootstrap::ui_app_driver::UiAppDriver<State>,
) -> fret_bootstrap::ui_app_driver::UiAppDriver<State> {
    driver.on_command(on_command)
}

fn on_command(
    app: &mut fret_app::App,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<fret_app::App>,
    st: &mut State,
    command: &CommandId,
) {
    let material = match command.as_str() {
        CMD_TO_NONE => WindowBackgroundMaterialRequest::None,
        CMD_TO_MICA => WindowBackgroundMaterialRequest::Mica,
        CMD_TO_ACRYLIC => WindowBackgroundMaterialRequest::Acrylic,
        CMD_TO_VIBRANCY => WindowBackgroundMaterialRequest::Vibrancy,
        CMD_QUIT => {
            app.push_effect(Effect::QuitApp);
            return;
        }
        _ => return,
    };

    app.push_effect(Effect::Window(WindowRequest::SetStyle {
        window,
        style: WindowStyleRequest {
            background_material: Some(material),
            ..Default::default()
        },
    }));

    let _ = app.models_mut().update(&st.status, |v| {
        *v = Arc::from(format!("Requested: {material:?}"));
    });
    app.request_redraw(window);
}

fn view(cx: &mut fret_ui::ElementContext<'_, fret_app::App>, st: &mut State) -> ViewElements {
    let is_windows = cfg!(target_os = "windows");
    let is_macos = cfg!(target_os = "macos");

    let theme = cx.theme().snapshot();
    let color_muted_foreground = theme.color_token("muted-foreground");

    let status = cx
        .app
        .models()
        .read(&st.status, |v| v.clone())
        .unwrap_or_else(|_| Arc::from("Idle"));

    let caps = cx.app.global::<PlatformCapabilities>().cloned();
    let caps_text: Arc<str> = Arc::from(match caps {
        Some(caps) if is_windows => format!(
            "caps: mica={} acrylic={} system_default={} transparent={}",
            caps.ui.window_background_material_mica,
            caps.ui.window_background_material_acrylic,
            caps.ui.window_background_material_system_default,
            caps.ui.window_transparent,
        ),
        Some(caps) if is_macos => format!(
            "caps: vibrancy={} transparent={}",
            caps.ui.window_background_material_vibrancy, caps.ui.window_transparent,
        ),
        Some(caps) => format!("caps: transparent={}", caps.ui.window_transparent),
        None => "caps: <unavailable>".to_string(),
    });

    let wgpu = cx.app.global::<WgpuAdapterSelectionSnapshot>().cloned();
    let wgpu_text: Arc<str> = Arc::from(match wgpu {
        Some(s) => format!(
            "wgpu: backend={} adapter={}",
            s.selected_backend, s.adapter_name
        ),
        None => "wgpu: <unavailable>".to_string(),
    });

    let effective_style = cx
        .app
        .global::<RunnerWindowStyleDiagnosticsStore>()
        .and_then(|store| store.effective_snapshot(st.window));
    let style_text: Arc<str> = Arc::from(match effective_style {
        Some(s) => format!(
            "effective: decorations={:?} resizable={} transparent={} material={:?}",
            s.decorations, s.resizable, s.transparent, s.background_material
        ),
        None => "effective: <unavailable>".to_string(),
    });

    let platform_text: Arc<str> = Arc::from(if is_windows {
        "platform: windows (mica/acrylic + implied transparency)".to_string()
    } else if is_macos {
        "platform: macos (vibrancy + implied transparency)".to_string()
    } else {
        format!(
            "platform: {} (window materials best-effort)",
            std::env::consts::OS
        )
    });

    let title = if is_windows {
        "Utility window materials + transparency (Windows)"
    } else if is_macos {
        "Utility window vibrancy + transparency (macOS)"
    } else {
        "Utility window style diagnostics"
    };

    let description = if is_windows {
        "Requests Mica/Acrylic via WindowStyleRequest and asserts effective/clamped results via diagnostics."
    } else if is_macos {
        "Requests Vibrancy via WindowStyleRequest and asserts effective/clamped results via diagnostics."
    } else {
        "Reports effective/clamped window style via diagnostics."
    };

    let header = shadcn::CardHeader::new(vec![
        shadcn::CardTitle::new(title).into_element(cx),
        shadcn::CardDescription::new(description).into_element(cx),
    ])
    .into_element(cx);

    let content = ui::v_flex(cx, |cx| {
        let platform_line = ui::text(cx, platform_text)
            .font_monospace()
            .text_sm()
            .text_color(ColorRef::Color(color_muted_foreground))
            .into_element(cx)
            .test_id(TEST_ID_PLATFORM_TEXT);
        let style_line = ui::text(cx, style_text)
            .font_monospace()
            .text_sm()
            .into_element(cx)
            .test_id(TEST_ID_STYLE_TEXT);
        let caps_line = ui::text(cx, caps_text)
            .font_monospace()
            .text_sm()
            .text_color(ColorRef::Color(color_muted_foreground))
            .into_element(cx);
        let wgpu_line = ui::text(cx, wgpu_text)
            .font_monospace()
            .text_sm()
            .text_color(ColorRef::Color(color_muted_foreground))
            .into_element(cx);
        let status_line = ui::text(cx, status)
            .text_sm()
            .text_color(ColorRef::Color(color_muted_foreground))
            .into_element(cx);

        let buttons = ui::h_flex(cx, |cx| {
            if is_windows {
                vec![
                    shadcn::Button::new("None")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from(CMD_TO_NONE))
                        .test_id(TEST_ID_TO_NONE)
                        .into_element(cx),
                    shadcn::Button::new("Mica")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from(CMD_TO_MICA))
                        .test_id(TEST_ID_TO_MICA)
                        .into_element(cx),
                    shadcn::Button::new("Acrylic")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from(CMD_TO_ACRYLIC))
                        .test_id(TEST_ID_TO_ACRYLIC)
                        .into_element(cx),
                    shadcn::Button::new("Quit")
                        .variant(shadcn::ButtonVariant::Destructive)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from(CMD_QUIT))
                        .into_element(cx),
                ]
            } else if is_macos {
                vec![
                    shadcn::Button::new("None")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from(CMD_TO_NONE))
                        .test_id(TEST_ID_TO_NONE)
                        .into_element(cx),
                    shadcn::Button::new("Vibrancy")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from(CMD_TO_VIBRANCY))
                        .test_id(TEST_ID_TO_VIBRANCY)
                        .into_element(cx),
                    shadcn::Button::new("Quit")
                        .variant(shadcn::ButtonVariant::Destructive)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from(CMD_QUIT))
                        .into_element(cx),
                ]
            } else {
                vec![
                    shadcn::Button::new("Quit")
                        .variant(shadcn::ButtonVariant::Destructive)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from(CMD_QUIT))
                        .into_element(cx),
                ]
            }
        })
        .gap(Space::N2)
        .into_element(cx);

        [
            platform_line,
            style_line,
            caps_line,
            wgpu_line,
            status_line,
            buttons,
        ]
    })
    .gap(Space::N3)
    .into_element(cx);

    let surface = shadcn::Card::new(vec![
        header,
        shadcn::CardContent::new(vec![content]).into_element(cx),
    ])
    .ui()
    .w_full()
    .max_w(Px(760.0))
    .into_element(cx);

    // Keep the window background empty so the OS material can show through where not covered by UI.
    // (Most of the content lives inside the centered Card.)
    ui::container(cx, |cx| {
        [ui::v_flex(cx, |_cx| [surface])
            .items_center()
            .justify_center()
            .size_full()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(Color::TRANSPARENT))
    .p(Space::N6)
    .size_full()
    .into_element(cx)
    .test_id(TEST_ID_ROOT)
    .into()
}

fn main() -> anyhow::Result<()> {
    ui_app_with_hooks(
        "cookbook-utility-window-materials-windows",
        init_window,
        view,
        configure_driver,
    )
    .with_default_diagnostics()
    .with_main_window("utility_window_materials_windows", (760.0, 520.0))
    .configure(|config| {
        let mut style = WindowStyleRequest {
            decorations: Some(WindowDecorationsRequest::None),
            resizable: Some(true),
            ..Default::default()
        };

        if cfg!(target_os = "windows") {
            // Intentionally omit `transparent`: allow ADR 0310 implicit transparency when a
            // composited OS material is effectively applied.
            style.background_material = Some(WindowBackgroundMaterialRequest::Mica);
        } else if cfg!(target_os = "macos") {
            // Intentionally omit `transparent`: allow ADR 0310 implicit transparency once a
            // non-None material is effectively applied.
            style.background_material = Some(WindowBackgroundMaterialRequest::Vibrancy);
        }

        config.main_window_style = style;
    })
    .init_app(|app| {
        // Keep a consistent cookbook look (tokens, typography).
        fret_cookbook::install_cookbook_defaults(app);

        if cfg!(target_os = "windows") {
            app.commands_mut().register(
                CommandId::from(CMD_TO_NONE),
                CommandMeta::new("Background material: None"),
            );
            app.commands_mut().register(
                CommandId::from(CMD_TO_MICA),
                CommandMeta::new("Background material: Mica"),
            );
            app.commands_mut().register(
                CommandId::from(CMD_TO_ACRYLIC),
                CommandMeta::new("Background material: Acrylic"),
            );
        } else if cfg!(target_os = "macos") {
            app.commands_mut().register(
                CommandId::from(CMD_TO_NONE),
                CommandMeta::new("Background material: None"),
            );
            app.commands_mut().register(
                CommandId::from(CMD_TO_VIBRANCY),
                CommandMeta::new("Background material: Vibrancy"),
            );
        }
        app.commands_mut()
            .register(CommandId::from(CMD_QUIT), CommandMeta::new("Quit"));
    })
    .run()
    .map_err(anyhow::Error::from)
}
