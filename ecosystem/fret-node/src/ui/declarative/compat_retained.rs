//! Compatibility declarative entrypoints that host retained subtrees internally.
//!
//! These surfaces are intentionally feature-gated behind `compat-retained-canvas` so that
//! downstream ecosystem authors can adopt `fret-node`'s declarative UI without enabling
//! `fret-ui/unstable-retained-bridge`.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::element::AnyElement;
use fret_ui::element::SemanticsProps;
use fret_ui::retained_bridge::RetainedSubtreeProps;

use crate::Graph;
use crate::io::NodeGraphViewState;
use crate::runtime::store::NodeGraphStore;
use crate::ui::{
    NodeGraphCanvas, NodeGraphEditQueue, NodeGraphEditor, NodeGraphInternalsStore,
    NodeGraphOverlayState, NodeGraphViewQueue,
};

#[derive(Clone)]
pub struct NodeGraphSurfaceCompatRetainedProps {
    pub graph: Model<Graph>,
    pub view_state: Model<NodeGraphViewState>,
    pub store: Option<Model<NodeGraphStore>>,
    pub edit_queue: Option<Model<NodeGraphEditQueue>>,
    pub view_queue: Option<Model<NodeGraphViewQueue>>,
    pub overlays: Option<Model<NodeGraphOverlayState>>,
    pub internals: Option<Arc<NodeGraphInternalsStore>>,
    pub fit_view_on_mount: bool,
    pub test_id: Option<Arc<str>>,
}

impl NodeGraphSurfaceCompatRetainedProps {
    pub fn new(graph: Model<Graph>, view_state: Model<NodeGraphViewState>) -> Self {
        Self {
            graph,
            view_state,
            store: None,
            edit_queue: None,
            view_queue: None,
            overlays: None,
            internals: None,
            fit_view_on_mount: false,
            test_id: None,
        }
    }
}

/// Declarative entrypoint that hosts the current retained node-graph canvas/editor as an internal
/// subtree.
///
/// This is a **compatibility** surface:
/// - It allows declarative composition at the ecosystem/app layer today.
/// - It keeps retained authoring out of the downstream API surface.
/// - It is delete-planned as the canvas interaction + portals move fully declarative.
pub fn node_graph_surface_compat_retained<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphSurfaceCompatRetainedProps,
) -> AnyElement {
    let NodeGraphSurfaceCompatRetainedProps {
        graph,
        view_state,
        store,
        edit_queue,
        view_queue,
        overlays,
        internals,
        fit_view_on_mount,
        test_id,
    } = props;

    let retained = RetainedSubtreeProps::new::<H>(move |ui| {
        use fret_ui::retained_bridge::UiTreeRetainedExt as _;

        let editor = ui.create_node_retained(NodeGraphEditor::new());

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view_state.clone());
        if let Some(store) = store.clone() {
            canvas = canvas.with_store(store);
        }
        if let Some(edit_queue) = edit_queue.clone() {
            canvas = canvas.with_edit_queue(edit_queue);
        }
        if let Some(view_queue) = view_queue.clone() {
            canvas = canvas.with_view_queue(view_queue);
        }
        if let Some(overlays) = overlays.clone() {
            canvas = canvas.with_overlay_state(overlays);
        }
        if let Some(internals) = internals.clone() {
            canvas = canvas.with_internals_store(internals);
        }
        if fit_view_on_mount {
            canvas = canvas.with_fit_view_on_mount();
        }

        let canvas_node = ui.create_node_retained(canvas);
        ui.set_children(editor, vec![canvas_node]);
        editor
    });

    let subtree = cx.retained_subtree(retained);

    let Some(test_id) = test_id else {
        return subtree;
    };

    cx.semantics(
        SemanticsProps {
            test_id: Some(test_id),
            ..Default::default()
        },
        move |_cx| vec![subtree],
    )
}
