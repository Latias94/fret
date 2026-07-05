use fret_core::Size;

use super::behavior::FloatingWindowOptions;
use super::resize::FloatingWindowResizeOptions;

#[derive(Debug, Clone, Default)]
pub struct WindowOptions {
    /// Optional `open` model controlling whether the window is rendered.
    ///
    /// When present, close actions update the model to `false`.
    pub open: Option<fret_runtime::Model<bool>>,
    /// Optional fixed initial size for the floating window.
    ///
    /// When absent, the window uses content-driven sizing and `resize` is ignored.
    pub size: Option<Size>,
    /// Optional resize policy for sized windows.
    ///
    /// This only takes effect when `size` is also set.
    pub resize: Option<FloatingWindowResizeOptions>,
    /// Behavior flags for the floating window surface.
    pub behavior: FloatingWindowOptions,
}

impl WindowOptions {
    pub fn with_open(mut self, open: impl crate::imui::IntoImUiBoolModel) -> Self {
        self.open = Some(open.into_imui_bool_model());
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_resize(mut self, resize: FloatingWindowResizeOptions) -> Self {
        self.resize = Some(resize);
        self
    }

    pub fn with_behavior(mut self, behavior: FloatingWindowOptions) -> Self {
        self.behavior = behavior;
        self
    }
}
