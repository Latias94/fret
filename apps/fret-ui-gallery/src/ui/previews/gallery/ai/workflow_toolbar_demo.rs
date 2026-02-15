use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_workflow_toolbar_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

    let toolbar = ui_ai::WorkflowToolbar::new([
        fret_ui_shadcn::Button::new("Run")
            .test_id("ui-ai-workflow-toolbar-demo-run")
            .into_element(cx),
        fret_ui_shadcn::Button::new("Stop")
            .test_id("ui-ai-workflow-toolbar-demo-stop")
            .into_element(cx),
        fret_ui_shadcn::Button::new("Inspect")
            .test_id("ui-ai-workflow-toolbar-demo-inspect")
            .into_element(cx),
    ])
    .test_id("ui-ai-workflow-toolbar-demo-toolbar")
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
                cx.text("WorkflowToolbar (AI Elements): compact tool row chrome."),
                cx.container(props, move |_cx| vec![toolbar]),
            ]
        },
    )]
}
