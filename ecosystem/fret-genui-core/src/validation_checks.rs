//! Validation checks (json-render-inspired).
//!
//! This module provides a small, data-driven validation surface similar to json-render's
//! `ValidationConfig` + `runValidation`. It is intentionally optional and policy-light:
//! apps decide when to validate (change/blur/submit) and how to present errors.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::json_pointer;
use crate::visibility::{RepeatScope, VisibilityConditionV1, VisibilityContext, evaluate};

/// When to run validation (UI policy; apps decide how to interpret this).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidateOnV1 {
    Change,
    Blur,
    Submit,
}

/// A single validation check (built-in or app-defined).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationCheckV1 {
    /// Check type (e.g. "required", "email", "minLength").
    #[serde(rename = "type")]
    pub ty: String,
    /// Optional arguments for the check (literal values or `{ "$state": "/path" }` refs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<BTreeMap<String, Value>>,
    /// Message used when the check fails.
    pub message: String,
}

/// Validation configuration for a field/path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ValidationConfigV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<ValidationCheckV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validate_on: Option<ValidateOnV1>,
    /// Optional visibility-like condition that enables validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<VisibilityConditionV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationCheckResultV1 {
    pub ty: String,
    pub valid: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResultV1 {
    pub valid: bool,
    pub errors: Vec<String>,
    pub checks: Vec<ValidationCheckResultV1>,
}

#[derive(Debug, Clone, Copy)]
pub struct ValidationContextV1<'a> {
    pub state: &'a Value,
}

fn resolve_dynamic_arg(arg: &Value, state: &Value) -> Value {
    let Some(obj) = arg.as_object() else {
        return arg.clone();
    };
    let Some(path) = obj.get("$state").and_then(|v| v.as_str()) else {
        return arg.clone();
    };
    json_pointer::get_opt(state, path)
        .cloned()
        .unwrap_or(Value::Null)
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.trim().is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn as_f64(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_i64().map(|i| i as f64))
        .or_else(|| v.as_u64().map(|u| u as f64))
        .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
}

fn run_builtin_check(
    ty: &str,
    value: &Value,
    args: Option<&BTreeMap<String, Value>>,
    state: &Value,
) -> bool {
    match ty {
        "required" => truthy(value),
        "email" => value
            .as_str()
            .is_some_and(|s| s.contains('@') && s.contains('.')),
        "minLength" => {
            let min = args
                .and_then(|a| a.get("min"))
                .map(|v| resolve_dynamic_arg(v, state))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            value
                .as_str()
                .is_some_and(|s| (s.chars().count() as u64) >= min)
        }
        "maxLength" => {
            let max = args
                .and_then(|a| a.get("max"))
                .map(|v| resolve_dynamic_arg(v, state))
                .and_then(|v| v.as_u64())
                .unwrap_or(u64::MAX);
            value
                .as_str()
                .is_some_and(|s| (s.chars().count() as u64) <= max)
        }
        "min" => {
            let min = args
                .and_then(|a| a.get("min"))
                .map(|v| resolve_dynamic_arg(v, state))
                .and_then(|v| as_f64(&v));
            let v = as_f64(value);
            v.zip(min).is_some_and(|(v, min)| v >= min)
        }
        "max" => {
            let max = args
                .and_then(|a| a.get("max"))
                .map(|v| resolve_dynamic_arg(v, state))
                .and_then(|v| as_f64(&v));
            let v = as_f64(value);
            v.zip(max).is_some_and(|(v, max)| v <= max)
        }
        "numeric" => as_f64(value).is_some(),
        "matches" => {
            let other = args
                .and_then(|a| a.get("other"))
                .map(|v| resolve_dynamic_arg(v, state))
                .unwrap_or(Value::Null);
            value == &other
        }
        _ => true, // Unknown checks are treated as pass (app-level checks can be layered on top).
    }
}

/// Run validation checks for a value with the provided config.
pub fn run_validation_v1(
    config: &ValidationConfigV1,
    value: &Value,
    ctx: ValidationContextV1<'_>,
) -> ValidationResultV1 {
    if let Some(enabled) = config.enabled.as_ref() {
        let vctx = VisibilityContext {
            state: ctx.state,
            repeat: RepeatScope::default(),
        };
        if !evaluate(enabled, &vctx) {
            return ValidationResultV1 {
                valid: true,
                errors: Vec::new(),
                checks: Vec::new(),
            };
        }
    }

    let mut checks_out: Vec<ValidationCheckResultV1> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for check in config.checks.iter() {
        let ok = run_builtin_check(check.ty.as_str(), value, check.args.as_ref(), ctx.state);
        checks_out.push(ValidationCheckResultV1 {
            ty: check.ty.clone(),
            valid: ok,
            message: check.message.clone(),
        });
        if !ok {
            errors.push(check.message.clone());
        }
    }

    ValidationResultV1 {
        valid: errors.is_empty(),
        errors,
        checks: checks_out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn required_fails_on_empty_string() {
        let config = ValidationConfigV1 {
            checks: vec![ValidationCheckV1 {
                ty: "required".to_string(),
                args: None,
                message: "required".to_string(),
            }],
            ..Default::default()
        };
        let out = run_validation_v1(
            &config,
            &json!(""),
            ValidationContextV1 { state: &json!({}) },
        );
        assert!(!out.valid);
        assert_eq!(out.errors, vec!["required"]);
    }

    #[test]
    fn min_length_uses_args_and_passes_when_sufficient() {
        let config = ValidationConfigV1 {
            checks: vec![ValidationCheckV1 {
                ty: "minLength".to_string(),
                args: Some([("min".to_string(), json!(3))].into_iter().collect()),
                message: "min".to_string(),
            }],
            ..Default::default()
        };
        let out = run_validation_v1(
            &config,
            &json!("abcd"),
            ValidationContextV1 { state: &json!({}) },
        );
        assert!(out.valid);
    }

    #[test]
    fn matches_can_reference_state() {
        let config = ValidationConfigV1 {
            checks: vec![ValidationCheckV1 {
                ty: "matches".to_string(),
                args: Some(
                    [("other".to_string(), json!({ "$state": "/a" }))]
                        .into_iter()
                        .collect(),
                ),
                message: "matches".to_string(),
            }],
            ..Default::default()
        };

        let out = run_validation_v1(
            &config,
            &json!(2),
            ValidationContextV1 {
                state: &json!({ "a": 2 }),
            },
        );
        assert!(out.valid);
    }

    #[test]
    fn enabled_gate_can_disable_validation() {
        let config = ValidationConfigV1 {
            checks: vec![ValidationCheckV1 {
                ty: "required".to_string(),
                args: None,
                message: "required".to_string(),
            }],
            enabled: Some(serde_json::from_value(json!({ "$state": "/enabled" })).unwrap()),
            ..Default::default()
        };

        let out = run_validation_v1(
            &config,
            &json!(""),
            ValidationContextV1 {
                state: &json!({ "enabled": false }),
            },
        );
        assert!(out.valid);
        assert!(out.errors.is_empty());
        assert!(out.checks.is_empty());
    }
}
