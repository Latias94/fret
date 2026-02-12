use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_workflow_node_graph_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_core::Px;
    use fret_icons::IconId;
    use fret_node::io::NodeGraphViewState;
    use fret_node::ui::{NodeGraphCanvas, NodeGraphEditor, NodeGraphViewQueue};
    use fret_node::{
        CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
        PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };
    use fret_ui::action::OnActivate;
    use fret_ui::element::{LayoutStyle, SemanticsProps};
    use fret_ui::retained_bridge::RetainedSubtreeProps;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{
        ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, MetricRef, Radius,
        Space,
    };

    #[derive(Default)]
    struct HarnessState {
        graph: Option<Model<Graph>>,
        view: Option<Model<NodeGraphViewState>>,
        view_queue: Option<Model<NodeGraphViewQueue>>,
    }

    fn build_demo_graph(graph_id: GraphId) -> Graph {
        let mut g = Graph::new(graph_id);

        let a = NodeId::from_u128(1);
        let b = NodeId::from_u128(2);
        let c = NodeId::from_u128(3);

        g.nodes.insert(
            a,
            Node {
                kind: NodeKindKey::new("demo.prompt"),
                kind_version: 1,
                pos: CanvasPoint {
                    x: -220.0,
                    y: -40.0,
                },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        );
        g.nodes.insert(
            b,
            Node {
                kind: NodeKindKey::new("demo.tool"),
                kind_version: 1,
                pos: CanvasPoint { x: 120.0, y: -80.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        );
        g.nodes.insert(
            c,
            Node {
                kind: NodeKindKey::new("demo.response"),
                kind_version: 1,
                pos: CanvasPoint { x: 120.0, y: 80.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        );

        let a_out = PortId::from_u128(10);
        let b_in = PortId::from_u128(11);
        let b_out = PortId::from_u128(12);
        let c_in = PortId::from_u128(13);

        g.ports.insert(
            a_out,
            Port {
                node: a,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
        g.ports.insert(
            b_in,
            Port {
                node: b,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
        g.ports.insert(
            b_out,
            Port {
                node: b,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
        g.ports.insert(
            c_in,
            Port {
                node: c,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        );

        g.edges.insert(
            EdgeId::from_u128(20),
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        g.edges.insert(
            EdgeId::from_u128(21),
            Edge {
                kind: EdgeKind::Data,
                from: b_out,
                to: c_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        g
    }

    let existing = cx.with_state(HarnessState::default, |st| {
        match (st.graph.clone(), st.view.clone(), st.view_queue.clone()) {
            (Some(graph), Some(view), Some(view_queue)) => Some((graph, view, view_queue)),
            _ => None,
        }
    });

    let (graph, view, view_queue) = if let Some(existing) = existing {
        existing
    } else {
        let graph = build_demo_graph(GraphId::from_u128(42));
        let graph = cx.app.models_mut().insert(graph);
        let view = cx.app.models_mut().insert(NodeGraphViewState::default());
        let view_queue = cx.app.models_mut().insert(NodeGraphViewQueue::default());

        cx.with_state(HarnessState::default, |st| {
            st.graph = Some(graph.clone());
            st.view = Some(view.clone());
            st.view_queue = Some(view_queue.clone());
        });

        (graph, view, view_queue)
    };

    let max_w = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(900.0)))
        .min_w_0();

    let zoom_in: OnActivate = Arc::new({
        let view = view.clone();
        move |host, _cx, _reason| {
            let _ = host.models_mut().update(&view, |st| {
                let z = st.zoom;
                let z = if z.is_finite() && z > 0.0 { z } else { 1.0 };
                st.zoom = (z * 1.10).min(4.0);
            });
        }
    });

    let zoom_out: OnActivate = Arc::new({
        let view = view.clone();
        move |host, _cx, _reason| {
            let _ = host.models_mut().update(&view, |st| {
                let z = st.zoom;
                let z = if z.is_finite() && z > 0.0 { z } else { 1.0 };
                st.zoom = (z / 1.10).max(0.15);
            });
        }
    });

    let fit_view: OnActivate = Arc::new({
        let graph = graph.clone();
        let view_queue = view_queue.clone();
        move |host, _cx, _reason| {
            let nodes = host
                .models_mut()
                .read(&graph, |g| g.nodes.keys().copied().collect::<Vec<_>>())
                .unwrap_or_default();
            let _ = host.models_mut().update(&view_queue, |q| {
                q.push_frame_nodes(nodes);
            });
        }
    });

    let reset_view: OnActivate = Arc::new({
        let view = view.clone();
        move |host, _cx, _reason| {
            let _ = host.models_mut().update(&view, |st| {
                st.pan = CanvasPoint::default();
                st.zoom = 1.0;
            });
        }
    });

    let controls = ui_ai::WorkflowControls::new([
        ui_ai::WorkflowControlsButton::new("Zoom in", IconId::new_static("lucide.plus"))
            .test_id("ui-ai-workflow-node-graph-demo-zoom-in")
            .on_activate(zoom_in)
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Zoom out", IconId::new_static("lucide.minus"))
            .test_id("ui-ai-workflow-node-graph-demo-zoom-out")
            .on_activate(zoom_out)
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Fit view", IconId::new_static("lucide.maximize-2"))
            .test_id("ui-ai-workflow-node-graph-demo-fit-view")
            .on_activate(fit_view)
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Reset view", IconId::new_static("lucide.refresh-ccw"))
            .test_id("ui-ai-workflow-node-graph-demo-reset-view")
            .on_activate(reset_view)
            .into_element(cx),
    ])
    .test_id("ui-ai-workflow-node-graph-demo-controls")
    .into_element(cx);

    let toolbar = ui_ai::WorkflowToolbar::new([
        shadcn::Button::new("Run")
            .test_id("ui-ai-workflow-node-graph-demo-run")
            .into_element(cx),
        shadcn::Button::new("Stop")
            .variant(shadcn::ButtonVariant::Secondary)
            .test_id("ui-ai-workflow-node-graph-demo-stop")
            .into_element(cx),
    ])
    .test_id("ui-ai-workflow-node-graph-demo-toolbar")
    .into_element(cx);

    let overlay_panel = ui_ai::WorkflowPanel::new([stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N3)
            .items(Items::Center)
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |_cx| vec![controls, toolbar],
    )])
    .test_id("ui-ai-workflow-node-graph-demo-overlay-panel")
    .refine_layout(
        LayoutRefinement::default()
            .absolute()
            .top(Space::N2)
            .left(Space::N2),
    )
    .into_element(cx);

    let stage_layout = LayoutRefinement::default()
        .w_full()
        .h_px(Px(420.0))
        .min_w_0()
        .relative()
        .overflow_hidden();

    let stage_props = decl_style::container_props(
        theme,
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
            .p(Space::N0),
        stage_layout,
    );

    let stage = cx.container(stage_props, move |cx| {
        let graph = graph.clone();
        let view = view.clone();
        let view_queue = view_queue.clone();

        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;

        let props = RetainedSubtreeProps::new::<App>(move |ui| {
            use fret_ui::retained_bridge::UiTreeRetainedExt as _;

            let editor = ui.create_node_retained(NodeGraphEditor::new());
            let canvas = NodeGraphCanvas::new(graph.clone(), view.clone())
                .with_view_queue(view_queue.clone())
                .with_fit_view_on_mount();
            let canvas_node = ui.create_node_retained(canvas);
            ui.set_children(editor, vec![canvas_node]);
            editor
        })
        .with_layout(layout);

        let subtree = cx.retained_subtree(props);
        vec![subtree, overlay_panel]
    });

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                cx.text("Workflow editor (engine-backed)"),
                cx.text("Uses fret-node for graph interaction + fret-ui-ai for chrome wrappers."),
                stage,
            ]
        },
    );

    let panel = ui_ai::WorkflowPanel::new([body])
        .test_id("ui-ai-workflow-node-graph-demo-root")
        .refine_layout(max_w)
        .into_element(cx);

    vec![cx.semantics(
        SemanticsProps {
            test_id: Some(Arc::<str>::from("ui-ai-workflow-node-graph-demo-page")),
            ..Default::default()
        },
        move |_cx| vec![panel],
    )]
}
