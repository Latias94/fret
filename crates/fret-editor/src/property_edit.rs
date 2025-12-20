use crate::property::{PropertyPath, PropertyValue};
use fret_core::AppWindowId;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyEditKind {
    Commit,
}

#[derive(Debug, Clone)]
pub struct PropertyEditRequest {
    pub targets: Vec<u64>,
    pub path: PropertyPath,
    pub value: PropertyValue,
    #[allow(dead_code)]
    pub kind: PropertyEditKind,
}

#[derive(Default)]
pub struct PropertyEditService {
    pending: HashMap<AppWindowId, PropertyEditRequest>,
}

impl PropertyEditService {
    pub fn set(&mut self, window: AppWindowId, request: PropertyEditRequest) {
        self.pending.insert(window, request);
    }

    pub fn take(&mut self, window: AppWindowId) -> Option<PropertyEditRequest> {
        self.pending.remove(&window)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self, window: AppWindowId) {
        self.pending.remove(&window);
    }
}
