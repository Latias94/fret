use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_workflow_panel_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

    let panel = ui_ai::WorkflowPanel::new([stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("WorkflowPanel (AI Elements)"),
                cx.text("Container chrome only. Apps own placement + interactions."),
            ]
        },
    )])
    .test_id("ui-ai-workflow-panel-demo-panel")
    .refine_layout(LayoutRefinement::default().m(Space::N0))
    .into_element(cx);

    let chrome = ChromeRefinement::default()
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .p(Space::N4);
    let props = decl_style::container_props(theme, chrome, LayoutRefinement::default().w_full());

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("WorkflowPanel (AI Elements): bordered container chrome."),
                cx.container(props, move |_cx| vec![panel]),
            ]
        },
    )]
}
