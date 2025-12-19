use crate::inspector_edit::{InspectorEditKind, InspectorEditRequest};
use crate::property::{PropertyPath, PropertyValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DemoTransform {
    pub position: [f32; 3],
    pub rotation_y: f32,
    pub scale: f32,
}

#[derive(Debug, Clone)]
pub struct DemoEntity {
    pub name: String,
    pub active: bool,
    pub transform: DemoTransform,
}

#[derive(Debug, Default, Clone)]
pub struct DemoWorld {
    entities: HashMap<u64, DemoEntity>,
}

impl DemoWorld {
    fn default_entity(id: u64) -> DemoEntity {
        DemoEntity {
            name: format!("Entity {id:06}"),
            active: id % 3 != 0,
            transform: DemoTransform {
                position: [
                    (id % 97) as f32 * 0.1,
                    (id % 53) as f32 * 0.1,
                    (id % 31) as f32 * 0.1,
                ],
                rotation_y: (id % 360) as f32,
                scale: 0.5 + (id % 100) as f32 * 0.01,
            },
        }
    }

    pub fn entity_mut(&mut self, id: u64) -> &mut DemoEntity {
        self.entities
            .entry(id)
            .or_insert_with(|| Self::default_entity(id))
    }

    fn entity_view(&self, id: u64) -> DemoEntity {
        self.entities
            .get(&id)
            .cloned()
            .unwrap_or_else(|| Self::default_entity(id))
    }

    pub fn get_property(&self, id: u64, path: &PropertyPath) -> Option<PropertyValue> {
        let e = self.entity_view(id);
        if path.matches_fields(&["name"]) {
            return Some(PropertyValue::String(e.name.clone()));
        }
        if path.matches_fields(&["active"]) {
            return Some(PropertyValue::Bool(e.active));
        }
        if path.matches_fields(&["transform", "position"]) {
            return Some(PropertyValue::Vec3(e.transform.position));
        }
        if path.matches_fields(&["transform", "rotation_y"]) {
            return Some(PropertyValue::F32(e.transform.rotation_y));
        }
        if path.matches_fields(&["transform", "scale"]) {
            return Some(PropertyValue::F32(e.transform.scale));
        }
        None
    }

    pub fn set_property(&mut self, id: u64, path: &PropertyPath, value: PropertyValue) -> bool {
        let e = self.entity_mut(id);
        if path.matches_fields(&["name"]) {
            if let PropertyValue::String(v) = value {
                e.name = v;
                return true;
            }
            return false;
        }
        if path.matches_fields(&["active"]) {
            if let PropertyValue::Bool(v) = value {
                e.active = v;
                return true;
            }
            return false;
        }
        if path.matches_fields(&["transform", "position"]) {
            if let PropertyValue::Vec3(v) = value {
                e.transform.position = v;
                return true;
            }
            return false;
        }
        if path.matches_fields(&["transform", "rotation_y"]) {
            if let PropertyValue::F32(v) = value {
                e.transform.rotation_y = v;
                return true;
            }
            return false;
        }
        if path.matches_fields(&["transform", "scale"]) {
            if let PropertyValue::F32(v) = value {
                e.transform.scale = v;
                return true;
            }
            return false;
        }
        false
    }

    pub fn apply_edit(&mut self, request: &InspectorEditRequest, input: &str) {
        let value = match request.kind {
            InspectorEditKind::String => PropertyValue::String(input.to_string()),
            InspectorEditKind::F32 => match input.trim().parse::<f32>() {
                Ok(v) => PropertyValue::F32(v),
                Err(_) => return,
            },
            InspectorEditKind::Vec3 => {
                let parts: Vec<&str> = input
                    .split(|c| c == ',' || c == ' ')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.len() != 3 {
                    return;
                }
                let mut v = [0.0f32; 3];
                for (i, p) in parts.iter().enumerate() {
                    let Ok(f) = p.parse::<f32>() else {
                        return;
                    };
                    v[i] = f;
                }
                PropertyValue::Vec3(v)
            }
        };

        for &id in &request.targets {
            let _ = self.set_property(id, &request.path, value.clone());
        }
    }
}
