use crate::property::PropertyPath;
use crate::property::PropertyValue;
use fret_core::AppWindowId;
use fret_core::{Px, Rect};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorEditKind {
    String,
    F32,
    Vec3,
}

pub fn parse_value(kind: InspectorEditKind, input: &str) -> Option<PropertyValue> {
    match kind {
        InspectorEditKind::String => Some(PropertyValue::String(input.to_string())),
        InspectorEditKind::F32 => match input.trim().parse::<f32>() {
            Ok(v) => Some(PropertyValue::F32(v)),
            Err(_) => None,
        },
        InspectorEditKind::Vec3 => {
            let parts: Vec<&str> = input
                .split([',', ' '])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() != 3 {
                return None;
            }
            let mut v = [0.0f32; 3];
            for (i, p) in parts.iter().enumerate() {
                let Ok(f) = p.parse::<f32>() else {
                    return None;
                };
                v[i] = f;
            }
            Some(PropertyValue::Vec3(v))
        }
    }
}

#[derive(Debug, Clone)]
pub struct InspectorEditRequest {
    pub targets: Vec<u64>,
    pub path: PropertyPath,
    pub kind: InspectorEditKind,
    pub initial_text: String,
    pub anchor: Option<Rect>,
    pub preferred_width: Option<Px>,
}

#[derive(Default)]
pub struct InspectorEditService {
    requests: HashMap<AppWindowId, InspectorEditRequest>,
    error: HashMap<AppWindowId, String>,
}

impl InspectorEditService {
    pub fn set_request(&mut self, window: AppWindowId, request: InspectorEditRequest) {
        self.requests.insert(window, request);
        self.error.remove(&window);
    }

    pub fn get(&self, window: AppWindowId) -> Option<&InspectorEditRequest> {
        self.requests.get(&window)
    }

    pub fn set_error(&mut self, window: AppWindowId, message: impl Into<String>) {
        self.error.insert(window, message.into());
    }

    pub fn error(&self, window: AppWindowId) -> Option<&str> {
        self.error.get(&window).map(|s| s.as_str())
    }

    pub fn clear(&mut self, window: AppWindowId) {
        self.requests.remove(&window);
        self.error.remove(&window);
    }
}
