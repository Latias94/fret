#[derive(Debug, Clone, Copy)]
pub(in crate::imui) struct ActiveTriggerBehaviorOptions {
    pub(in crate::imui) primary_active: bool,
    pub(in crate::imui) request_focus_on_press: bool,
    pub(in crate::imui) clear_pointer_move: bool,
}

impl Default for ActiveTriggerBehaviorOptions {
    fn default() -> Self {
        Self {
            primary_active: true,
            request_focus_on_press: true,
            clear_pointer_move: false,
        }
    }
}
