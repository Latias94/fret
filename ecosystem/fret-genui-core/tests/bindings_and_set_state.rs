use fret_genui_core::actions;
use fret_genui_core::props::{PropResolutionContext, resolve_bindings, resolve_value};
use fret_genui_core::visibility::RepeatScope;
use serde_json::{Map, Value, json};

#[test]
fn bind_state_extracts_binding_paths() {
    let state = json!({"name": "Ada", "enabled": true});
    let ctx = PropResolutionContext {
        state: &state,
        repeat: RepeatScope::default(),
    };

    let props: Map<String, Value> = serde_json::from_value(json!({
        "value": {"$bindState": "/name"},
        "checked": {"$bindState": "/enabled"},
        "placeholder": "Type..."
    }))
    .unwrap();

    let bindings = resolve_bindings(&props, &ctx);
    assert_eq!(bindings.get("value").map(|s| s.as_str()), Some("/name"));
    assert_eq!(
        bindings.get("checked").map(|s| s.as_str()),
        Some("/enabled")
    );
}

#[test]
fn resolve_value_reads_bind_state_and_state_refs() {
    let state = json!({"name": "Ada"});
    let ctx = PropResolutionContext {
        state: &state,
        repeat: RepeatScope::default(),
    };

    assert_eq!(
        resolve_value(&json!({"$bindState": "/name"}), &ctx),
        json!("Ada")
    );
    assert_eq!(
        resolve_value(&json!({"$state": "/name"}), &ctx),
        json!("Ada")
    );
}

#[test]
fn set_state_action_updates_json_pointer_path() {
    let mut state = json!({"name": "Ada"});
    let applied = actions::apply_standard_action(
        &mut state,
        "setState",
        &json!({"statePath": "/name", "value": "Grace"}),
    );
    assert!(applied);
    assert_eq!(state, json!({"name": "Grace"}));
}
