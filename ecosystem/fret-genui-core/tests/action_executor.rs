use std::sync::Arc;

use fret_app::App;
use fret_genui_core::executor::{GenUiActionExecutorV1, GenUiActionOutcome};
use fret_genui_core::render::GenUiActionInvocation;
use fret_runtime::CommandId;
use fret_runtime::Effect;
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
fn executor_applies_standard_actions() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({ "count": 0 }));

    let mut exec = GenUiActionExecutorV1::new(state.clone()).with_standard_actions();
    let mut host = UiActionHostAdapter { app: &mut app };

    let out = exec.execute_invocation(
        &mut host,
        &inv(
            "incrementState",
            json!({ "statePath": "/count", "delta": 2 }),
        ),
    );
    assert_eq!(out, GenUiActionOutcome::Applied);
    assert_eq!(
        host.app.models().get_cloned(&state),
        Some(json!({ "count": 2 }))
    );
}

#[test]
fn executor_respects_confirm_false() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({ "name": "Ada" }));

    let mut exec = GenUiActionExecutorV1::new(state.clone()).with_standard_actions();
    let mut host = UiActionHostAdapter { app: &mut app };

    let mut i = inv(
        "setState",
        json!({ "statePath": "/name", "value": "Grace" }),
    );
    i.confirm = Some(Value::Bool(false));

    let out = exec.execute_invocation(&mut host, &i);
    assert_eq!(out, GenUiActionOutcome::Skipped);
    assert_eq!(
        host.app.models().get_cloned(&state),
        Some(json!({ "name": "Ada" }))
    );
}

#[test]
fn executor_runs_on_success_chain() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({}));

    let mut exec = GenUiActionExecutorV1::new(state.clone()).with_standard_actions();
    let mut host = UiActionHostAdapter { app: &mut app };

    let mut i = inv("setState", json!({ "statePath": "/a", "value": 1 }));
    i.on_success =
        Some(json!({ "action": "setState", "params": { "statePath": "/b", "value": 2 } }));

    let out = exec.execute_invocation(&mut host, &i);
    assert_eq!(out, GenUiActionOutcome::Applied);
    assert_eq!(
        host.app.models().get_cloned(&state),
        Some(json!({ "a": 1, "b": 2 }))
    );
}

#[test]
fn executor_runs_on_error_chain_for_failed_standard_action() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({ "count": 0 }));

    let mut exec = GenUiActionExecutorV1::new(state.clone()).with_standard_actions();
    let mut host = UiActionHostAdapter { app: &mut app };

    let mut i = inv(
        "incrementState",
        json!({ "statePath": "/count", "delta": "nope" }),
    );
    i.on_error =
        Some(json!({ "action": "setState", "params": { "statePath": "/error", "value": true } }));

    let out = exec.execute_invocation(&mut host, &i);
    assert!(matches!(out, GenUiActionOutcome::Error(_)));
    assert_eq!(
        host.app.models().get_cloned(&state),
        Some(json!({ "count": 0, "error": true }))
    );
}

#[test]
fn executor_can_emit_portable_effects() {
    let mut app = App::new();
    let state = app.models_mut().insert(Value::Null);

    let mut exec = GenUiActionExecutorV1::new(state).with_portable_effect_actions();
    let mut host = UiActionHostAdapter { app: &mut app };

    let out = exec.execute_invocation(
        &mut host,
        &inv(
            "openUrl",
            json!({ "url": "https://example.com", "target": "_blank" }),
        ),
    );
    assert_eq!(out, GenUiActionOutcome::Applied);

    let effects = host.app.flush_effects();
    assert!(
        effects.iter().any(|e| match e {
            Effect::OpenUrl { url, target, .. } =>
                url == "https://example.com" && target.as_deref() == Some("_blank"),
            _ => false,
        }),
        "expected Effect::OpenUrl, got {effects:?}"
    );
}

#[test]
fn executor_can_dispatch_commands_for_action_ids() {
    let mut app = App::new();
    let state = app.models_mut().insert(Value::Null);

    let cmd: CommandId = "app.test.dispatch_me.v1".into();
    let mut exec = GenUiActionExecutorV1::new(state).with_dispatch_command_actions([cmd.clone()]);
    let mut host = UiActionHostAdapter { app: &mut app };

    let out = exec.execute_invocation(&mut host, &inv(cmd.as_str(), json!({})));
    assert_eq!(out, GenUiActionOutcome::Applied);

    let effects = host.app.flush_effects();
    assert!(
        effects.iter().any(|e| match e {
            Effect::Command { window, command } =>
                *window == Some(Default::default()) && command.as_str() == cmd.as_str(),
            _ => false,
        }),
        "expected Effect::Command, got {effects:?}"
    );
}
