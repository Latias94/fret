use std::sync::Arc;

use fret_app::App;
use fret_genui_core::executor::{GenUiActionExecutorV1, GenUiExecError};
use fret_genui_core::form_validation::{
    ValidationIssueV1, ValidationRegistryV1, ValidationStateV1, validate_all,
};
use fret_genui_core::json_pointer;
use fret_genui_core::render::GenUiActionInvocation;
use fret_ui::GlobalElementId;
use fret_ui::action::UiActionHostAdapter;
use serde_json::{Value, json};

fn inv(action: &str, params: Value) -> GenUiActionInvocation {
    GenUiActionInvocation {
        window: Default::default(),
        source: GlobalElementId(1),
        element_key: Arc::from("k"),
        event: Arc::from("press"),
        action: Arc::from(action),
        params,
        confirm: None,
        on_success: None,
        on_error: None,
        repeat_base_path: None,
        repeat_index: None,
    }
}

#[test]
fn executor_gates_submit_and_records_validation_issues() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({
        "form": { "email": "" },
        "validation": { "issues": [] },
        "lastResult": ""
    }));
    let validation = app.models_mut().insert(ValidationStateV1::default());

    let registry = ValidationRegistryV1::new().with_validator(Arc::new(|st| {
        let email = json_pointer::get_opt(st, "/form/email")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if email.is_empty() {
            return vec![ValidationIssueV1 {
                path: "/form/email".to_string(),
                code: "required".to_string(),
                message: "Email is required.".to_string(),
            }];
        }
        Vec::new()
    }));

    let mut exec = GenUiActionExecutorV1::new(state.clone()).with_standard_actions();
    let validation_model = validation.clone();
    #[allow(clippy::arc_with_non_send_sync)]
    let handler = Arc::new(
        move |host: &mut dyn fret_ui::action::UiActionHost,
              state: &fret_runtime::Model<Value>,
              _inv: &GenUiActionInvocation| {
            let snapshot = host
                .models_mut()
                .read(state, Clone::clone)
                .ok()
                .unwrap_or(Value::Null);
            let out = validate_all(&snapshot, &registry);
            let ok = out.is_ok();

            let issues_for_state = Value::Array(
                out.issues
                    .iter()
                    .map(|i| {
                        let mut obj = serde_json::Map::new();
                        obj.insert("path".to_string(), Value::String(i.path.clone()));
                        obj.insert("code".to_string(), Value::String(i.code.clone()));
                        obj.insert("message".to_string(), Value::String(i.message.clone()));
                        Value::Object(obj)
                    })
                    .collect::<Vec<_>>(),
            );

            let _ = host.models_mut().update(&validation_model, |v| *v = out);
            let _ = host.models_mut().update(state, |v| {
                let _ = json_pointer::set(v, "/validation/issues", issues_for_state.clone());
                let _ = json_pointer::set(v, "/validation/hasErrors", Value::Bool(!ok));
            });
            if ok {
                Ok(())
            } else {
                Err(GenUiExecError::HandlerFailed {
                    message: "validation failed".to_string(),
                })
            }
        },
    );
    exec.register_handler("formSubmit", handler);

    let mut host = UiActionHostAdapter { app: &mut app };
    let mut i = inv("formSubmit", json!({ "formName": "Demo" }));
    i.on_error = Some(
        json!({ "action": "setState", "params": { "statePath": "/lastResult", "value": "error" } }),
    );

    let out = exec.execute_invocation(&mut host, &i);
    assert!(matches!(
        out,
        fret_genui_core::executor::GenUiActionOutcome::Error(_)
    ));
    assert_eq!(
        host.app.models().get_cloned(&validation).unwrap().count(),
        1
    );
    assert_eq!(
        host.app.models().get_cloned(&state),
        Some(json!({
            "form": { "email": "" },
            "validation": {
                "issues": [
                    { "path": "/form/email", "code": "required", "message": "Email is required." }
                ],
                "hasErrors": true
            },
            "lastResult": "error"
        }))
    );
}

#[test]
fn executor_allows_submit_when_valid_and_clears_issues() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({
        "form": { "email": "a@b.com" },
        "validation": {
            "issues": [
                { "path": "/form/email", "code": "required", "message": "stale" }
            ],
            "hasErrors": true
        },
        "lastResult": ""
    }));
    let validation = app.models_mut().insert(ValidationStateV1::default());

    let registry = ValidationRegistryV1::new().with_validator(Arc::new(|st| {
        let email = json_pointer::get_opt(st, "/form/email")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if email.is_empty() {
            return vec![ValidationIssueV1 {
                path: "/form/email".to_string(),
                code: "required".to_string(),
                message: "Email is required.".to_string(),
            }];
        }
        Vec::new()
    }));

    let mut exec = GenUiActionExecutorV1::new(state.clone()).with_standard_actions();
    let validation_model = validation.clone();
    #[allow(clippy::arc_with_non_send_sync)]
    let handler = Arc::new(
        move |host: &mut dyn fret_ui::action::UiActionHost,
              state: &fret_runtime::Model<Value>,
              _inv: &GenUiActionInvocation| {
            let snapshot = host
                .models_mut()
                .read(state, Clone::clone)
                .ok()
                .unwrap_or(Value::Null);
            let out = validate_all(&snapshot, &registry);
            let ok = out.is_ok();

            let issues_for_state = Value::Array(
                out.issues
                    .iter()
                    .map(|i| {
                        let mut obj = serde_json::Map::new();
                        obj.insert("path".to_string(), Value::String(i.path.clone()));
                        obj.insert("code".to_string(), Value::String(i.code.clone()));
                        obj.insert("message".to_string(), Value::String(i.message.clone()));
                        Value::Object(obj)
                    })
                    .collect::<Vec<_>>(),
            );

            let _ = host.models_mut().update(&validation_model, |v| *v = out);
            let _ = host.models_mut().update(state, |v| {
                let _ = json_pointer::set(v, "/validation/issues", issues_for_state.clone());
                let _ = json_pointer::set(v, "/validation/hasErrors", Value::Bool(!ok));
            });
            if ok {
                Ok(())
            } else {
                Err(GenUiExecError::HandlerFailed {
                    message: "validation failed".to_string(),
                })
            }
        },
    );
    exec.register_handler("formSubmit", handler);

    let mut host = UiActionHostAdapter { app: &mut app };
    let mut i = inv("formSubmit", json!({ "formName": "Demo" }));
    i.on_success = Some(
        json!({ "action": "setState", "params": { "statePath": "/lastResult", "value": "ok" } }),
    );

    let out = exec.execute_invocation(&mut host, &i);
    assert_eq!(out, fret_genui_core::executor::GenUiActionOutcome::Applied);
    assert_eq!(
        host.app.models().get_cloned(&validation).unwrap().count(),
        0
    );
    assert_eq!(
        host.app.models().get_cloned(&state),
        Some(json!({
            "form": { "email": "a@b.com" },
            "validation": { "issues": [], "hasErrors": false },
            "lastResult": "ok"
        }))
    );
}
