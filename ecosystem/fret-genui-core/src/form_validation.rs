//! Form validation helpers (app-owned policy hooks).
//!
//! This module is intentionally data-only:
//! - It does not know about UI components.
//! - It does not enforce any particular form schema.
//! - Apps/ecosystem layers provide validators and decide how to present issues.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssueV1 {
    /// JSON Pointer path (e.g. "/form/email").
    pub path: String,
    /// Stable issue code (app-defined or shared conventions).
    pub code: String,
    /// Human-readable message (typically English).
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationStateV1 {
    pub issues: Vec<ValidationIssueV1>,
}

impl ValidationStateV1 {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn count(&self) -> usize {
        self.issues.len()
    }

    pub fn by_path(&self) -> BTreeMap<String, Vec<&ValidationIssueV1>> {
        let mut out: BTreeMap<String, Vec<&ValidationIssueV1>> = BTreeMap::new();
        for issue in &self.issues {
            out.entry(issue.path.clone()).or_default().push(issue);
        }
        out
    }
}

pub type ValidationFnV1 =
    std::sync::Arc<dyn Fn(&Value) -> Vec<ValidationIssueV1> + Send + Sync + 'static>;

#[derive(Clone, Default)]
pub struct ValidationRegistryV1 {
    validators: Vec<ValidationFnV1>,
}

impl ValidationRegistryV1 {
    pub fn new() -> Self {
        Self { validators: vec![] }
    }

    pub fn with_validator(mut self, f: ValidationFnV1) -> Self {
        self.validators.push(f);
        self
    }
}

pub fn validate_all(state: &Value, registry: &ValidationRegistryV1) -> ValidationStateV1 {
    let mut issues: Vec<ValidationIssueV1> = Vec::new();
    for v in registry.validators.iter() {
        issues.extend(v(state));
    }
    ValidationStateV1 { issues }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_all_aggregates_issues() {
        let registry = ValidationRegistryV1::new()
            .with_validator(std::sync::Arc::new(|_state| {
                vec![ValidationIssueV1 {
                    path: "/form/email".to_string(),
                    code: "required".to_string(),
                    message: "Email is required.".to_string(),
                }]
            }))
            .with_validator(std::sync::Arc::new(|_state| {
                vec![ValidationIssueV1 {
                    path: "/name".to_string(),
                    code: "min_len".to_string(),
                    message: "Name must not be empty.".to_string(),
                }]
            }));

        let st = json!({"name": "", "form": {"email": ""}});
        let out = validate_all(&st, &registry);
        assert_eq!(out.count(), 2);
        assert!(!out.is_ok());
        let grouped = out.by_path();
        assert_eq!(grouped.get("/name").map(|v| v.len()), Some(1));
        assert_eq!(grouped.get("/form/email").map(|v| v.len()), Some(1));
    }
}
