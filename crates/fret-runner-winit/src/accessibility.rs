use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use accesskit::{ActionRequest, Rect, TreeUpdate};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::Window;

pub use fret_a11y_accesskit::{
    ScrollByData, SetTextSelectionData, SetValueData, focus_target_from_action,
    invoke_target_from_action, replace_selected_text_from_action, scroll_by_from_action,
    set_text_selection_from_action, set_value_from_action, tree_update_from_snapshot,
};

struct ActivationHandlerImpl {
    requested: Arc<AtomicBool>,
    active: Arc<AtomicBool>,
}

impl accesskit::ActivationHandler for ActivationHandlerImpl {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.requested.store(true, Ordering::SeqCst);
        self.active.store(true, Ordering::SeqCst);
        None
    }
}

struct ActionHandlerImpl {
    pending: Arc<Mutex<Vec<ActionRequest>>>,
    tx: mpsc::Sender<ActionRequest>,
}

impl accesskit::ActionHandler for ActionHandlerImpl {
    fn do_action(&mut self, request: ActionRequest) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.push(request.clone());
        }
        let _ = self.tx.send(request);
    }
}

struct DeactivationHandlerImpl {
    active: Arc<AtomicBool>,
}

impl accesskit::DeactivationHandler for DeactivationHandlerImpl {
    fn deactivate_accessibility(&mut self) {
        self.active.store(false, Ordering::SeqCst);
    }
}

pub struct WinitAccessibility {
    adapter: Option<crate::accessibility_accesskit_platform::Adapter>,
    actions_rx: mpsc::Receiver<ActionRequest>,
    pending_actions: Arc<Mutex<Vec<ActionRequest>>>,
    activation_requested: Arc<AtomicBool>,
    is_active: Arc<AtomicBool>,
}

impl WinitAccessibility {
    pub fn new(event_loop: &dyn ActiveEventLoop, window: &dyn Window) -> Self {
        let _ = event_loop;

        let (actions_tx, actions_rx) = mpsc::channel::<ActionRequest>();
        let pending_actions: Arc<Mutex<Vec<ActionRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let activation_requested: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let is_active: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let pending_actions_for_handler = pending_actions.clone();
        let actions_tx_for_handler = actions_tx.clone();
        let activation_requested_for_handler = activation_requested.clone();
        let is_active_for_handler = is_active.clone();

        let adapter = window.window_handle().ok().map(|h| h.as_raw()).map(|raw| {
            crate::accessibility_accesskit_platform::Adapter::new(
                raw,
                ActivationHandlerImpl {
                    requested: activation_requested_for_handler,
                    active: is_active_for_handler.clone(),
                },
                ActionHandlerImpl {
                    pending: pending_actions_for_handler,
                    tx: actions_tx_for_handler,
                },
                DeactivationHandlerImpl {
                    active: is_active_for_handler,
                },
            )
        });

        let mut out = Self {
            adapter,
            actions_rx,
            pending_actions,
            activation_requested,
            is_active,
        };
        out.refresh_window_bounds(window);
        out
    }

    pub fn process_event(&mut self, window: &dyn Window, event: &WindowEvent) {
        match event {
            WindowEvent::Focused(is_focused) => {
                if let Some(adapter) = self.adapter.as_mut() {
                    adapter.set_focus(*is_focused);
                }
            }
            WindowEvent::Moved(_) | WindowEvent::SurfaceResized(_) => {
                self.refresh_window_bounds(window);
            }
            #[allow(unreachable_patterns)]
            WindowEvent::ScaleFactorChanged { .. } => {
                self.refresh_window_bounds(window);
            }
            _ => {}
        }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        let Some(adapter) = self.adapter.as_mut() else {
            return;
        };
        adapter.update_if_active(updater);
    }

    pub fn take_activation_request(&self) -> bool {
        self.activation_requested.swap(false, Ordering::SeqCst)
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    pub fn drain_actions(&mut self, out: &mut Vec<ActionRequest>) {
        while let Ok(req) = self.actions_rx.try_recv() {
            out.push(req);
        }
    }

    pub fn drain_actions_fallback(&mut self, out: &mut Vec<ActionRequest>) {
        if let Ok(mut pending) = self.pending_actions.lock() {
            out.append(&mut *pending);
        }
    }

    fn refresh_window_bounds(&mut self, window: &dyn Window) {
        let Some(adapter) = self.adapter.as_mut() else {
            return;
        };

        let outer_position: (_, _) = window
            .outer_position()
            .unwrap_or_default()
            .cast::<f64>()
            .into();
        let outer_size: (_, _) = window.outer_size().cast::<f64>().into();
        let inner_position: (_, _) = window.surface_position().cast::<f64>().into();
        let inner_size: (_, _) = window.surface_size().cast::<f64>().into();

        adapter.set_window_bounds(
            Rect::from_origin_size(outer_position, outer_size),
            Rect::from_origin_size(inner_position, inner_size),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use accesskit::{
        Action, ActionHandler, ActivationHandler, DeactivationHandler, NodeId, TreeId,
    };

    #[test]
    fn activation_handler_requests_initial_tree_and_sets_flags() {
        let requested = Arc::new(AtomicBool::new(false));
        let active = Arc::new(AtomicBool::new(false));
        let mut handler = ActivationHandlerImpl {
            requested: requested.clone(),
            active: active.clone(),
        };

        assert!(handler.request_initial_tree().is_none());
        assert!(requested.load(Ordering::SeqCst));
        assert!(active.load(Ordering::SeqCst));
    }

    #[test]
    fn action_handler_queues_requests() {
        let (tx, rx) = mpsc::channel::<ActionRequest>();
        let pending = Arc::new(Mutex::new(Vec::new()));
        let mut handler = ActionHandlerImpl {
            pending: pending.clone(),
            tx,
        };

        let req = ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: NodeId(1),
            data: None,
        };
        handler.do_action(req.clone());

        let pending_now = pending.lock().expect("pending actions lock");
        assert_eq!(pending_now.as_slice(), std::slice::from_ref(&req));
        drop(pending_now);

        assert_eq!(rx.try_recv().ok(), Some(req));
    }

    #[test]
    fn deactivation_handler_clears_active_flag() {
        let active = Arc::new(AtomicBool::new(true));
        let mut handler = DeactivationHandlerImpl {
            active: active.clone(),
        };

        handler.deactivate_accessibility();
        assert!(!active.load(Ordering::SeqCst));
    }
}
