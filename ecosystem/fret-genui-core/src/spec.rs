//! Spec types for GenUI.
//!
//! The spec is intentionally flat (root key + elements map) to be LLM-friendly.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ElementKey(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecV1 {
    pub schema_version: u32,
    pub root: ElementKey,
    pub elements: BTreeMap<ElementKey, ElementV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementV1 {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(default)]
    pub props: serde_json::Map<String, Value>,
    #[serde(default)]
    pub children: Vec<ElementKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<crate::visibility::VisibilityConditionV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on: Option<BTreeMap<String, OnBindingV1>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<RepeatV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OnBindingV1 {
    One(ActionBindingV1),
    Many(Vec<ActionBindingV1>),
}

impl OnBindingV1 {
    pub fn iter(&self) -> impl Iterator<Item = &ActionBindingV1> {
        match self {
            Self::One(v) => std::slice::from_ref(v).iter(),
            Self::Many(v) => v.iter(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionBindingV1 {
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Map<String, Value>>,
    // Forward-compat fields (app-owned; not interpreted by core in v1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "onSuccess")]
    pub on_success: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "onError")]
    pub on_error: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepeatV1 {
    #[serde(rename = "statePath")]
    pub state_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}
