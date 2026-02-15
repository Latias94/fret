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
                for prop in component.props.keys() {
                    out.push_str("  - ");
                    out.push_str(prop);
                    out.push('\n');
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
                    if let Some(desc) = v.description.as_deref() {
                        out.push_str(" (");
                        out.push_str(desc);
                        out.push(')');
                    }
                    out.push('\n');
                }
            }
        }

        out.push_str("\nRules:\n");
        out.push_str("- Use unique element keys in `elements`.\n");
        out.push_str("- Every child key must exist in `elements`.\n");
        out.push_str("- Prefer a small, readable tree over many tiny nodes.\n");
        out.push_str("- Do not use components/actions outside the catalog.\n");
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
}

impl CatalogPropV1 {
    pub fn new() -> Self {
        Self { description: None }
    }
}

impl Default for CatalogPropV1 {
    fn default() -> Self {
        Self::new()
    }
}

fn action_binding_schema(catalog: &CatalogV1) -> Value {
    let actions = catalog.actions.keys().cloned().collect::<Vec<_>>();
    json!({
      "type": "object",
      "additionalProperties": false,
      "required": ["action"],
      "properties": {
        "action": { "type": "string", "enum": actions },
        "params": { "type": "object", "additionalProperties": {} },
        "confirm": {},
        "onSuccess": {},
        "onError": {}
      }
    })
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
        .keys()
        .map(|k| (k.clone(), json!({})))
        .collect::<serde_json::Map<_, _>>();

    json!({
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "props", "children"],
      "properties": {
        "type": { "const": name },
        "props": {
          "type": "object",
          "additionalProperties": false,
          "properties": props_properties
        },
        "children": { "type": "array", "items": { "type": "string" } },
        "visible": {},
        "on": on_schema(catalog),
        "repeat": repeat_schema()
      }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_schema_includes_component_and_action_enums() {
        let mut catalog = CatalogV1::new();
        catalog
            .components
            .insert("Text".to_string(), CatalogComponentV1::default());
        catalog.actions.insert(
            "setState".to_string(),
            CatalogActionV1 {
                description: None,
                params: BTreeMap::new(),
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
        assert!(!elements.is_empty());

        let text_variant = elements.iter().find(|v| {
            v.get("properties")
                .and_then(|p| p.get("type"))
                .and_then(|t| t.get("const"))
                .and_then(|v| v.as_str())
                == Some("Text")
        });
        assert!(text_variant.is_some());

        let action_enum: Vec<String> = schema
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
            .and_then(|binding| binding.get("properties"))
            .and_then(|p| p.get("action"))
            .and_then(|a| a.get("enum"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        assert!(action_enum.iter().any(|v| v == "setState"));
    }
}
