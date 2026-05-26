use std::sync::Arc;

use fret_ui::GlobalElementId;

use super::super::hover::ResponseExt;

#[derive(Debug, Clone)]
pub struct InputTextPickerResponse {
    pub(crate) input: ResponseExt,
    pub(crate) open: bool,
    pub(crate) picked_index: Option<usize>,
    pub(crate) picked: Option<Arc<str>>,
}

impl InputTextPickerResponse {
    pub fn id(&self) -> Option<GlobalElementId> {
        self.input.id()
    }

    pub fn response(&self) -> ResponseExt {
        self.input
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn changed(&self) -> bool {
        self.input.changed()
    }

    pub fn picked(&self) -> Option<&str> {
        self.picked.as_deref()
    }

    pub fn picked_index(&self) -> Option<usize> {
        self.picked_index
    }
}
