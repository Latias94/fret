use crate::property::PropertyPath;
use fret_core::AppWindowId;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorEditKind {
    String,
    F32,
    Vec3,
}

#[derive(Debug, Clone)]
pub struct InspectorEditRequest {
    pub targets: Vec<u64>,
    pub path: PropertyPath,
    pub kind: InspectorEditKind,
    pub initial_text: String,
}

#[derive(Default)]
pub struct InspectorEditService {
    requests: HashMap<AppWindowId, InspectorEditRequest>,
}

impl InspectorEditService {
    pub fn set_request(&mut self, window: AppWindowId, request: InspectorEditRequest) {
        self.requests.insert(window, request);
    }

    pub fn get(&self, window: AppWindowId) -> Option<&InspectorEditRequest> {
        self.requests.get(&window)
    }

    pub fn clear(&mut self, window: AppWindowId) {
        self.requests.remove(&window);
    }
}
