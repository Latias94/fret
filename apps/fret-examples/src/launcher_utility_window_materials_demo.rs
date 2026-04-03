use std::sync::Arc;

use fret::advanced::prelude::*;
use fret_app::{CommandId, Effect, WindowRequest};
use fret_core::Px;
use fret_runtime::{
    RunnerWindowStyleDiagnosticsStore, WindowBackgroundMaterialRequest, WindowDecorationsRequest,
    WindowStyleRequest,
};
use fret_ui::ElementContext;
use fret_ui::element::{LayoutStyle, Length, SemanticsDecoration, SizeStyle};
use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
use fret_ui_shadcn::facade as shadcn;

const CMD_TO_NONE: &str = "launcher_utility_window_materials_demo.to_none";
const CMD_TO_MICA: &str = "launcher_utility_window_materials_demo.to_mica";
const CMD_TO_ACRYLIC: &str = "launcher_utility_window_materials_demo.to_acrylic";
const CMD_QUIT: &str = "launcher_utility_window_materials_demo.quit";

const TEST_ID_ROOT: &str = "utility-window.materials.root";
const TEST_ID_TO_NONE: &str = "utility-window.materials.to_none";
const TEST_ID_TO_MICA: &str = "utility-window.materials.to_mica";
const TEST_ID_TO_ACRYLIC: &str = "utility-window.materials.to_acrylic";
const TEST_ID_STYLE_TEXT: &str = "utility-window.materials.style_effective";

fn install_commands(app: &mut KernelApp) {
    app.commands_mut().register(
        CommandId::from(CMD_TO_NONE),
        fret_app::CommandMeta::new("Background material: None"),
    );
    app.commands_mut().register(
        CommandId::from(CMD_TO_MICA),
        fret_app::CommandMeta::new("Background material: Mica"),
    );
    app.commands_mut().register(
        CommandId::from(CMD_TO_ACRYLIC),
        fret_app::CommandMeta::new("Background material: Acrylic"),
    );
    app.commands_mut().register(
        CommandId::from(CMD_QUIT),
        fret_app::CommandMeta::new("Quit"),
    );
}

pub fn run() -> anyhow::Result<()> {
    ui_app_with_hooks(
        "launcher-utility-window-materials-demo",
        init_window,
        view,
        configure_driver,
    )
    .with_default_diagnostics()
    .with_main_window("launcher_utility_window_materials_demo", (720.0, 460.0))
    .configure(|config| {
        // Prefer implicit transparency when a material is requested (ADR 0310).
        config.main_window_style = WindowStyleRequest {
            decorations: Some(WindowDecorationsRequest::None),
            resizable: Some(true),
            background_material: Some(WindowBackgroundMaterialRequest::Mica),
            ..Default::default()
        };
    })
    .setup(install_commands)
    .run()
    .map_err(anyhow::Error::from)
}

struct LauncherUtilityWindowMaterialsState {
    window: AppWindowId,
    status: LocalState<Arc<str>>,
}

fn init_window(app: &mut KernelApp, window: AppWindowId) -> LauncherUtilityWindowMaterialsState {
    LauncherUtilityWindowMaterialsState {
        window,
        status: LocalState::new_in(app.models_mut(), Arc::from("Idle")),
    }
}

fn configure_driver(
    driver: UiAppDriver<LauncherUtilityWindowMaterialsState>,
) -> UiAppDriver<LauncherUtilityWindowMaterialsState> {
    driver.on_command(on_command)
}

fn on_command(
    app: &mut KernelApp,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<KernelApp>,
    st: &mut LauncherUtilityWindowMaterialsState,
    command: &CommandId,
) {
    let material = match command.as_str() {
        CMD_TO_NONE => WindowBackgroundMaterialRequest::None,
        CMD_TO_MICA => WindowBackgroundMaterialRequest::Mica,
        CMD_TO_ACRYLIC => WindowBackgroundMaterialRequest::Acrylic,
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
    let _ = st.status.set_in(
        app.models_mut(),
        Arc::from(format!("Requested: {material:?}")),
    );
    app.request_redraw(window);
}

fn view(
    cx: &mut ElementContext<'_, KernelApp>,
    st: &mut LauncherUtilityWindowMaterialsState,
) -> ViewElements {
    let theme = cx.theme().snapshot();
    let color_muted_foreground = theme.color_token("muted-foreground");
    let color_secondary = theme.color_token("secondary");

    let status = st.status.layout_value_in(cx);

    let effective_style = cx
        .app
        .global::<RunnerWindowStyleDiagnosticsStore>()
        .and_then(|store| store.effective_snapshot(st.window));
    let style_text: Arc<str> = Arc::from(match effective_style {
        Some(s) => format!(
            "effective: decorations={:?} resizable={} surface_alpha={} visual_transparent={} material={:?}",
            s.decorations,
            s.resizable,
            s.surface_composited_alpha,
            s.visual_transparent,
            s.background_material
        ),
        None => "effective: <unavailable>".to_string(),
    });

    let header = cx.container(
        fret_ui::element::ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(44.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: fret_core::Edges::all(Px(8.0)).into(),
            background: Some(color_secondary),
            corner_radii: fret_core::Corners::all(Px(10.0)),
            ..Default::default()
        },
        move |cx| {
            vec![
                ui::text("Utility Window Materials (Mica/Acrylic)")
                    .font_semibold()
                    .into_element(cx),
            ]
        },
    );

    let content = shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Background material").into_element(cx),
            shadcn::CardDescription::new(
                "Requests are capability-gated and observable via diagnostics snapshots.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([ui::v_flex(move |cx| {
            let style_line = ui::text(style_text)
                .font_monospace()
                .text_sm()
                .test_id(TEST_ID_STYLE_TEXT)
                .into_element(cx);
            let status_line = ui::text(status)
                .text_sm()
                .text_color(ColorRef::Color(color_muted_foreground))
                .into_element(cx);

            let buttons_row = ui::h_flex(move |cx| {
                [
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
            })
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx);

            vec![style_line, status_line, buttons_row]
        })
        .gap(Space::N3)
        .into_element(cx)])
        .into_element(cx),
    ])
    .into_element(cx);

    let root = cx
        .container(
            fret_ui::element::ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges::all(Px(16.0)).into(),
                background: None,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::v_flex(move |_cx| [header, content])
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(SemanticsDecoration::default().test_id(TEST_ID_ROOT));

    vec![root].into()
}
