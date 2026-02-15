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
    let state = app
        .models_mut()
        .insert(json!({ "form": { "email": "" }, "lastResult": "" }));
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
    exec.register_handler(
        "formSubmit",
        Arc::new(move |host, state, _inv| {
            let snapshot = host
                .models_mut()
                .read(state, Clone::clone)
                .ok()
                .unwrap_or(Value::Null);
            let out = validate_all(&snapshot, &registry);
            let ok = out.is_ok();
            let _ = host.models_mut().update(&validation_model, |v| *v = out);
            if ok {
                Ok(())
            } else {
                Err(GenUiExecError::HandlerFailed {
                    message: "validation failed".to_string(),
                })
            }
        }),
    );

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
        Some(json!({ "form": { "email": "" }, "lastResult": "error" }))
    );
}

#[test]
fn executor_allows_submit_when_valid_and_clears_issues() {
    let mut app = App::new();
    let state = app
        .models_mut()
        .insert(json!({ "form": { "email": "a@b.com" }, "lastResult": "" }));
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
    exec.register_handler(
        "formSubmit",
        Arc::new(move |host, state, _inv| {
            let snapshot = host
                .models_mut()
                .read(state, Clone::clone)
                .ok()
                .unwrap_or(Value::Null);
            let out = validate_all(&snapshot, &registry);
            let ok = out.is_ok();
            let _ = host.models_mut().update(&validation_model, |v| *v = out);
            if ok {
                Ok(())
            } else {
                Err(GenUiExecError::HandlerFailed {
                    message: "validation failed".to_string(),
                })
            }
        }),
    );

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
        Some(json!({ "form": { "email": "a@b.com" }, "lastResult": "ok" }))
    );
}
