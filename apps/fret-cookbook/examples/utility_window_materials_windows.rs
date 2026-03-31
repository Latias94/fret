use std::sync::Arc;

use fret::component::prelude::*;
use fret::{advanced::prelude::*, shadcn};
use fret_app::{CommandId, CommandMeta, Effect, WindowRequest};
use fret_core::{AppWindowId, Color, Px};
use fret_render::WgpuAdapterSelectionSnapshot;
use fret_runtime::{
    Model, PlatformCapabilities, RunnerWindowStyleDiagnosticsStore,
    WindowBackgroundMaterialRequest, WindowDecorationsRequest, WindowStyleRequest,
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

fn install_defaults_and_commands(app: &mut KernelApp) {
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
}

#[derive(Debug)]
struct State {
    window: AppWindowId,
    status: Model<Arc<str>>,
}

fn init_window(app: &mut KernelApp, window: AppWindowId) -> State {
    State {
        window,
        status: app.models_mut().insert(Arc::from("Idle")),
    }
}

fn configure_driver(driver: UiAppDriver<State>) -> UiAppDriver<State> {
    driver.on_command(on_command)
}

fn on_command(
    app: &mut KernelApp,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<KernelApp>,
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

fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut State) -> ViewElements {
    let is_windows = cfg!(target_os = "windows");
    let is_macos = cfg!(target_os = "macos");

    let theme = cx.theme().snapshot();
    let color_muted_foreground = theme.color_token("muted-foreground");

    let status = cx
        .app
        .models()
        .read(&st.status, |v: &Arc<str>| Arc::clone(v))
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
    let root_background = effective_style
        .as_ref()
        .map(|s| s.visual_transparent)
        .unwrap_or(false)
        .then_some(Color::TRANSPARENT)
        .unwrap_or_else(|| theme.color_token("background"));
    let style_text: Arc<str> = Arc::from(match effective_style {
        Some(s) => format!(
            "effective: decorations={:?} resizable={} surface_alpha={} surface_alpha_source={:?} visual_transparent={} material={:?}",
            s.decorations,
            s.resizable,
            s.surface_composited_alpha,
            s.surface_composited_alpha_source,
            s.visual_transparent,
            s.background_material
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

    let header = shadcn::card_header(|cx| {
        ui::children![
            cx;
            shadcn::card_title(title),
            shadcn::card_description(description),
        ]
    });

    let content = ui::v_flex(|cx| {
        let platform_line = ui::text(platform_text)
            .font_monospace()
            .text_sm()
            .text_color(ColorRef::Color(color_muted_foreground))
            .test_id(TEST_ID_PLATFORM_TEXT);
        let style_line = ui::text(style_text)
            .font_monospace()
            .text_sm()
            .test_id(TEST_ID_STYLE_TEXT);
        let caps_line = ui::text(caps_text)
            .font_monospace()
            .text_sm()
            .text_color(ColorRef::Color(color_muted_foreground));
        let wgpu_line = ui::text(wgpu_text)
            .font_monospace()
            .text_sm()
            .text_color(ColorRef::Color(color_muted_foreground));
        let status_line = ui::text(status)
            .text_sm()
            .text_color(ColorRef::Color(color_muted_foreground));

        let buttons_children = if is_windows {
            ui::children![
                cx;
                shadcn::Button::new("None")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(CMD_TO_NONE)
                    .test_id(TEST_ID_TO_NONE),
                shadcn::Button::new("Mica")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(CMD_TO_MICA)
                    .test_id(TEST_ID_TO_MICA),
                shadcn::Button::new("Acrylic")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(CMD_TO_ACRYLIC)
                    .test_id(TEST_ID_TO_ACRYLIC),
                shadcn::Button::new("Quit")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .size(shadcn::ButtonSize::Sm)
                    .action(CMD_QUIT),
            ]
        } else if is_macos {
            ui::children![
                cx;
                shadcn::Button::new("None")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(CMD_TO_NONE)
                    .test_id(TEST_ID_TO_NONE),
                shadcn::Button::new("Vibrancy")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(CMD_TO_VIBRANCY)
                    .test_id(TEST_ID_TO_VIBRANCY),
                shadcn::Button::new("Quit")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .size(shadcn::ButtonSize::Sm)
                    .action(CMD_QUIT),
            ]
        } else {
            ui::children![
                cx;
                shadcn::Button::new("Quit")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .size(shadcn::ButtonSize::Sm)
                    .action(CMD_QUIT),
            ]
        };

        let buttons = ui::h_flex(|_cx| buttons_children).gap(Space::N2);

        ui::children![
            cx;
            platform_line,
            style_line,
            caps_line,
            wgpu_line,
            status_line,
            buttons,
        ]
    })
    .gap(Space::N3);

    let surface = shadcn::card(|cx| {
        ui::children![
            cx;
            header,
            shadcn::card_content(|cx| ui::single(cx, content)),
        ]
    })
    .ui()
    .w_full()
    .max_w(Px(760.0))
    .into_element(cx);

    // Default to an opaque background so `material=None` looks like a typical window. When a
    // backdrop material is enabled, switch to a transparent root so the OS material can show
    // through where not covered by UI (most content lives inside the centered Card).
    ui::container(|cx| {
        ui::single(
            cx,
            ui::v_flex(|cx| ui::single(cx, surface))
                .items_center()
                .justify_center()
                .size_full(),
        )
    })
    .bg(ColorRef::Color(root_background))
    .p(Space::N6)
    .size_full()
    .test_id(TEST_ID_ROOT)
    .into_element(cx)
    .into()
}

fn main() -> anyhow::Result<()> {
    let builder = ui_app_with_hooks(
        "cookbook-utility-window-materials-windows",
        init_window,
        view,
        configure_driver,
    )
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
    .setup(install_defaults_and_commands);

    #[cfg(feature = "cookbook-diag")]
    let builder = builder.with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}
