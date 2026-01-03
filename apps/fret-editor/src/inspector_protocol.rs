use crate::property::{PropertyPath, PropertyValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyTypeTag(pub &'static str);

impl PropertyTypeTag {
    pub const fn new(tag: &'static str) -> Self {
        Self(tag)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PropertyMeta {
    #[allow(dead_code)]
    pub read_only: bool,
    #[allow(dead_code)]
    pub unit: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct PropertyLeaf {
    pub path: PropertyPath,
    pub label: String,
    pub type_tag: PropertyTypeTag,
    pub value: PropertyValue,
    pub meta: PropertyMeta,
}

#[derive(Debug, Clone)]
pub enum PropertyNode {
    Group {
        label: String,
        children: Vec<PropertyNode>,
    },
    Leaf(PropertyLeaf),
}

#[derive(Debug, Clone, Default)]
pub struct PropertyTree {
    pub roots: Vec<PropertyNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorEditorKind {
    BoolToggle,
    TextPopup,
    NumberPopup,
    Vec3Popup,
    AngleDegreesPopup,
}

#[derive(Debug, Clone)]
struct EditorOverride {
    type_tag: PropertyTypeTag,
    path_prefix: Option<PropertyPath>,
    kind: InspectorEditorKind,
    order: u64,
}

#[derive(Debug, Default)]
pub struct InspectorEditorRegistry {
    overrides: Vec<EditorOverride>,
    next_order: u64,
}

impl InspectorEditorRegistry {
    #[allow(dead_code)]
    pub fn register_type(&mut self, type_tag: PropertyTypeTag, kind: InspectorEditorKind) {
        let order = self.next_order;
        self.next_order += 1;
        self.overrides.push(EditorOverride {
            type_tag,
            path_prefix: None,
            kind,
            order,
        });
    }

    pub fn register_path_prefix(
        &mut self,
        type_tag: PropertyTypeTag,
        path_prefix: PropertyPath,
        kind: InspectorEditorKind,
    ) {
        let order = self.next_order;
        self.next_order += 1;
        self.overrides.push(EditorOverride {
            type_tag,
            path_prefix: Some(path_prefix),
            kind,
            order,
        });
    }

    pub fn resolve_kind(&self, leaf: &PropertyLeaf) -> InspectorEditorKind {
        let mut best_kind: Option<InspectorEditorKind> = None;
        let mut best_prefix_len: usize = 0;
        let mut best_order: u64 = 0;

        for o in &self.overrides {
            if o.type_tag != leaf.type_tag {
                continue;
            }
            let prefix_len = match o.path_prefix.as_ref() {
                Some(prefix) => {
                    if !leaf.path.starts_with(prefix) {
                        continue;
                    }
                    prefix.0.len()
                }
                None => 0,
            };

            if best_kind.is_none()
                || prefix_len > best_prefix_len
                || (prefix_len == best_prefix_len && o.order > best_order)
            {
                best_kind = Some(o.kind);
                best_prefix_len = prefix_len;
                best_order = o.order;
            }
        }

        if let Some(kind) = best_kind {
            return kind;
        }

        match leaf.type_tag.0 {
            "bool" => InspectorEditorKind::BoolToggle,
            "string" => InspectorEditorKind::TextPopup,
            "f32" => InspectorEditorKind::NumberPopup,
            "vec3" => InspectorEditorKind::Vec3Popup,
            _ => InspectorEditorKind::TextPopup,
        }
    }

    pub fn display_value(&self, leaf: &PropertyLeaf) -> String {
        let kind = self.resolve_kind(leaf);

        match (kind, &leaf.value, leaf.meta.unit) {
            (_, PropertyValue::Mixed, _) => "—".to_string(),
            (InspectorEditorKind::AngleDegreesPopup, PropertyValue::F32(v), _) => {
                format!("{v:.1}°")
            }
            (_, v, Some(unit)) => format!("{} {unit}", v.as_display_string()),
            (_, v, None) => v.as_display_string(),
        }
    }
}
