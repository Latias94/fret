#![cfg(feature = "accesskit")]

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use accesskit::{ActionRequest, TreeUpdate};
use accesskit_winit::Adapter;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

pub use fret_a11y_accesskit::{
    ScrollByData, SetTextSelectionData, SetValueData, StepperAction, focus_target_from_action,
    invoke_target_from_action, replace_selected_text_from_action, scroll_by_from_action,
    set_text_selection_from_action, set_value_from_action, stepper_target_from_action,
    tree_update_from_snapshot,
};

pub struct WinitAccessibility {
    adapter: Adapter,
    actions_rx: mpsc::Receiver<ActionRequest>,
    pending_actions: Arc<Mutex<Vec<ActionRequest>>>,
    activation_requested: Arc<AtomicBool>,
    is_active: Arc<AtomicBool>,
}

impl WinitAccessibility {
    pub fn new(event_loop: &dyn ActiveEventLoop, window: &dyn Window) -> Self {
        let (actions_tx, actions_rx) = mpsc::channel::<ActionRequest>();
        let pending_actions: Arc<Mutex<Vec<ActionRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let activation_requested: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let is_active: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let pending_actions_for_handler = pending_actions.clone();
        let actions_tx_for_handler = actions_tx.clone();
        let activation_requested_for_handler = activation_requested.clone();
        let is_active_for_handler = is_active.clone();

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

        let adapter = Adapter::with_direct_handlers(
            event_loop,
            window,
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
        );

        Self {
            adapter,
            actions_rx,
            pending_actions,
            activation_requested,
            is_active,
        }
    }

    pub fn process_event(&mut self, window: &dyn Window, event: &WindowEvent) {
        self.adapter.process_event(window, event);
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
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
}
