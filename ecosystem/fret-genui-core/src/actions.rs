//! Standard action semantics (json-render-inspired).
//!
//! This module is intentionally small and stable: it defines the built-in actions whose meaning
//! we want to lock early to avoid future spec rewrites.
//!
//! The renderer does **not** apply actions automatically. Apps decide when/how to apply them
//! (e.g. immediately, batched, transactional, with permissions, etc).

use serde_json::Value;

use std::sync::atomic::{AtomicU64, Ordering};

/// Apply a built-in action to the JSON state model.
///
/// Returns `true` if the action was recognized and applied.
pub fn apply_standard_action(state: &mut Value, action: &str, params: &Value) -> bool {
    match action {
        "setState" => apply_set_state(state, params),
        "incrementState" => apply_increment_state(state, params),
        "pushState" => apply_push_state(state, params),
        "removeState" => apply_remove_state(state, params),
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

fn apply_push_state(state: &mut Value, params: &Value) -> bool {
    let Some(obj) = params.as_object() else {
        return false;
    };

    let path = obj
        .get("statePath")
        .or_else(|| obj.get("path"))
        .and_then(|v| v.as_str());
    let value = obj.get("value").cloned();
    let clear_path = obj
        .get("clearStatePath")
        .or_else(|| obj.get("clearPath"))
        .and_then(|v| v.as_str());

    let (Some(path), Some(mut value)) = (path, value) else {
        return false;
    };

    // Ensure the target is an array.
    match crate::json_pointer::get_opt(state, path) {
        Some(Value::Array(_)) => {}
        Some(Value::Null) | None => {
            if crate::json_pointer::set(state, path, Value::Array(Vec::new())).is_err() {
                return false;
            }
        }
        Some(_) => return false,
    }

    // Expand "$id" placeholders inside the pushed value (json-render convention).
    expand_id_placeholders(&mut value);

    let append_path = if path.is_empty() || path == "/" {
        "/-".to_string()
    } else {
        format!("{path}/-")
    };
    if crate::json_pointer::set(state, &append_path, value).is_err() {
        return false;
    }

    if let Some(clear_path) = clear_path {
        // Minimal clear semantics: set to empty string (common for input state).
        let _ = crate::json_pointer::set(state, clear_path, Value::String(String::new()));
    }

    true
}

fn apply_remove_state(state: &mut Value, params: &Value) -> bool {
    let Some(obj) = params.as_object() else {
        return false;
    };

    let path = obj
        .get("statePath")
        .or_else(|| obj.get("path"))
        .and_then(|v| v.as_str());
    let index = obj.get("index");

    let Some(path) = path else {
        return false;
    };

    let Some(idx) = index
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_u64().and_then(|u| i64::try_from(u).ok()))
        })
        .and_then(|v| usize::try_from(v).ok())
    else {
        return false;
    };

    let Some(arr) = crate::json_pointer::get_opt(state, path).and_then(|v| v.as_array()) else {
        return false;
    };
    if idx >= arr.len() {
        return false;
    }

    let mut next = arr.clone();
    next.remove(idx);
    crate::json_pointer::set(state, path, Value::Array(next)).is_ok()
}

static NEXT_ID_SUFFIX: AtomicU64 = AtomicU64::new(1);

fn gen_id() -> String {
    // Cheap, dependency-free unique ids:
    // - time component for global uniqueness,
    // - monotonic suffix to avoid same-tick collisions.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let suffix = NEXT_ID_SUFFIX.fetch_add(1, Ordering::Relaxed);
    format!("genui_{ts:x}_{suffix:x}")
}

fn expand_id_placeholders(v: &mut Value) {
    match v {
        Value::String(s) => {
            if s == "$id" {
                *s = gen_id();
            }
        }
        Value::Array(a) => {
            for item in a {
                expand_id_placeholders(item);
            }
        }
        Value::Object(o) => {
            for (_, val) in o.iter_mut() {
                expand_id_placeholders(val);
            }
        }
        _ => {}
    }
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

    #[test]
    fn push_state_appends_and_expands_id_and_clears_input() {
        let mut state = json!({
          "todos": [],
          "newTodoText": "Hello"
        });
        let applied = apply_standard_action(
            &mut state,
            "pushState",
            &json!({
              "statePath": "/todos",
              "value": { "id": "$id", "label": "Hello" },
              "clearStatePath": "/newTodoText"
            }),
        );
        assert!(applied);
        let todos = state.get("todos").and_then(|v| v.as_array()).unwrap();
        assert_eq!(todos.len(), 1);
        let id = todos[0].get("id").and_then(|v| v.as_str()).unwrap();
        assert_ne!(id, "$id");
        assert!(id.starts_with("genui_"));
        assert_eq!(todos[0].get("label"), Some(&json!("Hello")));
        assert_eq!(state.get("newTodoText"), Some(&json!("")));
    }

    #[test]
    fn remove_state_removes_by_index() {
        let mut state = json!({
          "todos": [ {"id": "a"}, {"id": "b"} ]
        });
        let applied = apply_standard_action(
            &mut state,
            "removeState",
            &json!({ "statePath": "/todos", "index": 0 }),
        );
        assert!(applied);
        assert_eq!(state, json!({ "todos": [ {"id": "b"} ] }));
    }
}
