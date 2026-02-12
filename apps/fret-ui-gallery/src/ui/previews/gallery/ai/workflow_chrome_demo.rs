use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_workflow_chrome_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::Px;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, MetricRef, Space};

    let max_w = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(760.0)))
        .min_w_0();

    let toolbar = ui_ai::WorkflowToolbar::new([
        shadcn::Button::new("Run")
            .test_id("ui-ai-workflow-toolbar-demo-run")
            .into_element(cx),
        shadcn::Button::new("Stop")
            .variant(shadcn::ButtonVariant::Secondary)
            .test_id("ui-ai-workflow-toolbar-demo-stop")
            .into_element(cx),
    ])
    .test_id("ui-ai-workflow-toolbar-demo-root")
    .into_element(cx);

    let panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                toolbar,
                cx.text("Workflow panel (chrome-only)."),
                cx.text("Apps own node/canvas engines and interaction policy."),
            ]
        },
    );

    let panel = ui_ai::WorkflowPanel::new([panel_body])
        .test_id("ui-ai-workflow-panel-demo-root")
        .refine_layout(max_w)
        .into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Workflow chrome (AI Elements)"),
                cx.text("UI-only ports of @xyflow/react wrappers (Panel/Toolbar)."),
                panel,
            ]
        },
    )]
}
