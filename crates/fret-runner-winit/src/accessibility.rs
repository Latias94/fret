use accesskit::{ActionRequest, TreeUpdate};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub use fret_a11y_accesskit::{
    SetTextSelectionData, SetValueData, focus_target_from_action, invoke_target_from_action,
    replace_selected_text_from_action, set_text_selection_from_action, set_value_from_action,
    tree_update_from_snapshot,
};

pub struct WinitAccessibility;

impl WinitAccessibility {
    pub fn new(_event_loop: &dyn ActiveEventLoop, _window: &dyn Window) -> Self {
        Self
    }

    pub fn process_event(&mut self, _window: &dyn Window, _event: &WindowEvent) {}

    pub fn update_if_active(&mut self, _updater: impl FnOnce() -> TreeUpdate) {}

    pub fn take_activation_request(&self) -> bool {
        false
    }

    pub fn is_active(&self) -> bool {
        false
    }

    pub fn drain_actions(&mut self, _out: &mut Vec<ActionRequest>) {}

    pub fn drain_actions_fallback(&mut self, _out: &mut Vec<ActionRequest>) {}
}
