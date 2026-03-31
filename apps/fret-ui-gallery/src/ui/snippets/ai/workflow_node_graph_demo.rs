pub const SOURCE: &str = include_str!("workflow_node_graph_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    use std::sync::Arc;

    use fret_core::Px;
    use fret_icons::IconId;
    use fret_node::io::NodeGraphViewState;
    use fret_node::runtime::store::NodeGraphStore;
    use fret_node::ui::advanced::{NodeGraphViewQueue, bind_controller_view_queue_transport};
    use fret_node::ui::{
        NodeGraphCanvas, NodeGraphController, NodeGraphEditor, NodeGraphSurfaceBinding,
    };
    use fret_node::{
        CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
        PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };
    use fret_runtime::Model;
    use fret_ui::action::{ActionCx, UiActionHost};
    use fret_ui::element::{LayoutStyle, SemanticsProps};
    use fret_ui::retained_bridge::RetainedSubtreeProps;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::ui;
    use fret_ui_kit::{
        ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, MetricRef, Radius,
        Space,
    };

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

    fn zoom_around_view_center(
        bounds: fret_core::Rect,
        pan: CanvasPoint,
        zoom: f32,
        next_zoom: f32,
    ) -> (CanvasPoint, f32) {
        let w = bounds.size.width.0;
        let h = bounds.size.height.0;

        let z0 = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let z1 = if next_zoom.is_finite() && next_zoom > 0.0 {
            next_zoom
        } else {
            z0
        };

        // Invert the node-graph viewport centering math used by the advanced viewport helper:
        // pan.x = w / (2*z) - center.x  =>  center.x = w / (2*z) - pan.x
        let center = CanvasPoint {
            x: w / (2.0 * z0) - pan.x,
            y: h / (2.0 * z0) - pan.y,
        };

        let pan1 = CanvasPoint {
            x: w / (2.0 * z1) - center.x,
            y: h / (2.0 * z1) - center.y,
        };

        (pan1, z1)
    }

    #[derive(Clone)]
    struct BoundsRecorder {
        bounds: Model<Option<fret_core::Rect>>,
    }

    impl BoundsRecorder {
        fn new(bounds: Model<Option<fret_core::Rect>>) -> Self {
            Self { bounds }
        }
    }

    impl<H: fret_ui::UiHost> fret_ui::retained_bridge::Widget<H> for BoundsRecorder {
        fn layout(
            &mut self,
            cx: &mut fret_ui::retained_bridge::LayoutCx<'_, H>,
        ) -> fret_core::Size {
            let prev = cx
                .app
                .models_mut()
                .read(&self.bounds, |b| *b)
                .ok()
                .flatten();
            if prev != Some(cx.bounds) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.bounds, |b| *b = Some(cx.bounds));
            }
            cx.bounds.size
        }

        fn hit_test(&self, _bounds: fret_core::Rect, _position: fret_core::Point) -> bool {
            false
        }
    }

    #[derive(Clone)]
    struct DemoSurfaceState {
        binding: NodeGraphSurfaceBinding,
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        view_queue: Model<NodeGraphViewQueue>,
    }

    let binding_slot = cx.keyed_slot_id("binding");
    let bounds = cx.local_model_keyed("bounds", || None::<fret_core::Rect>);
    let surface = cx.state_for(
        binding_slot,
        || None::<DemoSurfaceState>,
        |slot| slot.clone(),
    );
    let surface = match surface {
        Some(surface) => surface,
        None => {
            let graph_value = build_demo_graph(GraphId::from_u128(42));
            let graph = cx.app.models_mut().insert(graph_value.clone());
            let view_state = cx.app.models_mut().insert(NodeGraphViewState::default());
            let view_queue = cx.app.models_mut().insert(NodeGraphViewQueue::default());
            let store = cx.app.models_mut().insert(NodeGraphStore::new(
                graph_value,
                NodeGraphViewState::default(),
            ));
            let controller = bind_controller_view_queue_transport(
                NodeGraphController::new(store),
                view_queue.clone(),
            );
            let surface = DemoSurfaceState {
                binding: NodeGraphSurfaceBinding::from_models(
                    graph.clone(),
                    view_state.clone(),
                    controller,
                ),
                graph,
                view_state,
                view_queue,
            };
            cx.state_for(
                binding_slot,
                || None::<DemoSurfaceState>,
                |slot| {
                    if slot.is_none() {
                        *slot = Some(surface.clone());
                    }
                    slot.clone()
                        .expect("workflow node graph binding slot must be initialized")
                },
            )
        }
    };
    let binding = surface.binding.clone();

    let max_w = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(900.0)))
        .min_w_0();

    let zoom_in = {
        let bounds = bounds.clone();
        let view_state = surface.view_state.clone();
        let view_queue = surface.view_queue.clone();
        move |host: &mut dyn UiActionHost, action_cx: ActionCx| {
            let Some(bounds) = host.models_mut().read(&bounds, |b| *b).ok().flatten() else {
                return;
            };

            let (pan, zoom) = host
                .models_mut()
                .read(&view_state, |state| (state.pan, state.zoom))
                .unwrap_or((CanvasPoint::default(), 1.0));
            let next_zoom = {
                let z = if zoom.is_finite() && zoom > 0.0 {
                    zoom
                } else {
                    1.0
                };
                (z * 1.10).min(4.0)
            };
            let (pan, zoom) = zoom_around_view_center(bounds, pan, zoom, next_zoom);
            let _ = host.models_mut().update(&view_queue, |queue| {
                queue.push_set_viewport(pan, zoom);
            });
            host.request_redraw(action_cx.window);
        }
    };

    let zoom_out = {
        let bounds = bounds.clone();
        let view_state = surface.view_state.clone();
        let view_queue = surface.view_queue.clone();
        move |host: &mut dyn UiActionHost, action_cx: ActionCx| {
            let Some(bounds) = host.models_mut().read(&bounds, |b| *b).ok().flatten() else {
                return;
            };

            let (pan, zoom) = host
                .models_mut()
                .read(&view_state, |state| (state.pan, state.zoom))
                .unwrap_or((CanvasPoint::default(), 1.0));
            let next_zoom = {
                let z = if zoom.is_finite() && zoom > 0.0 {
                    zoom
                } else {
                    1.0
                };
                (z / 1.10).max(0.15)
            };
            let (pan, zoom) = zoom_around_view_center(bounds, pan, zoom, next_zoom);
            let _ = host.models_mut().update(&view_queue, |queue| {
                queue.push_set_viewport(pan, zoom);
            });
            host.request_redraw(action_cx.window);
        }
    };

    let fit_view = {
        let graph = surface.graph.clone();
        let view_queue = surface.view_queue.clone();
        move |host: &mut dyn UiActionHost, action_cx: ActionCx| {
            let nodes = host
                .models_mut()
                .read(&graph, |graph| {
                    graph.nodes.keys().copied().collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let _ = host.models_mut().update(&view_queue, |queue| {
                queue.push_frame_nodes(nodes);
            });
            host.request_redraw(action_cx.window);
        }
    };

    let reset_view = {
        let view_queue = surface.view_queue.clone();
        move |host: &mut dyn UiActionHost, action_cx: ActionCx| {
            let _ = host.models_mut().update(&view_queue, |queue| {
                queue.push_set_viewport(CanvasPoint::default(), 1.0);
            });
            host.request_redraw(action_cx.window);
        }
    };

    let controls = ui_ai::WorkflowControls::new([
        ui_ai::WorkflowControlsButton::new("Zoom in", IconId::new_static("lucide.plus"))
            .test_id("ui-ai-workflow-node-graph-demo-zoom-in")
            .on_activate(cx.actions().listen(zoom_in))
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Zoom out", IconId::new_static("lucide.minus"))
            .test_id("ui-ai-workflow-node-graph-demo-zoom-out")
            .on_activate(cx.actions().listen(zoom_out))
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Fit view", IconId::new_static("lucide.maximize-2"))
            .test_id("ui-ai-workflow-node-graph-demo-fit-view")
            .on_activate(cx.actions().listen(fit_view))
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Reset view", IconId::new_static("lucide.refresh-ccw"))
            .test_id("ui-ai-workflow-node-graph-demo-reset-view")
            .on_activate(cx.actions().listen(reset_view))
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

    let overlay_panel = ui_ai::WorkflowPanel::new([ui::h_flex(move |_cx| vec![controls, toolbar])
        .gap(Space::N3)
        .items(Items::Center)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)])
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

    let stage_props = cx.with_theme(|theme| {
        decl_style::container_props(
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
        )
    });

    let stage = cx.container(stage_props, move |cx| {
        let binding = binding.clone();
        let bounds = bounds.clone();

        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;

        let props = RetainedSubtreeProps::new::<fret::app::App>(move |ui| {
            use fret_ui::retained_bridge::UiTreeRetainedExt as _;

            let editor = ui.create_node_retained(NodeGraphEditor::new());
            let canvas = NodeGraphCanvas::new(binding.graph_model(), binding.view_state_model())
                .with_controller(binding.controller())
                .with_fit_view_on_mount();
            let canvas_node = ui.create_node_retained(canvas);
            let bounds_node = ui.create_node_retained(BoundsRecorder::new(bounds.clone()));
            ui.set_children(editor, vec![canvas_node, bounds_node]);
            editor
        })
        .with_layout(layout);

        let subtree = cx.retained_subtree(props);
        vec![subtree, overlay_panel]
    });

    let body = ui::v_flex(move |cx| {
        vec![
            cx.text("Workflow editor (engine-backed)"),
            cx.text("Uses fret-node for graph interaction + fret-ui-ai for chrome wrappers."),
            stage,
        ]
    })
    .gap(Space::N3)
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let panel = ui_ai::WorkflowPanel::new([body])
        .test_id("ui-ai-workflow-node-graph-demo-root")
        .refine_layout(max_w)
        .into_element(cx);

    cx.semantics(
        SemanticsProps {
            test_id: Some(Arc::<str>::from("ui-ai-workflow-node-graph-demo-page")),
            ..Default::default()
        },
        move |_cx| vec![panel],
    )
}
// endregion: example
