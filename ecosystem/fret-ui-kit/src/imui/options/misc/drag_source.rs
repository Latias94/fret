#[derive(Debug, Clone, Copy)]
pub struct DragSourceOptions {
    /// When false, the helper does not publish a payload for the trigger's drag gesture.
    pub enabled: bool,
    /// When true, upgrade the trigger's runtime drag session to cross-window hover routing.
    pub cross_window: bool,
}

impl Default for DragSourceOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            cross_window: false,
        }
    }
}
