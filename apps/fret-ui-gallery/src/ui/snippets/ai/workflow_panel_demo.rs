pub const SOURCE: &str = include_str!("workflow_panel_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let panel = ui_ai::WorkflowPanel::new([ui::v_flex(|cx| {
        vec![
            cx.text("WorkflowPanel (AI Elements)"),
            cx.text("Container chrome only. Apps own placement + interactions."),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .into_element(cx)])
    .test_id("ui-ai-workflow-panel-demo-panel")
    .refine_layout(LayoutRefinement::default().m(Space::N0))
    .into_element(cx);

    let props = cx.with_theme(|theme| {
        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .p(Space::N4);
        decl_style::container_props(theme, chrome, LayoutRefinement::default().w_full())
    });

    ui::v_flex(move |cx| {
        vec![
            cx.text("WorkflowPanel (AI Elements): bordered container chrome."),
            cx.container(props, move |_cx| vec![panel]),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .into_element(cx)
}
// endregion: example
