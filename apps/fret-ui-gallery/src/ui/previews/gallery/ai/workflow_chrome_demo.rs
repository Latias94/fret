use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_workflow_chrome_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::Px;
    use fret_icons::IconId;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, MetricRef, Space};

    let max_w = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(760.0)))
        .min_w_0();

    let controls = ui_ai::WorkflowControls::new([
        ui_ai::WorkflowControlsButton::new("Zoom in", IconId::new_static("lucide.plus"))
            .test_id("ui-ai-workflow-controls-demo-zoom-in")
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Zoom out", IconId::new_static("lucide.minus"))
            .test_id("ui-ai-workflow-controls-demo-zoom-out")
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Fit view", IconId::new_static("lucide.maximize-2"))
            .test_id("ui-ai-workflow-controls-demo-fit-view")
            .into_element(cx),
    ])
    .test_id("ui-ai-workflow-controls-demo-root")
    .into_element(cx);

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

    let node = ui_ai::WorkflowNode::new([
        ui_ai::WorkflowNodeHeader::new([
            ui_ai::WorkflowNodeTitle::new("Summarize docs").into_element(cx),
            ui_ai::WorkflowNodeDescription::new("Chrome-only node wrapper.").into_element(cx),
            ui_ai::WorkflowNodeAction::new([shadcn::Button::new("Details")
                .variant(shadcn::ButtonVariant::Ghost)
                .test_id("ui-ai-workflow-node-demo-details")
                .into_element(cx)])
            .test_id("ui-ai-workflow-node-demo-action")
            .into_element(cx),
        ])
        .test_id("ui-ai-workflow-node-demo-header")
        .into_element(cx),
        ui_ai::WorkflowNodeContent::new([
            cx.text("Node content is app-owned; this is the shadcn-aligned chrome surface.")
        ])
        .test_id("ui-ai-workflow-node-demo-content")
        .into_element(cx),
        ui_ai::WorkflowNodeFooter::new([cx.text("Footer area (optional).")])
            .test_id("ui-ai-workflow-node-demo-footer")
            .into_element(cx),
    ])
    .handles(ui_ai::WorkflowNodeHandles {
        target: true,
        source: true,
    })
    .test_id("ui-ai-workflow-node-demo-root")
    .refine_layout(
        LayoutRefinement::default()
            .max_w(MetricRef::Px(Px(360.0)))
            .min_w_0(),
    )
    .into_element(cx);

    let panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N3)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full().min_w_0()),
                    move |_cx| vec![controls, toolbar],
                ),
                cx.text("Workflow panel (chrome-only)."),
                cx.text("Apps own node/canvas engines and interaction policy."),
                node,
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
