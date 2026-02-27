use super::{WindowLogicalSize, WindowPosition};

#[derive(Debug, Clone)]
pub struct WindowCreateSpec {
    pub title: String,
    pub size: WindowLogicalSize,
    pub position: Option<WindowPosition>,
    pub visible: bool,
}

impl WindowCreateSpec {
    pub fn new(title: impl Into<String>, size: WindowLogicalSize) -> Self {
        Self {
            title: title.into(),
            size,
            position: None,
            visible: true,
        }
    }

    pub fn with_position(mut self, position: WindowPosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}
