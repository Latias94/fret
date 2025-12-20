use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyPathSegment {
    Field(Arc<str>),
    Index(u32),
    Key(Arc<str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyPath(pub Vec<PropertyPathSegment>);

impl PropertyPath {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn field(mut self, name: impl Into<Arc<str>>) -> Self {
        self.0.push(PropertyPathSegment::Field(name.into()));
        self
    }

    pub fn matches_fields(&self, fields: &[&str]) -> bool {
        if self.0.len() != fields.len() {
            return false;
        }
        for (seg, field) in self.0.iter().zip(fields.iter().copied()) {
            match seg {
                PropertyPathSegment::Field(name) if name.as_ref() == field => {}
                _ => return false,
            }
        }
        true
    }

    pub fn starts_with(&self, prefix: &PropertyPath) -> bool {
        if prefix.0.len() > self.0.len() {
            return false;
        }
        self.0
            .iter()
            .take(prefix.0.len())
            .zip(prefix.0.iter())
            .all(|(a, b)| a == b)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Bool(bool),
    String(String),
    F32(f32),
    Vec3([f32; 3]),
    Mixed,
}

impl PropertyValue {
    pub fn as_display_string(&self) -> String {
        match self {
            PropertyValue::Bool(v) => v.to_string(),
            PropertyValue::String(v) => v.clone(),
            PropertyValue::F32(v) => format!("{v:.3}"),
            PropertyValue::Vec3([x, y, z]) => format!("{x:.3}, {y:.3}, {z:.3}"),
            PropertyValue::Mixed => "—".to_string(),
        }
    }
}
