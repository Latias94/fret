#[derive(Debug, Clone)]
pub struct VirtualListResponse {
    pub(crate) handle: fret_ui::scroll::VirtualListScrollHandle,
    pub(crate) rendered_range: Option<(usize, usize)>,
}

impl VirtualListResponse {
    pub fn handle(&self) -> fret_ui::scroll::VirtualListScrollHandle {
        self.handle.clone()
    }

    pub fn rendered_range(&self) -> Option<(usize, usize)> {
        self.rendered_range
    }
}
