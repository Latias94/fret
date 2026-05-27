mod resize;

pub use resize::{ChildRegionResizeXResponse, ChildRegionResizeYResponse};

#[derive(Debug, Clone, Default)]
pub struct ChildRegionResponse {
    pub(crate) resize_x: ChildRegionResizeXResponse,
    pub(crate) resize_y: ChildRegionResizeYResponse,
}

impl ChildRegionResponse {
    pub(crate) fn empty() -> Self {
        Self::default()
    }

    pub(crate) fn resize_y_mut(&mut self) -> &mut ChildRegionResizeYResponse {
        &mut self.resize_y
    }

    pub(crate) fn resize_x_mut(&mut self) -> &mut ChildRegionResizeXResponse {
        &mut self.resize_x
    }

    pub fn resize_x(&self) -> &ChildRegionResizeXResponse {
        &self.resize_x
    }

    pub fn resize_y(&self) -> &ChildRegionResizeYResponse {
        &self.resize_y
    }

    pub fn resizing_x(&self) -> bool {
        self.resize_x.dragging()
    }

    pub fn resizing_y(&self) -> bool {
        self.resize_y.dragging()
    }
}
