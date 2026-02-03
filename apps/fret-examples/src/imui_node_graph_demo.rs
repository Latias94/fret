use fret_kit::prelude::*;
use fret_node::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use fret_node::io::NodeGraphViewState;
use fret_node::ui::{NodeGraphCanvas, NodeGraphEditor};
use fret_runtime::Model;
use fret_ui::UiTree;
use fret_ui::element::{LayoutStyle, Length};
use fret_ui::retained_bridge::{RetainedSubtreeProps, UiTreeRetainedExt as _};
use serde_json::Value;

struct ImUiNodeGraphState {
    graph: Model<Graph>,
    view: Model<NodeGraphViewState>,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app("imui-node-graph-demo", init_window, view)?
        .with_main_window("imui_node_graph_demo", (980.0, 720.0))
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> ImUiNodeGraphState {
    let graph = demo_graph();
    let graph = app.models_mut().insert(graph);
    let view = app.models_mut().insert(NodeGraphViewState::default());
    ImUiNodeGraphState { graph, view }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut ImUiNodeGraphState) -> fret_kit::ViewElements {
    let graph = st.graph.clone();
    let view = st.view.clone();

    fret_imui::imui(cx, |ui| {
        use fret_ui_kit::imui::UiWriterUiKitExt as _;

        let root = fret_ui_kit::ui::v_flex_build(ui.cx_mut(), move |cx, out| {
            fret_imui::imui_build(cx, out, |ui| {
                let title = fret_ui_kit::ui::text(
                    ui.cx_mut(),
                    "imui + retained node graph (RetainedSubtree prototype)",
                )
                .font_semibold();
                ui.add_ui(title);
                ui.separator();

                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout.flex.grow = 1.0;

                fret_node::imui::retained_subtree_with(
                    ui,
                    RetainedSubtreeProps::new::<App>(move |ui_tree: &mut UiTree<App>| {
                        let canvas = NodeGraphCanvas::new(graph.clone(), view.clone())
                            .with_fit_view_on_mount();
                        let canvas_node = ui_tree.create_node_retained(canvas);

                        let root = ui_tree.create_node_retained(NodeGraphEditor::new());
                        ui_tree.set_children(root, vec![canvas_node]);
                        root
                    })
                    .with_layout(layout),
                );
            });
        })
        .size_full();
        ui.add_ui(root);
    })
}

fn demo_graph() -> Graph {
    let mut graph = Graph::new(GraphId::from_u128(
        0x6bfb_1b37_f0b9_4b62_8ac5_0f8c_0b8a_2f01,
    ));

    let src_node = NodeId::new();
    let dst_node = NodeId::new();
    let src_out = PortId::new();
    let dst_in = PortId::new();
    let edge = EdgeId::new();

    graph.nodes.insert(
        src_node,
        Node {
            kind: NodeKindKey::new("demo.source"),
            kind_version: 1,
            pos: CanvasPoint { x: 80.0, y: 80.0 },
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
            ports: vec![src_out],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        src_out,
        Port {
            node: src_node,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );

    graph.nodes.insert(
        dst_node,
        Node {
            kind: NodeKindKey::new("demo.sink"),
            kind_version: 1,
            pos: CanvasPoint { x: 420.0, y: 240.0 },
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
            ports: vec![dst_in],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        dst_in,
        Port {
            node: dst_node,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );

    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: src_out,
            to: dst_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    graph
}
