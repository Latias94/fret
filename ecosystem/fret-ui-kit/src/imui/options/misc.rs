use std::sync::Arc;

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

#[derive(Debug, Clone, Copy)]
pub struct DropTargetOptions {
    /// When false, the target ignores active drags and never reports preview/delivery.
    pub enabled: bool,
}

impl Default for DropTargetOptions {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SeparatorTextOptions {
    pub test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, Default)]
pub struct BulletTextOptions {
    pub test_id: Option<Arc<str>>,
}
