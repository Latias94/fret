use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_workflow_chrome_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::Point;
    use fret_core::Px;
    use fret_icons::IconId;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{
        ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Radius, Space,
    };

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
            let canvas_controls = ui_ai::WorkflowControls::new([
                ui_ai::WorkflowControlsButton::new("Zoom in", IconId::new_static("lucide.plus"))
                    .test_id("ui-ai-workflow-canvas-demo-zoom-in")
                    .into_element(cx),
                ui_ai::WorkflowControlsButton::new("Zoom out", IconId::new_static("lucide.minus"))
                    .test_id("ui-ai-workflow-canvas-demo-zoom-out")
                    .into_element(cx),
            ])
            .test_id("ui-ai-workflow-canvas-demo-controls")
            .into_element(cx);

            let canvas_toolbar = ui_ai::WorkflowToolbar::new([shadcn::Button::new("Run")
                .variant(shadcn::ButtonVariant::Secondary)
                .test_id("ui-ai-workflow-canvas-demo-run")
                .into_element(cx)])
            .test_id("ui-ai-workflow-canvas-demo-toolbar")
            .into_element(cx);

            let canvas_overlay_panel = ui_ai::WorkflowPanel::new([stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N3)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                move |_cx| vec![canvas_controls, canvas_toolbar],
            )])
            .test_id("ui-ai-workflow-canvas-demo-panel")
            .refine_layout(
                LayoutRefinement::default()
                    .absolute()
                    .top(Space::N0)
                    .left(Space::N0),
            )
            .into_element(cx);

            let canvas = ui_ai::WorkflowCanvas::new([canvas_overlay_panel])
                .test_id("ui-ai-workflow-canvas-demo-root")
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_px(Px(240.0))
                        .min_w_0(),
                )
                .into_element(cx);

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
                {
                    let stage_layout = LayoutRefinement::default()
                        .w_full()
                        .h_px(Px(160.0))
                        .min_w_0()
                        .relative();

                    let stage_props = decl_style::container_props(
                        _theme,
                        ChromeRefinement::default()
                            .rounded(Radius::Md)
                            .border_1()
                            .bg(ColorRef::Token {
                                key: "card",
                                fallback: ColorFallback::ThemePanelBackground,
                            })
                            .border_color(ColorRef::Token {
                                key: "border",
                                fallback: ColorFallback::ThemePanelBorder,
                            })
                            .p(Space::N2),
                        stage_layout,
                    );

                    let abs = LayoutRefinement::default().absolute().inset(Space::N0);

                    let connection = ui_ai::WorkflowConnection::new(
                        Point::new(Px(40.0), Px(32.0)),
                        Point::new(Px(320.0), Px(32.0)),
                    )
                    .test_id("ui-ai-workflow-connection-demo-root")
                    .refine_layout(abs.clone())
                    .into_element(cx);

                    let edge_temporary = ui_ai::WorkflowEdgeTemporary::new(
                        Point::new(Px(40.0), Px(80.0)),
                        Point::new(Px(320.0), Px(80.0)),
                    )
                    .test_id("ui-ai-workflow-edge-temporary-demo-root")
                    .refine_layout(abs.clone())
                    .into_element(cx);

                    let edge_animated = ui_ai::WorkflowEdgeAnimated::new(
                        Point::new(Px(40.0), Px(128.0)),
                        Point::new(Px(320.0), Px(128.0)),
                    )
                    .test_id("ui-ai-workflow-edge-animated-demo-root")
                    .refine_layout(abs)
                    .into_element(cx);

                    cx.container(stage_props, move |_cx| {
                        vec![connection, edge_temporary, edge_animated]
                    })
                    .test_id("ui-ai-workflow-edge-stage-demo-root")
                },
                canvas,
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
