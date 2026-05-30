use std::cell::RefCell;
use std::rc::Rc;

use fret_core::{AppWindowId, Point};
use fret_ui::GlobalElementId;
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};

use crate::core::Graph;
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::runtime::events::ConnectEnd;
use crate::runtime::store::DispatchOutcome;
use crate::ui::{NodeGraphControllerError, NodeGraphSurfaceBinding};

/// Outcome returned by a declarative node-graph interaction hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeGraphDeclarativeInteractionOutcome {
    /// Continue to the built-in declarative surface behavior.
    NotHandled,
    /// Stop propagation at the node-graph surface.
    Handled,
}

impl NodeGraphDeclarativeInteractionOutcome {
    pub fn is_handled(self) -> bool {
        matches!(self, Self::Handled)
    }
}

/// Request emitted when declarative connection policy should open an insert-node picker.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeGraphDeclarativeInsertNodePickerRequest {
    /// Connection-end event that caused the picker request.
    pub connect_end: ConnectEnd,
    /// Drop position in window/surface screen coordinates.
    pub screen_position: Point,
    /// Drop position mapped through the current node-graph viewport into canvas coordinates.
    pub canvas_position: Point,
}

/// Store-first context passed to declarative interaction hooks.
///
/// This context deliberately exposes snapshots and binding/controller commit helpers instead of a
/// mutable `Graph` or raw model store. Hooks can intercept UI input, but graph edits must still
/// flow through `NodeGraphStore` via the binding/controller path.
pub struct NodeGraphDeclarativeInteractionContext<'a> {
    host: &'a mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    binding: &'a NodeGraphSurfaceBinding,
}

impl<'a> NodeGraphDeclarativeInteractionContext<'a> {
    pub(crate) fn new(
        host: &'a mut dyn UiFocusActionHost,
        action_cx: ActionCx,
        binding: &'a NodeGraphSurfaceBinding,
    ) -> Self {
        Self {
            host,
            action_cx,
            binding,
        }
    }

    /// Returns the app-facing binding without exposing raw model-store mutation.
    pub fn binding(&self) -> &NodeGraphSurfaceBinding {
        self.binding
    }

    pub fn action_cx(&self) -> ActionCx {
        self.action_cx
    }

    pub fn window(&self) -> AppWindowId {
        self.action_cx.window
    }

    pub fn surface_target(&self) -> GlobalElementId {
        self.action_cx.target
    }

    pub fn graph_snapshot(&mut self) -> Option<Graph> {
        let store = self.binding.store_model();
        self.host
            .models_mut()
            .read(&store, |store| store.graph().clone())
            .ok()
    }

    pub fn view_state_snapshot(&mut self) -> Option<NodeGraphViewState> {
        let store = self.binding.store_model();
        self.host
            .models_mut()
            .read(&store, |store| store.view_state().clone())
            .ok()
    }

    pub fn dispatch_transaction(
        &mut self,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, NodeGraphControllerError> {
        self.binding.dispatch_transaction_action_host(self.host, tx)
    }

    pub fn replace_view_state(
        &mut self,
        view_state: NodeGraphViewState,
    ) -> Result<(), NodeGraphControllerError> {
        self.binding
            .replace_view_state_action_host(self.host, view_state)
    }

    pub fn request_focus_to_surface(&mut self) {
        self.host.request_focus(self.action_cx.target);
    }

    pub fn request_redraw(&mut self) {
        self.host.request_redraw(self.action_cx.window);
    }

    pub fn notify(&mut self) {
        self.host.notify(self.action_cx);
    }
}

/// Declarative surface interaction hook.
///
/// Hooks are policy seams over the default declarative surface. Graph edits must still flow through
/// the store-first context instead of raw model mutation.
pub trait NodeGraphDeclarativeInteractionHook: 'static {
    fn handle_key_down(
        &mut self,
        _ctx: &mut NodeGraphDeclarativeInteractionContext<'_>,
        _key: KeyDownCx,
    ) -> NodeGraphDeclarativeInteractionOutcome {
        NodeGraphDeclarativeInteractionOutcome::NotHandled
    }

    fn handle_insert_node_picker_request(
        &mut self,
        _ctx: &mut NodeGraphDeclarativeInteractionContext<'_>,
        _request: NodeGraphDeclarativeInsertNodePickerRequest,
    ) -> NodeGraphDeclarativeInteractionOutcome {
        NodeGraphDeclarativeInteractionOutcome::NotHandled
    }
}

pub type NodeGraphDeclarativeInteractionHookRef =
    Rc<RefCell<dyn NodeGraphDeclarativeInteractionHook>>;
