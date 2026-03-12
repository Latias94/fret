pub const SOURCE: &str = include_str!("sandbox_demo.rs");

// region: example
use fret_core::Px;
use fret_ui::element::SemanticsDecoration;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Radius, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let max_w = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(760.0)))
        .min_w_0();

    let list = shadcn::TabsList::new()
        .trigger(
            shadcn::TabsTrigger::new("console", "Console")
                .test_id("ui-ai-sandbox-demo-tab-console"),
        )
        .trigger(
            shadcn::TabsTrigger::new("files", "Files").test_id("ui-ai-sandbox-demo-tab-files"),
        );

    let console_panel = {
        let panel = ui::v_flex(move |cx| {
            vec![
                cx.text("Sandbox console output (demo).").attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Generic)
                        .test_id("ui-ai-sandbox-demo-panel-console"),
                ),
                cx.text("Apps own execution backends; this is UI-only."),
            ]
        })
        .gap(Space::N2)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().p(Space::N3),
                LayoutRefinement::default().w_full().min_w_0(),
            )
        });
        cx.container(props, move |_cx| [panel])
    };

    let files_panel = {
        let panel = ui::v_flex(move |cx| {
            vec![
                cx.text("Sandbox files view (demo).").attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Generic)
                        .test_id("ui-ai-sandbox-demo-panel-files"),
                ),
                cx.text("Tabs are composable; provide your own panels."),
            ]
        })
        .gap(Space::N2)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().p(Space::N3),
                LayoutRefinement::default().w_full().min_w_0(),
            )
        });
        cx.container(props, move |_cx| [panel])
    };

    let tabs = ui_ai::SandboxTabs::uncontrolled(Some("console"))
        .list(list)
        .contents([
            shadcn::TabsContent::new("console", [console_panel]),
            shadcn::TabsContent::new("files", [files_panel]),
        ])
        .into_element(cx);

    let sandbox = ui_ai::Sandbox::new(
        ui_ai::SandboxHeader::new(ui_ai::ToolStatus::InputAvailable)
            .title("Code")
            .test_id("ui-ai-sandbox-demo-trigger"),
        ui_ai::SandboxContent::new([tabs]),
    )
    .default_open(false)
    .refine_layout(max_w)
    .refine_style(ChromeRefinement::default().rounded(Radius::Md))
    .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Sandbox (AI Elements)"),
            cx.text("Collapsible + tabs chrome. Apps own the sandbox backend."),
            sandbox,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
