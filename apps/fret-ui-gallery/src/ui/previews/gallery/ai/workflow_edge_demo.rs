use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_workflow_edge_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::Point;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

    let from = Point::new(Px(48.0), Px(56.0));
    let to = Point::new(Px(312.0), Px(176.0));

    let dashed = ui_ai::WorkflowEdgeTemporary::new(from, to)
        .test_id("ui-ai-workflow-edge-demo-dashed")
        .into_element(cx);

    let animated = ui_ai::WorkflowEdgeAnimated::new(
        Point::new(Px(48.0), Px(176.0)),
        Point::new(Px(312.0), Px(56.0)),
    )
    .marker_end(ui_ai::WorkflowEdgeMarkerEnd::Arrow)
    .test_id("ui-ai-workflow-edge-demo-animated")
    .into_element(cx);

    let chrome = ChromeRefinement::default()
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")));
    let props = decl_style::container_props(
        theme,
        chrome,
        LayoutRefinement::default()
            .w_full()
            .h_px(Px(260.0))
            .min_w_0()
            .min_h_0()
            .relative(),
    );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("WorkflowEdge (AI Elements): dashed + animated stroke renderers."),
                cx.container(props, move |_cx| vec![dashed, animated]),
            ]
        },
    )]
}
