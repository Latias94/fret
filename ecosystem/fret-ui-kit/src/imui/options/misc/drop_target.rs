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
