use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_workflow_node_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

    let node = ui_ai::WorkflowNode::new([
        ui_ai::WorkflowNodeHeader::new([ui_ai::WorkflowNodeTitle::new("Summarize")
            .into_element(cx)
            .test_id("ui-ai-workflow-node-demo-title")])
        .test_id("ui-ai-workflow-node-demo-header")
        .into_element(cx),
        ui_ai::WorkflowNodeContent::new([
            cx.text("Node content slot: apps own interaction + state."),
            cx.text("Use handles as a styling seam (not an engine)."),
        ])
        .test_id("ui-ai-workflow-node-demo-content")
        .into_element(cx),
        ui_ai::WorkflowNodeFooter::new([cx.text("Footer slot")])
            .test_id("ui-ai-workflow-node-demo-footer")
            .into_element(cx),
    ])
    .handles(ui_ai::WorkflowNodeHandles {
        target: true,
        source: true,
    })
    .test_id("ui-ai-workflow-node-demo-node")
    .refine_layout(LayoutRefinement::default().w_px(Px(320.0)).min_w_0())
    .into_element(cx);

    let chrome = ChromeRefinement::default()
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .p(Space::N4);
    let props = decl_style::container_props(
        theme,
        chrome,
        LayoutRefinement::default().w_full().min_w_0(),
    );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("WorkflowNode (AI Elements): header/content/footer chrome."),
                cx.container(props, move |_cx| vec![node]),
            ]
        },
    )]
}
