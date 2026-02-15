//! Catalog schema for GenUI (json-render-inspired).
//!
//! The catalog defines the "guardrails" for LLM-generated specs:
//! - which components exist and which prop keys they accept,
//! - which actions exist (and their intended params shape),
//! - optional descriptions and event names for prompting.
//!
//! This is intentionally small and portable. It is not tied to any particular UI backend.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CatalogValueTypeV1 {
    Any,
    String,
    Boolean,
    Number,
    Integer,
    Enum {
        values: Vec<String>,
    },
    Object {
        #[serde(default)]
        fields: BTreeMap<String, CatalogPropV1>,
        #[serde(default)]
        additional: bool,
    },
    Array {
        items: Box<CatalogPropV1>,
    },
    OneOf {
        #[serde(default)]
        variants: Vec<CatalogValueTypeV1>,
    },
}

impl Default for CatalogValueTypeV1 {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogV1 {
    pub schema_version: u32,
    #[serde(default)]
    pub components: BTreeMap<String, CatalogComponentV1>,
    #[serde(default)]
    pub actions: BTreeMap<String, CatalogActionV1>,
}

impl CatalogV1 {
    pub fn new() -> Self {
        Self {
            schema_version: 1,
            components: BTreeMap::new(),
            actions: BTreeMap::new(),
        }
    }

    /// Export a JSON Schema for `SpecV1` constrained by this catalog.
    ///
    /// Notes:
    /// - This schema focuses on guardrails (component/action names, prop keys).
    /// - Prop values are left as "any JSON" on purpose to support expressions.
    pub fn spec_json_schema(&self) -> Value {
        let component_variants = self
            .components
            .iter()
            .map(|(name, c)| component_element_schema(name, c, self))
            .collect::<Vec<_>>();

        json!({
          "$schema": "https://json-schema.org/draft/2020-12/schema",
          "title": "GenUI SpecV1",
          "type": "object",
          "additionalProperties": false,
          "required": ["schema_version", "root", "elements"],
          "properties": {
            "schema_version": { "const": 1 },
            "root": { "type": "string" },
            "elements": {
              "type": "object",
              "additionalProperties": {
                "oneOf": component_variants
              }
            },
            "state": {}
          }
        })
    }

    /// Generate a system prompt for LLMs to produce valid specs for this catalog.
    ///
    /// The output is English by repository convention (docs/comments in English).
    pub fn system_prompt(&self) -> String {
        let mut out = String::new();

        out.push_str("You are a GenUI spec generator.\n");
        out.push_str("Output a single JSON object that matches the SpecV1 shape:\n");
        out.push_str("- schema_version: 1\n");
        out.push_str("- root: string (element key)\n");
        out.push_str("- elements: object map { key -> element }\n");
        out.push_str("- state: optional JSON object/array/value\n\n");

        out.push_str("Element shape:\n");
        out.push_str("- type: component name (must be from the catalog)\n");
        out.push_str("- props: object (component-specific keys only)\n");
        out.push_str("- children: array of element keys\n");
        out.push_str("- visible: optional condition (boolean or expression object)\n");
        out.push_str("- on: optional { eventName: { action, params? } | [ ... ] }\n");
        out.push_str("- repeat: optional { statePath: \"/path\", key?: \"field\" }\n\n");

        out.push_str("Expressions (allowed inside props and action params):\n");
        out.push_str("- { \"$state\": \"/path\" }\n");
        out.push_str("- { \"$item\": \"field\" } (repeat only)\n");
        out.push_str("- { \"$index\": true } (repeat only)\n");
        out.push_str("- { \"$bindState\": \"/path\" } (two-way binding)\n");
        out.push_str("- { \"$bindItem\": \"field\" } (two-way binding, repeat only)\n");
        out.push_str("- { \"$cond\": <visible>, \"$then\": <expr>, \"$else\": <expr> }\n\n");

        out.push_str("AVAILABLE COMPONENTS:\n");
        for (name, component) in &self.components {
            out.push_str("- ");
            out.push_str(name);
            if let Some(desc) = component.description.as_deref() {
                out.push_str(": ");
                out.push_str(desc);
            }
            out.push('\n');

            if !component.props.is_empty() {
                out.push_str("  props:\n");
                for (prop, def) in &component.props {
                    out.push_str("  - ");
                    out.push_str(prop);
                    out.push_str(": ");
                    out.push_str(def.value_type.prompt_hint().as_str());
                    if def.required {
                        out.push_str(" (required)");
                    }
                    if let Some(default) = def.default.as_ref() {
                        out.push_str(" default=");
                        out.push_str(default.to_string().as_str());
                    }
                    if let Some(desc) = def.description.as_deref() {
                        out.push_str(" — ");
                        out.push_str(desc);
                    }
                    out.push('\n');

                    if let CatalogValueTypeV1::Object { fields, .. } = &def.value_type {
                        if !fields.is_empty() {
                            out.push_str("    fields:\n");
                            for (k, v) in fields {
                                out.push_str("    - ");
                                out.push_str(k);
                                out.push_str(": ");
                                out.push_str(v.value_type.prompt_hint().as_str());
                                if v.required {
                                    out.push_str(" (required)");
                                }
                                if let Some(default) = v.default.as_ref() {
                                    out.push_str(" default=");
                                    out.push_str(default.to_string().as_str());
                                }
                                if let Some(desc) = v.description.as_deref() {
                                    out.push_str(" — ");
                                    out.push_str(desc);
                                }
                                out.push('\n');
                            }
                        }
                    }
                }
            } else {
                out.push_str("  props: (none)\n");
            }

            if !component.events.is_empty() {
                out.push_str("  events:\n");
                for ev in &component.events {
                    out.push_str("  - ");
                    out.push_str(ev);
                    out.push('\n');
                }
            }
        }

        out.push_str("\nAVAILABLE ACTIONS:\n");
        for (name, action) in &self.actions {
            out.push_str("- ");
            out.push_str(name);
            if let Some(desc) = action.description.as_deref() {
                out.push_str(": ");
                out.push_str(desc);
            }
            out.push('\n');
            if !action.params.is_empty() {
                out.push_str("  params:\n");
                for (k, v) in &action.params {
                    out.push_str("  - ");
                    out.push_str(k);
                    out.push_str(": ");
                    out.push_str(v.value_type.prompt_hint().as_str());
                    if v.required {
                        out.push_str(" (required)");
                    }
                    if let Some(default) = v.default.as_ref() {
                        out.push_str(" default=");
                        out.push_str(default.to_string().as_str());
                    }
                    if let Some(desc) = v.description.as_deref() {
                        out.push_str(" — ");
                        out.push_str(desc);
                    }
                    out.push('\n');

                    if let CatalogValueTypeV1::Object { fields, .. } = &v.value_type {
                        if !fields.is_empty() {
                            out.push_str("    fields:\n");
                            for (fk, fv) in fields {
                                out.push_str("    - ");
                                out.push_str(fk);
                                out.push_str(": ");
                                out.push_str(fv.value_type.prompt_hint().as_str());
                                if fv.required {
                                    out.push_str(" (required)");
                                }
                                if let Some(default) = fv.default.as_ref() {
                                    out.push_str(" default=");
                                    out.push_str(default.to_string().as_str());
                                }
                                out.push('\n');
                            }
                        }
                    }
                }
            }
        }

        out.push_str("\nRules:\n");
        out.push_str("- Use unique element keys in `elements`.\n");
        out.push_str("- Every child key must exist in `elements`.\n");
        out.push_str("- Prefer a small, readable tree over many tiny nodes.\n");
        out.push_str("- Do not use components/actions outside the catalog.\n");
        out.push_str("- Omit optional props/params instead of setting them to null.\n");
        out
    }
}

impl Default for CatalogV1 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogComponentV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub props: BTreeMap<String, CatalogPropV1>,
    #[serde(default)]
    pub events: BTreeSet<String>,
}

impl CatalogComponentV1 {
    pub fn new() -> Self {
        Self {
            description: None,
            props: BTreeMap::new(),
            events: BTreeSet::new(),
        }
    }
}

impl Default for CatalogComponentV1 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogActionV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub params: BTreeMap<String, CatalogPropV1>,
}

impl CatalogActionV1 {
    pub fn new() -> Self {
        Self {
            description: None,
            params: BTreeMap::new(),
        }
    }
}

impl Default for CatalogActionV1 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogPropV1 {
    /// Optional human prompt hint (not used for validation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub value_type: CatalogValueTypeV1,
    #[serde(default)]
    pub nullable: bool,
    /// Whether this prop/param is required to be present (catalog validation only).
    #[serde(default)]
    pub required: bool,
    /// Optional default value for prompting/schema (not auto-applied by core).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

impl CatalogPropV1 {
    pub fn new() -> Self {
        Self {
            description: None,
            value_type: CatalogValueTypeV1::Any,
            nullable: false,
            required: false,
            default: None,
        }
    }

    pub fn any() -> Self {
        Self::new()
    }

    pub fn string() -> Self {
        Self {
            value_type: CatalogValueTypeV1::String,
            ..Self::new()
        }
    }

    pub fn boolean() -> Self {
        Self {
            value_type: CatalogValueTypeV1::Boolean,
            ..Self::new()
        }
    }

    pub fn number() -> Self {
        Self {
            value_type: CatalogValueTypeV1::Number,
            ..Self::new()
        }
    }

    pub fn integer() -> Self {
        Self {
            value_type: CatalogValueTypeV1::Integer,
            ..Self::new()
        }
    }

    pub fn enum_values(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut values = values.into_iter().map(Into::into).collect::<Vec<_>>();
        values.sort();
        values.dedup();
        Self {
            value_type: CatalogValueTypeV1::Enum { values },
            ..Self::new()
        }
    }

    pub fn object_fields(
        fields: impl IntoIterator<Item = (impl Into<String>, CatalogPropV1)>,
    ) -> Self {
        let mut map = BTreeMap::new();
        for (k, v) in fields {
            map.insert(k.into(), v);
        }
        Self {
            value_type: CatalogValueTypeV1::Object {
                fields: map,
                additional: false,
            },
            ..Self::new()
        }
    }

    pub fn object_fields_allowing_additional(
        fields: impl IntoIterator<Item = (impl Into<String>, CatalogPropV1)>,
    ) -> Self {
        let mut out = Self::object_fields(fields);
        if let CatalogValueTypeV1::Object { additional, .. } = &mut out.value_type {
            *additional = true;
        }
        out
    }

    pub fn array_of(item: CatalogPropV1) -> Self {
        Self {
            value_type: CatalogValueTypeV1::Array {
                items: Box::new(item),
            },
            ..Self::new()
        }
    }

    pub fn one_of(variants: impl IntoIterator<Item = CatalogValueTypeV1>) -> Self {
        Self {
            value_type: CatalogValueTypeV1::OneOf {
                variants: variants.into_iter().collect(),
            },
            ..Self::new()
        }
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn default_value(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }
}

impl Default for CatalogPropV1 {
    fn default() -> Self {
        Self::new()
    }
}

fn action_binding_schema(catalog: &CatalogV1) -> Value {
    let per_action = catalog
        .actions
        .iter()
        .map(|(name, action)| {
            json!({
              "type": "object",
              "additionalProperties": false,
              "required": ["action"],
              "properties": {
                "action": { "const": name },
                "params": action_params_schema(action),
                "confirm": {},
                "onSuccess": {},
                "onError": {}
              }
            })
        })
        .collect::<Vec<_>>();

    json!({ "oneOf": per_action })
}

fn on_schema(catalog: &CatalogV1) -> Value {
    let binding = action_binding_schema(catalog);
    json!({
      "type": "object",
      "additionalProperties": {
        "oneOf": [binding, { "type": "array", "items": binding }]
      }
    })
}

fn repeat_schema() -> Value {
    json!({
      "type": "object",
      "additionalProperties": false,
      "required": ["statePath"],
      "properties": {
        "statePath": { "type": "string" },
        "key": { "type": "string" }
      }
    })
}

fn component_element_schema(name: &str, c: &CatalogComponentV1, catalog: &CatalogV1) -> Value {
    let props_properties = c
        .props
        .iter()
        .map(|(k, v)| (k.clone(), dynamic_prop_schema(v)))
        .collect::<serde_json::Map<_, _>>();
    let required_props = c
        .props
        .iter()
        .filter_map(|(k, v)| v.required.then_some(k.clone()))
        .collect::<Vec<_>>();

    json!({
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "props", "children"],
      "properties": {
        "type": { "const": name },
        "props": {
          "type": "object",
          "additionalProperties": false,
          "properties": props_properties,
          "required": required_props
        },
        "children": { "type": "array", "items": { "type": "string" } },
        "visible": {},
        "on": on_schema(catalog),
        "repeat": repeat_schema()
      }
    })
}

fn action_params_schema(action: &CatalogActionV1) -> Value {
    if action.params.is_empty() {
        return json!({
          "type": "object",
          "additionalProperties": {}
        });
    }
    let props = action
        .params
        .iter()
        .map(|(k, v)| (k.clone(), dynamic_prop_schema(v)))
        .collect::<serde_json::Map<_, _>>();
    let required = action
        .params
        .iter()
        .filter_map(|(k, v)| v.required.then_some(k.clone()))
        .collect::<Vec<_>>();
    json!({
      "type": "object",
      "additionalProperties": false,
      "properties": props,
      "required": required
    })
}

fn expression_object_schema() -> Value {
    json!({
      "oneOf": [
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["$state"],
          "properties": { "$state": { "type": "string" } }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["$item"],
          "properties": { "$item": { "type": "string" } }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["$index"],
          "properties": { "$index": { "type": "boolean", "const": true } }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["$bindState"],
          "properties": { "$bindState": { "type": "string" } }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["$bindItem"],
          "properties": { "$bindItem": { "type": "string" } }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["$cond", "$then", "$else"],
          "properties": {
            "$cond": {},
            "$then": {},
            "$else": {}
          }
        }
      ]
    })
}

fn dynamic_value_schema(ty: &CatalogValueTypeV1, nullable: bool) -> Value {
    if matches!(ty, CatalogValueTypeV1::Any) {
        return json!({});
    }

    let mut base = base_value_schema(ty);
    if nullable {
        base = json!({
          "anyOf": [base, { "type": "null" }]
        });
    }
    json!({
      "anyOf": [base, expression_object_schema()]
    })
}

fn dynamic_prop_schema(def: &CatalogPropV1) -> Value {
    let mut v = dynamic_value_schema(&def.value_type, def.nullable);
    if let Some(default) = def.default.as_ref() {
        if let Some(obj) = v.as_object_mut() {
            obj.insert("default".to_string(), default.clone());
        }
    }
    v
}

fn base_value_schema(ty: &CatalogValueTypeV1) -> Value {
    match ty {
        CatalogValueTypeV1::Any => json!({}),
        CatalogValueTypeV1::String => json!({ "type": "string" }),
        CatalogValueTypeV1::Boolean => json!({ "type": "boolean" }),
        CatalogValueTypeV1::Number => json!({ "type": "number" }),
        CatalogValueTypeV1::Integer => json!({ "type": "integer" }),
        CatalogValueTypeV1::Enum { values } => json!({ "type": "string", "enum": values }),
        CatalogValueTypeV1::Object { fields, additional } => {
            let properties = fields
                .iter()
                .map(|(k, v)| (k.clone(), dynamic_prop_schema(v)))
                .collect::<serde_json::Map<_, _>>();
            let required = fields
                .iter()
                .filter_map(|(k, v)| v.required.then_some(k.clone()))
                .collect::<Vec<_>>();
            json!({
              "type": "object",
              "additionalProperties": *additional,
              "properties": properties,
              "required": required
            })
        }
        CatalogValueTypeV1::Array { items } => json!({
          "type": "array",
          "items": dynamic_prop_schema(items)
        }),
        CatalogValueTypeV1::OneOf { variants } => json!({
          "oneOf": variants.iter().map(base_value_schema).collect::<Vec<_>>()
        }),
    }
}

impl CatalogValueTypeV1 {
    pub fn prompt_hint(&self) -> String {
        match self {
            Self::Any => "any".to_string(),
            Self::String => "string".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::Number => "number".to_string(),
            Self::Integer => "integer".to_string(),
            Self::Enum { values } => format!("enum({})", values.join(" | ")),
            Self::Object { .. } => "object".to_string(),
            Self::Array { items } => format!("array<{}>", items.value_type.prompt_hint()),
            Self::OneOf { variants } => format!(
                "oneOf({})",
                variants
                    .iter()
                    .map(CatalogValueTypeV1::prompt_hint)
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogActionV1, CatalogComponentV1};

    #[test]
    fn spec_schema_includes_component_and_action_enums() {
        let mut catalog = CatalogV1::new();
        catalog
            .components
            .insert("Text".to_string(), CatalogComponentV1::default());
        catalog
            .actions
            .insert("setState".to_string(), CatalogActionV1::default());

        let schema = catalog.spec_json_schema();
        let elements = schema
            .get("properties")
            .and_then(|p| p.get("elements"))
            .and_then(|e| e.get("additionalProperties"))
            .and_then(|a| a.get("oneOf"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(!elements.is_empty());

        let text_variant = elements.iter().find(|v| {
            v.get("properties")
                .and_then(|p| p.get("type"))
                .and_then(|t| t.get("const"))
                .and_then(|v| v.as_str())
                == Some("Text")
        });
        assert!(text_variant.is_some());

        let action_variants = schema
            .get("properties")
            .and_then(|p| p.get("elements"))
            .and_then(|e| e.get("additionalProperties"))
            .and_then(|a| a.get("oneOf"))
            .and_then(|v| v.as_array())
            .and_then(|variants| variants.first())
            .and_then(|v| v.get("properties"))
            .and_then(|p| p.get("on"))
            .and_then(|on| on.get("additionalProperties"))
            .and_then(|ap| ap.get("oneOf"))
            .and_then(|v| v.as_array())
            .and_then(|oneof| oneof.first())
            .and_then(|binding| binding.get("oneOf"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        assert!(action_variants.iter().any(|v| {
            v.get("properties")
                .and_then(|p| p.get("action"))
                .and_then(|a| a.get("const"))
                .and_then(|v| v.as_str())
                == Some("setState")
        }));
    }

    #[test]
    fn spec_schema_marks_required_props_and_params() {
        let mut catalog = CatalogV1::new();
        catalog.components.insert(
            "Text".to_string(),
            CatalogComponentV1 {
                description: None,
                props: {
                    let mut p = BTreeMap::new();
                    p.insert("text".to_string(), CatalogPropV1::string().required(true));
                    p.insert(
                        "tone".to_string(),
                        CatalogPropV1::enum_values(["muted", "default"])
                            .default_value(Value::String("default".to_string())),
                    );
                    p
                },
                events: Default::default(),
            },
        );
        catalog.actions.insert(
            "doIt".to_string(),
            CatalogActionV1 {
                description: None,
                params: {
                    let mut p = BTreeMap::new();
                    p.insert("id".to_string(), CatalogPropV1::string().required(true));
                    p
                },
            },
        );

        let schema = catalog.spec_json_schema();
        let elements = schema
            .get("properties")
            .and_then(|p| p.get("elements"))
            .and_then(|e| e.get("additionalProperties"))
            .and_then(|a| a.get("oneOf"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let text_variant = elements
            .iter()
            .find(|v| {
                v.get("properties")
                    .and_then(|p| p.get("type"))
                    .and_then(|t| t.get("const"))
                    .and_then(|v| v.as_str())
                    == Some("Text")
            })
            .cloned()
            .expect("Text variant must exist");

        let required_props = text_variant
            .get("properties")
            .and_then(|p| p.get("props"))
            .and_then(|p| p.get("required"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(required_props.iter().any(|v| v.as_str() == Some("text")));

        let tone_default = text_variant
            .get("properties")
            .and_then(|p| p.get("props"))
            .and_then(|p| p.get("properties"))
            .and_then(|p| p.get("tone"))
            .and_then(|v| v.get("default"))
            .cloned()
            .unwrap_or(Value::Null);
        assert_eq!(tone_default, Value::String("default".to_string()));

        let action_variants = text_variant
            .get("properties")
            .and_then(|p| p.get("on"))
            .and_then(|on| on.get("additionalProperties"))
            .and_then(|ap| ap.get("oneOf"))
            .and_then(|v| v.as_array())
            .and_then(|oneof| oneof.first())
            .and_then(|binding| binding.get("oneOf"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let do_it = action_variants
            .iter()
            .find(|v| {
                v.get("properties")
                    .and_then(|p| p.get("action"))
                    .and_then(|a| a.get("const"))
                    .and_then(|v| v.as_str())
                    == Some("doIt")
            })
            .cloned()
            .expect("doIt action variant must exist");
        let required_params = do_it
            .get("properties")
            .and_then(|p| p.get("params"))
            .and_then(|p| p.get("required"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(required_params.iter().any(|v| v.as_str() == Some("id")));
    }
}
