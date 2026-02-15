//! Standard action semantics (json-render-inspired).
//!
//! This module is intentionally small and stable: it defines the built-in actions whose meaning
//! we want to lock early to avoid future spec rewrites.
//!
//! The renderer does **not** apply actions automatically. Apps decide when/how to apply them
//! (e.g. immediately, batched, transactional, with permissions, etc).

use serde_json::Value;

/// Apply a built-in action to the JSON state model.
///
/// Returns `true` if the action was recognized and applied.
pub fn apply_standard_action(state: &mut Value, action: &str, params: &Value) -> bool {
    match action {
        "setState" => apply_set_state(state, params),
        "incrementState" => apply_increment_state(state, params),
        _ => false,
    }
}

fn apply_set_state(state: &mut Value, params: &Value) -> bool {
    let Some(obj) = params.as_object() else {
        return false;
    };

    let path = obj
        .get("statePath")
        .or_else(|| obj.get("path"))
        .and_then(|v| v.as_str());
    let value = obj.get("value").cloned();

    let (Some(path), Some(value)) = (path, value) else {
        return false;
    };

    crate::json_pointer::set(state, path, value).is_ok()
}

fn apply_increment_state(state: &mut Value, params: &Value) -> bool {
    let Some(obj) = params.as_object() else {
        return false;
    };

    let path = obj
        .get("statePath")
        .or_else(|| obj.get("path"))
        .and_then(|v| v.as_str());
    let delta = obj.get("delta").or_else(|| obj.get("by"));

    let Some(path) = path else {
        return false;
    };

    let delta = delta.cloned().unwrap_or_else(|| Value::from(1));
    let Some(delta_i) = delta
        .as_i64()
        .or_else(|| delta.as_u64().and_then(|v| i64::try_from(v).ok()))
    else {
        // `incrementState` is numeric-only; reject non-numeric deltas.
        return false;
    };

    let current = crate::json_pointer::get_opt(state, path).cloned();
    let current_i = current
        .as_ref()
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_u64().and_then(|u| i64::try_from(u).ok()))
        })
        .unwrap_or(0);

    let next_i = current_i.saturating_add(delta_i);
    crate::json_pointer::set(state, path, Value::from(next_i)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn set_state_writes_value() {
        let mut state = json!({"a": {"b": 1}});
        let applied = apply_standard_action(
            &mut state,
            "setState",
            &json!({"statePath": "/a/b", "value": 2}),
        );
        assert!(applied);
        assert_eq!(state, json!({"a": {"b": 2}}));
    }

    #[test]
    fn set_state_creates_containers() {
        let mut state = json!({});
        let applied = apply_standard_action(
            &mut state,
            "setState",
            &json!({"statePath": "/x/y", "value": "z"}),
        );
        assert!(applied);
        assert_eq!(state, json!({"x": {"y": "z"}}));
    }

    #[test]
    fn increment_state_increments_existing() {
        let mut state = json!({"count": 2});
        let applied = apply_standard_action(
            &mut state,
            "incrementState",
            &json!({"statePath": "/count", "delta": 3}),
        );
        assert!(applied);
        assert_eq!(state, json!({"count": 5}));
    }

    #[test]
    fn increment_state_defaults_delta_to_one_and_creates_containers() {
        let mut state = json!({});
        let applied = apply_standard_action(
            &mut state,
            "incrementState",
            &json!({"statePath": "/nested/count"}),
        );
        assert!(applied);
        assert_eq!(state, json!({"nested": {"count": 1}}));
    }
}
