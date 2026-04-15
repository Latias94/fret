//! Retained-bridge IMUI demo for `fret-node`.
//! This example is compatibility-oriented and should not be treated as the default downstream
//! authoring path for node-graph apps.
//! Prefer the declarative node-graph surfaces for normal downstream guidance.
use fret::{FretApp, advanced::prelude::*, component::prelude::*};
use fret_node::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use fret_node::io::{NodeGraphEditorConfig, NodeGraphViewState};
use fret_runtime::Model;
use serde_json::Value;

struct ImUiNodeGraphView {
    graph: Model<Graph>,
    view: Model<NodeGraphViewState>,
    editor_config: Model<NodeGraphEditorConfig>,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-node-graph-demo")
        .window("imui_node_graph_demo", (980.0, 720.0))
        .view::<ImUiNodeGraphView>()?
        .run()?;
    Ok(())
}

impl View for ImUiNodeGraphView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        let graph = demo_graph();
        let graph = app.models_mut().insert(graph);
        let view = app.models_mut().insert(NodeGraphViewState::default());
        let editor_config = app.models_mut().insert(NodeGraphEditorConfig::default());
        Self {
            graph,
            view,
            editor_config,
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let graph = self.graph.clone();
        let view = self.view.clone();
        let editor_config = self.editor_config.clone();

        fret_imui::imui_in(cx, |ui| {
            use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
            use fret_ui_kit::imui::UiWriterUiKitExt as _;

            let root = fret_ui_kit::ui::v_flex_build(move |cx, out| {
                fret_imui::imui_build(cx, out, |ui| {
                    let title = fret_ui_kit::ui::text("imui node-graph compatibility proof")
                        .font_semibold();
                    ui.add_ui(title);
                    ui.separator();

                    let mut surface_props =
                        fret_node::ui::declarative::NodeGraphSurfaceCompatRetainedProps::new(
                            graph.clone(),
                            view.clone(),
                            editor_config.clone(),
                        );
                    surface_props.fit_view_on_mount = true;
                    let surface = fret_node::ui::declarative::node_graph_surface_compat_retained(
                        ui.cx_mut(),
                        surface_props,
                    );
                    ui.add(surface);
                });
            })
            .size_full();
            ui.add_ui(root);
        })
    }
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
