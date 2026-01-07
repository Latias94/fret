use fret_app::App;
use fret_app::{CommandId, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext, run_app,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;
use fret_ui::{UiFrameCx, UiTree};

use fret_node::Graph;
use fret_node::TypeDesc;
use fret_node::core::{CanvasPoint, Edge, EdgeId, EdgeKind, Node, NodeId, NodeKindKey, Port};
use fret_node::core::{PortCapacity, PortDirection, PortId, PortKey, PortKind};
use fret_node::io::NodeGraphViewState;
use fret_node::ui::NodeGraphCanvas;

#[derive(Clone)]
struct NodeGraphDemoModels {
    graph: fret_runtime::Model<Graph>,
    view: fret_runtime::Model<NodeGraphViewState>,
}

fn build_demo_graph() -> Graph {
    let mut graph = Graph::default();

    let node_value_a = NodeId::new();
    let node_value_b = NodeId::new();
    let node_merge = NodeId::new();
    let node_add = NodeId::new();
    let node_out = NodeId::new();

    let port_value_a_out = PortId::new();
    let port_value_b_out = PortId::new();
    let port_merge_in0 = PortId::new();
    let port_merge_in1 = PortId::new();
    let port_merge_out = PortId::new();
    let port_add_a = PortId::new();
    let port_add_b = PortId::new();
    let port_add_out = PortId::new();
    let port_out_in = PortId::new();

    graph.nodes.insert(
        node_value_a,
        Node {
            kind: NodeKindKey::new("demo.float"),
            kind_version: 1,
            pos: CanvasPoint { x: 40.0, y: 60.0 },
            collapsed: false,
            ports: vec![port_value_a_out],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_value_b,
        Node {
            kind: NodeKindKey::new("demo.float"),
            kind_version: 1,
            pos: CanvasPoint { x: 40.0, y: 170.0 },
            collapsed: false,
            ports: vec![port_value_b_out],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_merge,
        Node {
            kind: NodeKindKey::new("fret.variadic_merge"),
            kind_version: 1,
            pos: CanvasPoint { x: 300.0, y: 90.0 },
            collapsed: false,
            ports: vec![port_merge_in0, port_merge_in1, port_merge_out],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_add,
        Node {
            kind: NodeKindKey::new("demo.add"),
            kind_version: 1,
            pos: CanvasPoint { x: 560.0, y: 100.0 },
            collapsed: false,
            ports: vec![port_add_a, port_add_b, port_add_out],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        node_out,
        Node {
            kind: NodeKindKey::new("demo.output"),
            kind_version: 1,
            pos: CanvasPoint { x: 840.0, y: 140.0 },
            collapsed: false,
            ports: vec![port_out_in],
            data: serde_json::Value::Null,
        },
    );

    graph.ports.insert(
        port_value_a_out,
        Port {
            node: node_value_a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_value_b_out,
        Port {
            node: node_value_b,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );

    graph.ports.insert(
        port_merge_in0,
        Port {
            node: node_merge,
            key: PortKey::new("in0"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_merge_in1,
        Port {
            node: node_merge,
            key: PortKey::new("in1"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_merge_out,
        Port {
            node: node_merge,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );

    graph.ports.insert(
        port_add_a,
        Port {
            node: node_add,
            key: PortKey::new("a"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_add_b,
        Port {
            node: node_add,
            key: PortKey::new("b"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_add_out,
        Port {
            node: node_add,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        port_out_in,
        Port {
            node: node_out,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: Some(TypeDesc::Float),
            data: serde_json::Value::Null,
        },
    );

    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_value_a_out,
            to: port_merge_in0,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_merge_out,
            to: port_add_a,
        },
    );
    graph.edges.insert(
        EdgeId::new(),
        Edge {
            kind: EdgeKind::Data,
            from: port_add_out,
            to: port_out_in,
        },
    );

    graph
}

struct NodeGraphDemoWindowState {
    ui: UiTree<App>,
    root: fret_core::NodeId,
}

#[derive(Default)]
struct NodeGraphDemoDriver;

impl NodeGraphDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> NodeGraphDemoWindowState {
        let models = app
            .global::<NodeGraphDemoModels>()
            .expect("NodeGraphDemoModels global must exist")
            .clone();

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let canvas = NodeGraphCanvas::new(models.graph, models.view)
            .with_close_command(CommandId::new("node_graph_demo.close"));
        let root = ui.create_node_retained(canvas);
        ui.set_root(root);

        NodeGraphDemoWindowState { ui, root }
    }
}

impl WinitAppDriver for NodeGraphDemoDriver {
    type WindowState = NodeGraphDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;

        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if let Event::KeyDown { key, .. } = event {
            if *key == fret_core::KeyCode::Escape {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: fret_app::CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
        } = context;

        if state.ui.dispatch_command(app, services, &command) {
            return;
        }

        if command.as_str() == "node_graph_demo.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        state.ui.set_root(state.root);
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();

        let mut frame = UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<fret_launch::WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());

    let graph = app.models_mut().insert(build_demo_graph());
    let view = app.models_mut().insert(NodeGraphViewState::default());
    app.set_global(NodeGraphDemoModels { graph, view });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo node_graph_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    run_app(config, app, NodeGraphDemoDriver::default()).map_err(anyhow::Error::from)
}
