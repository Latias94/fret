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
    const GRID_W: u64 = 64;
    const GRID_H: u64 = 36;
    const VIEWPORT_SCALE: f32 = 10.0;

    fn default_position(id: u64) -> [f32; 3] {
        if id == 0 {
            return [0.0, 0.0, 0.0];
        }

        let idx = id.saturating_sub(1);
        let x = idx % Self::GRID_W;
        let y = (idx / Self::GRID_W).min(Self::GRID_H.saturating_sub(1));

        let u = (x as f32 + 0.5) / Self::GRID_W as f32;
        let v = (y as f32 + 0.5) / Self::GRID_H as f32;

        [
            u * Self::VIEWPORT_SCALE,
            (1.0 - v) * Self::VIEWPORT_SCALE,
            0.0,
        ]
    }

    fn default_entity(id: u64) -> DemoEntity {
        DemoEntity {
            name: format!("Entity {id:06}"),
            active: id % 3 != 0,
            transform: DemoTransform {
                position: Self::default_position(id),
                rotation_y: (id % 360) as f32,
                scale: 0.5 + (id % 100) as f32 * 0.01,
            },
        }
    }

    pub fn position(&self, id: u64) -> [f32; 3] {
        self.entities
            .get(&id)
            .map(|e| e.transform.position)
            .unwrap_or_else(|| Self::default_position(id))
    }

    pub fn rotation_y(&self, id: u64) -> f32 {
        self.entities
            .get(&id)
            .map(|e| e.transform.rotation_y)
            .unwrap_or_else(|| Self::default_entity(id).transform.rotation_y)
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
        if path.matches_fields(&["transform", "position"]) {
            return Some(PropertyValue::Vec3(self.position(id)));
        }

        let e = self.entity_view(id);
        if path.matches_fields(&["name"]) {
            return Some(PropertyValue::String(e.name.clone()));
        }
        if path.matches_fields(&["active"]) {
            return Some(PropertyValue::Bool(e.active));
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

    pub fn apply_property_value(
        &mut self,
        targets: &[u64],
        path: &PropertyPath,
        value: PropertyValue,
    ) {
        for &id in targets {
            let _ = self.set_property(id, path, value.clone());
        }
    }
}
