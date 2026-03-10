use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_genui_core::catalog::{CatalogActionV1, CatalogV1};
use fret_genui_core::executor::{GenUiActionExecutorV1, GenUiActionOutcome};
use fret_genui_core::form_validation::{
    ValidationIssueV1, ValidationRegistryV1, ValidationStateV1, validate_all,
};
use fret_genui_core::json_pointer;
use fret_genui_core::mixed_stream::{MixedSpecStreamCompiler, MixedStreamMode, MixedStreamOptions};
use fret_genui_core::render::{GenUiActionQueue, GenUiRuntime, render_spec};
use fret_genui_core::spec::SpecV1;
use fret_genui_core::spec_fixer::{SpecFixups, auto_fix_spec};
use fret_genui_core::validate::ValidationMode;
use fret_genui_shadcn::catalog::shadcn_catalog_v1;
use fret_genui_shadcn::resolver::ShadcnResolver;
use serde_json::Value;
mod act {
    fret::actions!([
        ClearActions = "genui_demo.clear_actions.v1",
        ApplyQueuedActions = "genui_demo.apply_queued_actions.v1",
        ResetState = "genui_demo.reset_state.v1",
        ApplyEditorSpec = "genui_demo.apply_editor_spec.v1",
        ResetEditor = "genui_demo.reset_editor.v1",
        ApplyStream = "genui_demo.apply_stream.v1",
        ResetStream = "genui_demo.reset_stream.v1",
        EnableLive = "genui_demo.enable_live.v1",
        AutoApplyToggled = "genui_demo.auto_apply_toggled.v1",
    ]);
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("genui-demo")
        .window("genui-demo", (980.0, 720.0))
        .install_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Light,
            );
        })
        .run_view::<GenUiView>()
        .map_err(anyhow::Error::from)
}

const SPEC_JSON: &str = r#"
{
  "schema_version": 1,
  "root": "root",
  "elements": {
    "root": {
      "type": "VStack",
      "props": { "gap": "N3", "minW0": true },
      "children": ["header", "content_box"]
    },
    "header": { "type": "Text", "props": { "text": "GenUI Demo (json-render-inspired)", "variant": "h3" }, "children": [] },
    "content_box": {
      "type": "Box",
      "props": { "p": "N3", "wFull": true, "minW0": true },
      "children": ["card"]
    },
    "card": {
      "type": "Card",
      "props": { "wrapContent": false },
      "children": ["card_header", "card_content"]
    },
    "card_header": {
      "type": "CardHeader",
      "props": {},
      "children": ["card_title", "card_desc"]
    },
    "card_title": { "type": "CardTitle", "props": { "text": "Spec-driven UI" }, "children": [] },
    "card_desc": { "type": "CardDescription", "props": { "text": "Bindings, repeat, and standard actions." }, "children": [] },
    "card_content": {
      "type": "CardContent",
      "props": {},
      "children": ["card_stack"]
    },
    "card_stack": {
      "type": "VStack",
      "props": { "gap": "N2" },
      "children": [
        "bind_title",
        "enabled_row",
        "name_row",
        "name_buttons",
        "sep_1",
        "counter_title",
        "counter_desc",
        "counter_row",
        "sep_2",
        "todos_title",
        "todo_add_row",
        "todos_list",
        "sep_3",
        "executor_title",
        "executor_desc",
        "executor_row",
        "executor_result_row",
        "sep_4",
        "validation_title",
        "validation_desc",
        "email_row",
        "email_errors",
        "submit_row",
        "sep_5",
        "responsive_title",
        "responsive_grid_box",
        "responsive_stack_title",
        "responsive_stack_box"
      ]
    },
    "bind_title": { "type": "Text", "props": { "text": "Bindings ($bindState demo)", "variant": "large" }, "children": [] },
    "enabled_row": {
      "type": "HStack",
      "props": { "gap": "N2", "wrap": true },
      "children": ["enabled_label", "enabled_switch", "enabled_badge", "disabled_badge"]
    },
    "enabled_label": { "type": "Text", "props": { "text": "Enabled:" }, "children": [] },
    "enabled_switch": {
      "type": "Switch",
      "props": { "checked": { "$bindState": "/enabled" } },
      "children": []
    },
    "enabled_badge": {
      "type": "Badge",
      "props": { "label": "Enabled", "variant": "secondary" },
      "visible": { "$state": "/enabled" },
      "children": []
    },
    "disabled_badge": {
      "type": "Badge",
      "props": { "label": "Disabled", "variant": "outline" },
      "visible": { "$state": "/enabled", "not": true },
      "children": []
    },
    "name_row": {
      "type": "HStack",
      "props": { "gap": "N2", "wrap": true, "items": "center" },
      "children": ["name_label", "name_input", "name_value"]
    },
    "name_label": { "type": "Text", "props": { "text": "Name:" }, "children": [] },
    "name_input": {
      "type": "Input",
      "props": { "placeholder": "Type your name…", "value": { "$bindState": "/name" }, "flex1": true, "minW0": true },
      "children": []
    },
    "name_value": { "type": "Text", "props": { "text": { "$state": "/name" } }, "children": [] },
    "name_buttons": {
      "type": "HStack",
      "props": { "gap": "N2" },
      "children": ["set_name_grace", "clear_name"]
    },
    "set_name_grace": {
      "type": "Button",
      "props": { "label": "Set name = Grace" },
      "on": { "press": { "action": "setState", "params": { "statePath": "/name", "value": "Grace" } } },
      "children": []
    },
    "clear_name": {
      "type": "Button",
      "props": { "label": "Clear name" },
      "on": { "press": { "action": "setState", "params": { "statePath": "/name", "value": "" } } },
      "children": []
    },
    "sep_1": { "type": "Separator", "props": {}, "children": [] },
    "counter_title": { "type": "Text", "props": { "text": "Counter (standard actions)", "variant": "large" }, "children": [] },
    "counter_desc": { "type": "Text", "props": { "text": "Auto-apply on: updates immediately. Auto-apply off: actions only append to the queue.", "variant": "muted" }, "children": [] },
    "counter_row": {
      "type": "HStack",
      "props": { "gap": "N2", "wrap": true },
      "children": ["counter_label", "counter_value", "counter_dec", "counter_inc"]
    },
    "counter_label": { "type": "Text", "props": { "text": "Count:" }, "children": [] },
    "counter_value": {
      "type": "Badge",
      "props": { "label": { "$state": "/count" }, "variant": "secondary" },
      "children": []
    },
    "counter_dec": {
      "type": "Button",
      "props": { "label": "Decrement" },
      "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": -1 } } },
      "children": []
    },
    "counter_inc": {
      "type": "Button",
      "props": { "label": "Increment" },
      "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": 1 } } },
      "children": []
    },
    "sep_2": { "type": "Separator", "props": {}, "children": [] },
    "todos_title": { "type": "Text", "props": { "text": "Todos (repeat demo)", "variant": "large" }, "children": [] },
    "todo_add_row": {
      "type": "HStack",
      "props": { "gap": "N2", "wrap": true, "items": "center" },
      "children": ["todo_input", "todo_add_btn"]
    },
    "todo_input": {
      "type": "Input",
      "props": { "placeholder": "New todo…", "value": { "$bindState": "/newTodoText" }, "flex1": true, "minW0": true },
      "children": []
    },
    "todo_add_btn": {
      "type": "Button",
      "props": { "label": "Add" },
      "on": { "press": { "action": "pushState", "params": { "statePath": "/todos", "value": { "id": "$id", "label": { "$state": "/newTodoText" } }, "clearStatePath": "/newTodoText" } } },
      "children": []
    },
    "todos_list": {
      "type": "VStack",
      "props": { "gap": "N1" },
      "repeat": { "statePath": "/todos", "key": "id" },
      "children": ["todo_item"]
    },
    "todo_item": { "type": "HStack", "props": { "gap": "N2", "wrap": true, "items": "center" }, "children": ["todo_label", "todo_remove"] },
    "todo_label": { "type": "Text", "props": { "text": { "$item": "label" } }, "children": [] },
    "todo_remove": {
      "type": "Button",
      "props": { "label": "Remove" },
      "on": { "press": { "action": "removeState", "params": { "statePath": "/todos", "index": { "$index": true } } } },
      "children": []
    },
    "sep_3": { "type": "Separator", "props": {}, "children": [] },

    "executor_title": { "type": "Text", "props": { "text": "Executor (confirm policy + chains)", "variant": "large" }, "children": [] },
    "executor_desc": { "type": "Text", "props": { "text": "Turn auto-apply off, press buttons to enqueue actions, then use “Apply queue (executor)”. In this demo, confirm=true actions require Enabled=true (app-owned confirm policy hook).", "variant": "muted" }, "children": [] },
    "executor_row": {
      "type": "HStack",
      "props": { "gap": "N2", "wrap": true, "items": "center", "wFull": true, "minW0": true },
      "children": ["ex_inc_confirm_false", "ex_inc_confirm_true", "ex_open_url", "ex_unknown_action"]
    },
    "ex_inc_confirm_false": {
      "type": "Button",
      "props": { "label": "Inc (confirm=false)" },
      "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": 1 }, "confirm": false } },
      "children": []
    },
    "ex_inc_confirm_true": {
      "type": "Button",
      "props": { "label": "Inc (confirm=true)" },
      "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": 1 }, "confirm": true, "onSuccess": [
        { "action": "setState", "params": { "statePath": "/lastResult", "value": "increment ok" } },
        { "action": "clipboardSetText", "params": { "text": "increment ok" } }
      ] } },
      "children": []
    },
    "ex_open_url": {
      "type": "Button",
      "props": { "label": "Open URL" },
      "on": { "press": { "action": "openUrl", "params": { "url": "https://example.com" }, "confirm": true, "onSuccess": { "action": "setState", "params": { "statePath": "/lastResult", "value": "openUrl queued" } } } },
      "children": []
    },
    "ex_unknown_action": {
      "type": "Button",
      "props": { "label": "Unknown action" },
      "on": { "press": { "action": "demoUnimplemented", "params": {}, "confirm": true, "onError": [
        { "action": "setState", "params": { "statePath": "/lastResult", "value": "unknown action error" } },
        { "action": "clipboardSetText", "params": { "text": "unknown action error" } }
      ] } },
      "children": []
    },
    "executor_result_row": {
      "type": "HStack",
      "props": { "gap": "N2", "wrap": true, "items": "center", "wFull": true, "minW0": true },
      "children": ["executor_result_label", "executor_result_value"]
    },
    "executor_result_label": { "type": "Text", "props": { "text": "lastResult:", "variant": "small" }, "children": [] },
    "executor_result_value": { "type": "Badge", "props": { "label": { "$state": "/lastResult" }, "variant": "secondary" }, "children": [] },

    "sep_4": { "type": "Separator", "props": {}, "children": [] },

    "validation_title": { "type": "Text", "props": { "text": "Validation loop (submit is executor-gated)", "variant": "large" }, "children": [] },
    "validation_desc": { "type": "Text", "props": { "text": "Turn auto-apply off, press Submit form (enqueue), then click “Apply queue (executor)”. Invalid email should record issues and fail fast (onError chain).", "variant": "muted" }, "children": [] },
    "email_row": {
      "type": "HStack",
      "props": { "gap": "N2", "wrap": true, "items": "center", "wFull": true, "minW0": true },
      "children": ["email_label", "email_input", "email_value"]
    },
    "email_label": { "type": "Text", "props": { "text": "Email:" }, "children": [] },
    "email_input": {
      "type": "Input",
      "props": { "placeholder": "you@example.com", "value": { "$bindState": "/form/email" }, "flex1": true, "minW0": true },
      "children": []
    },
    "email_value": { "type": "Text", "props": { "text": { "$state": "/form/email" }, "variant": "small" }, "children": [] },
    "email_errors": {
      "type": "VStack",
      "props": { "gap": "N1" },
      "repeat": { "statePath": "/validation/issues" },
      "children": ["email_error_item"]
    },
    "email_error_item": {
      "type": "Badge",
      "props": { "label": { "$item": "message" }, "variant": "destructive" },
      "visible": { "$item": "path", "eq": "/form/email" },
      "children": []
    },
    "submit_row": {
      "type": "HStack",
      "props": { "gap": "N2", "wrap": true, "items": "center" },
      "children": ["submit_btn"]
    },
    "submit_btn": {
      "type": "Button",
      "props": { "label": "Submit form" },
      "on": { "press": { "action": "formSubmit", "params": { "formName": "Demo" }, "confirm": true,
        "onSuccess": { "action": "setState", "params": { "statePath": "/lastResult", "value": "submit ok" } },
        "onError": { "action": "setState", "params": { "statePath": "/lastResult", "value": "submit failed (see Validation tab)" } }
      } },
      "children": []
    },

    "sep_5": { "type": "Separator", "props": {}, "children": [] },
    "responsive_title": { "type": "Text", "props": { "text": "ResponsiveGrid (container query demo)", "variant": "large" }, "children": [] },
    "responsive_grid_box": {
      "type": "Box",
      "props": { "p": "N2", "wFull": true, "minW0": true },
      "children": ["responsive_grid"]
    },
    "responsive_grid": {
      "type": "ResponsiveGrid",
      "props": {
        "gap": "N2",
        "query": "container",
        "fillLastRow": true,
        "columns": { "base": 1, "md": 2, "lg": 3 }
      },
      "children": [
        "rg_card_1",
        "rg_card_2",
        "rg_card_3",
        "rg_card_4",
        "rg_card_5",
        "rg_card_6"
      ]
    },
    "rg_card_1": { "type": "Card", "props": {}, "children": ["rg_card_1_text"] },
    "rg_card_1_text": { "type": "Text", "props": { "text": "Card 1" }, "children": [] },
    "rg_card_2": { "type": "Card", "props": {}, "children": ["rg_card_2_text"] },
    "rg_card_2_text": { "type": "Text", "props": { "text": "Card 2" }, "children": [] },
    "rg_card_3": { "type": "Card", "props": {}, "children": ["rg_card_3_text"] },
    "rg_card_3_text": { "type": "Text", "props": { "text": "Card 3" }, "children": [] },
    "rg_card_4": { "type": "Card", "props": {}, "children": ["rg_card_4_text"] },
    "rg_card_4_text": { "type": "Text", "props": { "text": "Card 4" }, "children": [] },
    "rg_card_5": { "type": "Card", "props": {}, "children": ["rg_card_5_text"] },
    "rg_card_5_text": { "type": "Text", "props": { "text": "Card 5" }, "children": [] },
    "rg_card_6": { "type": "Card", "props": {}, "children": ["rg_card_6_text"] },
    "rg_card_6_text": { "type": "Text", "props": { "text": "Card 6" }, "children": [] },

    "responsive_stack_title": { "type": "Text", "props": { "text": "ResponsiveStack (container query demo)", "variant": "large" }, "children": [] },
    "responsive_stack_box": {
      "type": "Box",
      "props": { "p": "N2", "wFull": true, "minW0": true },
      "children": ["responsive_stack"]
    },
    "responsive_stack": {
      "type": "ResponsiveStack",
      "props": {
        "gap": "N2",
        "query": "container",
        "direction": { "base": "vertical", "lg": "horizontal" }
      },
      "children": ["rs_card_1", "rs_card_2", "rs_card_3"]
    },
    "rs_card_1": { "type": "Card", "props": {}, "children": ["rs_card_1_text"] },
    "rs_card_1_text": { "type": "Text", "props": { "text": "Stack card A" }, "children": [] },
    "rs_card_2": { "type": "Card", "props": {}, "children": ["rs_card_2_text"] },
    "rs_card_2_text": { "type": "Text", "props": { "text": "Stack card B" }, "children": [] },
    "rs_card_3": { "type": "Card", "props": {}, "children": ["rs_card_3_text"] },
    "rs_card_3_text": { "type": "Text", "props": { "text": "Stack card C" }, "children": [] }
  },
  "state": {
    "name": "Ada",
    "enabled": true,
    "count": 0,
    "newTodoText": "",
    "todos": [
      { "id": "a", "label": "Keep runtime mechanism-only (ADR 0066)" },
      { "id": "b", "label": "Render from a flat spec + catalog" },
      { "id": "c", "label": "Repeat scopes with stable keys" }
    ],
    "lastResult": "",
    "form": { "email": "" },
    "validation": { "issues": [] }
  }
}
"#;

struct GenUiState {
    spec: SpecV1,
    catalog: Arc<CatalogV1>,
    genui_state: Model<Value>,
    validation_state: Model<ValidationStateV1>,
    action_queue: Model<GenUiActionQueue>,
    queue_summary: Option<Arc<str>>,
    auto_apply_standard_actions: Model<bool>,
    auto_fix_on_apply: Model<bool>,
    auto_fix_summary: Option<Arc<str>>,
    editor_text: Model<String>,
    editor_error: Option<Arc<str>>,
    stream_text: Model<String>,
    stream_patch_only: Model<bool>,
    stream_summary: Option<Arc<str>>,
    stream_error: Option<Arc<str>>,
}

const TRANSIENT_GENUI_CLEAR_ACTIONS: u64 = 0x47454E5549_0001;
const TRANSIENT_GENUI_APPLY_QUEUED_ACTIONS: u64 = 0x47454E5549_0002;
const TRANSIENT_GENUI_RESET_STATE: u64 = 0x47454E5549_0003;
const TRANSIENT_GENUI_APPLY_EDITOR_SPEC: u64 = 0x47454E5549_0004;
const TRANSIENT_GENUI_RESET_EDITOR: u64 = 0x47454E5549_0005;
const TRANSIENT_GENUI_APPLY_STREAM: u64 = 0x47454E5549_0006;
const TRANSIENT_GENUI_RESET_STREAM: u64 = 0x47454E5549_0007;
const TRANSIENT_GENUI_ENABLE_LIVE: u64 = 0x47454E5549_0008;
const TRANSIENT_GENUI_AUTO_APPLY_TOGGLED: u64 = 0x47454E5549_0009;

struct GenUiView {
    st: GenUiState,
}

#[derive(Debug, Clone)]
enum Msg {
    ClearActions,
    ApplyQueuedActions,
    ResetState,
    ApplyEditorSpec,
    ResetEditor,
    ApplyStream,
    ResetStream,
    SetAutoApply(bool),
    AutoApplyToggled,
}

impl GenUiView {
    fn init_state(app: &mut KernelApp) -> GenUiState {
        let spec: SpecV1 = serde_json::from_str(SPEC_JSON).expect("SPEC_JSON must parse");
        let seed = spec.state.clone().unwrap_or(Value::Null);
        let mut catalog = shadcn_catalog_v1();
        catalog.actions.insert(
            "demoUnimplemented".to_string(),
            CatalogActionV1 {
                description: Some(
                    "Demo-only action. Intentionally unimplemented to exercise executor error paths."
                        .to_string(),
                ),
                params: Default::default(),
            },
        );
        GenUiState {
            spec,
            catalog: Arc::new(catalog),
            genui_state: app.models_mut().insert(seed),
            validation_state: app.models_mut().insert(ValidationStateV1::default()),
            action_queue: app.models_mut().insert(GenUiActionQueue::default()),
            queue_summary: None,
            auto_apply_standard_actions: app.models_mut().insert(true),
            auto_fix_on_apply: app.models_mut().insert(true),
            auto_fix_summary: None,
            editor_text: app.models_mut().insert(SPEC_JSON.to_string()),
            editor_error: None,
            stream_text: app.models_mut().insert(String::new()),
            stream_patch_only: app.models_mut().insert(false),
            stream_summary: None,
            stream_error: None,
        }
    }

    fn handle_msg(app: &mut KernelApp, state: &mut GenUiState, message: Msg) {
        match message {
            Msg::ClearActions => {
                let _ = app
                    .models_mut()
                    .update(&state.action_queue, |q| q.invocations.clear());
                state.queue_summary = None;
            }
            Msg::ApplyQueuedActions => {
                let auto_apply = app
                    .models()
                    .get_copied(&state.auto_apply_standard_actions)
                    .unwrap_or(true);
                let invocations = app
                    .models()
                    .read(&state.action_queue, |q| q.invocations.clone())
                    .ok()
                    .unwrap_or_default();

                if auto_apply {
                    state.queue_summary = Some(Arc::<str>::from(
                        "Queue apply is disabled when auto-apply is on (queue is a log). Turn auto-apply off to replay the queue via executor.",
                    ));
                    return;
                }

                let mut host = fret_ui::action::UiActionHostAdapter { app };
                let state_model = state.genui_state.clone();
                let state_model_for_confirm = state_model.clone();
                let state_model_for_submit = state_model.clone();
                let validation_model = state.validation_state.clone();
                let validation_registry =
                    Arc::new(ValidationRegistryV1::new().with_validator(Arc::new(|st| {
                        let mut issues: Vec<ValidationIssueV1> = Vec::new();
                        let email = json_pointer::get_opt(st, "/form/email")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .trim();
                        if email.is_empty() {
                            issues.push(ValidationIssueV1 {
                                path: "/form/email".to_string(),
                                code: "required".to_string(),
                                message: "Email is required.".to_string(),
                            });
                        } else if !email.contains('@') || !email.contains('.') {
                            issues.push(ValidationIssueV1 {
                                path: "/form/email".to_string(),
                                code: "format".to_string(),
                                message: "Email must look like an email address.".to_string(),
                            });
                        }
                        issues
                    })));
                let mut executor = GenUiActionExecutorV1::new(state_model.clone())
                    .confirm_policy(Box::new(move |host, inv, confirm| {
                        let requires_confirm = match confirm {
                            Value::Bool(v) => *v,
                            _ => true,
                        };
                        if !requires_confirm {
                            return true;
                        }
                        if inv.action.as_ref() == "formSubmit" {
                            return true;
                        }

                        let enabled = host
                            .models_mut()
                            .read(&state_model_for_confirm, |v| {
                                json_pointer::get_opt(v, "/enabled")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(true)
                            })
                            .ok()
                            .unwrap_or(true);
                        if enabled {
                            return true;
                        }

                        let _ = host.models_mut().update(&state_model_for_confirm, |v| {
                            let _ = json_pointer::set(
                                v,
                                "/lastResult",
                                Value::String("confirm denied (enabled=false)".to_string()),
                            );
                        });
                        false
                    }))
                    .with_standard_actions()
                    .with_portable_effect_actions();
                {
                    let validation_registry = validation_registry.clone();
                    executor.register_handler(
                        "formSubmit",
                        Arc::new(move |host, state, _inv| {
                            let current = host
                                .models_mut()
                                .read(state, Clone::clone)
                                .ok()
                                .unwrap_or(Value::Null);
                            let out = validate_all(&current, &validation_registry);
                            let ok = out.is_ok();

                            let issues_for_state = Value::Array(
                                out.issues
                                    .iter()
                                    .map(|i| {
                                        let mut obj = serde_json::Map::new();
                                        obj.insert(
                                            "path".to_string(),
                                            Value::String(i.path.clone()),
                                        );
                                        obj.insert(
                                            "code".to_string(),
                                            Value::String(i.code.clone()),
                                        );
                                        obj.insert(
                                            "message".to_string(),
                                            Value::String(i.message.clone()),
                                        );
                                        Value::Object(obj)
                                    })
                                    .collect::<Vec<_>>(),
                            );

                            let _ = host.models_mut().update(&validation_model, |v| *v = out);
                            let _ = host.models_mut().update(&state_model_for_submit, |v| {
                                let _ = json_pointer::set(
                                    v,
                                    "/validation/issues",
                                    issues_for_state.clone(),
                                );
                                let _ =
                                    json_pointer::set(v, "/validation/hasErrors", Value::Bool(!ok));
                            });

                            if ok {
                                Ok(())
                            } else {
                                Err(fret_genui_core::executor::GenUiExecError::HandlerFailed {
                                    message: "validation failed".to_string(),
                                })
                            }
                        }),
                    );
                }

                let mut applied: usize = 0;
                let mut skipped: usize = 0;
                let mut errors: usize = 0;
                for inv in invocations.iter() {
                    match executor.execute_invocation(&mut host, inv) {
                        GenUiActionOutcome::Applied => applied = applied.saturating_add(1),
                        GenUiActionOutcome::Skipped => skipped = skipped.saturating_add(1),
                        GenUiActionOutcome::Error(_) => errors = errors.saturating_add(1),
                    }
                }
                state.queue_summary = Some(Arc::<str>::from(format!(
                    "applied queue via executor: applied={applied}, skipped={skipped}, errors={errors}"
                )));
                let _ = app
                    .models_mut()
                    .update(&state.action_queue, |q| q.invocations.clear());
            }
            Msg::SetAutoApply(value) => {
                let _ = app
                    .models_mut()
                    .update(&state.auto_apply_standard_actions, |v| *v = value);
                if value {
                    let _ = app
                        .models_mut()
                        .update(&state.action_queue, |q| q.invocations.clear());
                    state.queue_summary = Some(Arc::<str>::from(
                        "Switched to live mode (auto-apply on). Queue cleared.",
                    ));
                } else {
                    state.queue_summary = Some(Arc::<str>::from(
                        "Switched to queue-only mode (auto-apply off). Pressing buttons will not change state until you apply the queue via executor.",
                    ));
                }
            }
            Msg::AutoApplyToggled => {
                let enabled = app
                    .models()
                    .get_copied(&state.auto_apply_standard_actions)
                    .unwrap_or(true);
                if enabled {
                    let _ = app
                        .models_mut()
                        .update(&state.action_queue, |q| q.invocations.clear());
                    state.queue_summary = Some(Arc::<str>::from(
                        "Switched to live mode (auto-apply on). Queue cleared.",
                    ));
                } else {
                    state.queue_summary = Some(Arc::<str>::from(
                        "Switched to queue-only mode (auto-apply off). Pressing buttons will not change state until you apply the queue via executor.",
                    ));
                }
            }
            Msg::ResetState => {
                let seed = state.spec.state.clone().unwrap_or(Value::Null);
                let _ = app.models_mut().update(&state.genui_state, |v| *v = seed);
                let _ = app.models_mut().update(&state.validation_state, |v| {
                    *v = ValidationStateV1::default()
                });
                let _ = app
                    .models_mut()
                    .update(&state.action_queue, |q| q.invocations.clear());
                state.queue_summary = None;
            }
            Msg::ApplyEditorSpec => {
                let text = app
                    .models()
                    .read(&state.editor_text, Clone::clone)
                    .ok()
                    .unwrap_or_default();
                match serde_json::from_str::<SpecV1>(&text) {
                    Ok(spec) => {
                        let auto_fix = app
                            .models()
                            .get_copied(&state.auto_fix_on_apply)
                            .unwrap_or(true);
                        let (spec, fixups) = maybe_auto_fix_spec(auto_fix, &spec);
                        state.spec = spec;
                        state.editor_error = None;
                        state.auto_fix_summary = summarize_fixups(auto_fix, &fixups);
                        let seed = state.spec.state.clone().unwrap_or(Value::Null);
                        let _ = app.models_mut().update(&state.genui_state, |v| *v = seed);
                        let _ = app.models_mut().update(&state.validation_state, |v| {
                            *v = ValidationStateV1::default()
                        });
                        let _ = app
                            .models_mut()
                            .update(&state.action_queue, |q| q.invocations.clear());
                        state.queue_summary = None;

                        if auto_fix {
                            let pretty = serde_json::to_string_pretty(&state.spec)
                                .unwrap_or_else(|_| "<spec>".to_string());
                            let _ = app.models_mut().update(&state.editor_text, |s| *s = pretty);
                        }
                    }
                    Err(err) => {
                        state.editor_error = Some(Arc::<str>::from(err.to_string()));
                    }
                }
            }
            Msg::ResetEditor => {
                let _ = app
                    .models_mut()
                    .update(&state.editor_text, |s| *s = SPEC_JSON.to_string());
                state.editor_error = None;
                state.auto_fix_summary = None;
                state.queue_summary = None;
            }
            Msg::ApplyStream => {
                let text = app
                    .models()
                    .read(&state.stream_text, Clone::clone)
                    .ok()
                    .unwrap_or_default();
                let patch_only = app
                    .models()
                    .get_copied(&state.stream_patch_only)
                    .unwrap_or(false);

                let mut compiler = MixedSpecStreamCompiler::new(MixedStreamOptions {
                    mode: if patch_only {
                        MixedStreamMode::PatchOnly
                    } else {
                        MixedStreamMode::Mixed
                    },
                    ..Default::default()
                });

                let delta1 = match compiler.push_chunk(&text) {
                    Ok(d) => d,
                    Err(err) => {
                        state.stream_error = Some(Arc::<str>::from(err.to_string()));
                        return;
                    }
                };
                let delta2 = match compiler.flush() {
                    Ok(d) => d,
                    Err(err) => {
                        state.stream_error = Some(Arc::<str>::from(err.to_string()));
                        return;
                    }
                };
                let mut delta = delta1;
                delta.text_lines.extend(delta2.text_lines);
                delta.patches.extend(delta2.patches);

                let value = compiler.into_result();
                match serde_json::from_value::<SpecV1>(value) {
                    Ok(spec) => {
                        let auto_fix = app
                            .models()
                            .get_copied(&state.auto_fix_on_apply)
                            .unwrap_or(true);
                        let (spec, fixups) = maybe_auto_fix_spec(auto_fix, &spec);
                        state.spec = spec;
                        state.editor_error = None;
                        state.stream_error = None;
                        state.auto_fix_summary = summarize_fixups(auto_fix, &fixups);
                        state.stream_summary = Some(Arc::<str>::from(format!(
                            "applied {} patches, saw {} text lines",
                            delta.patches.len(),
                            delta.text_lines.len()
                        )));

                        let seed = state.spec.state.clone().unwrap_or(Value::Null);
                        let _ = app.models_mut().update(&state.genui_state, |v| *v = seed);
                        let _ = app.models_mut().update(&state.validation_state, |v| {
                            *v = ValidationStateV1::default()
                        });
                        let _ = app
                            .models_mut()
                            .update(&state.action_queue, |q| q.invocations.clear());
                        state.queue_summary = None;

                        let pretty = serde_json::to_string_pretty(&state.spec)
                            .unwrap_or_else(|_| "<spec>".to_string());
                        let _ = app.models_mut().update(&state.editor_text, |s| *s = pretty);
                    }
                    Err(err) => {
                        state.stream_error =
                            Some(Arc::<str>::from(format!("Spec parse error: {err}")));
                    }
                }
            }
            Msg::ResetStream => {
                let _ = app.models_mut().update(&state.stream_text, |s| s.clear());
                let _ = app
                    .models_mut()
                    .update(&state.stream_patch_only, |v| *v = false);
                state.stream_summary = None;
                state.stream_error = None;
                state.auto_fix_summary = None;
                state.queue_summary = None;
            }
        }
    }
}

impl View for GenUiView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self {
            st: Self::init_state(app),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, KernelApp>) -> Elements {
        cx.on_action_notify_transient::<act::ClearActions>(TRANSIENT_GENUI_CLEAR_ACTIONS);
        cx.on_action_notify_transient::<act::ApplyQueuedActions>(
            TRANSIENT_GENUI_APPLY_QUEUED_ACTIONS,
        );
        cx.on_action_notify_transient::<act::ResetState>(TRANSIENT_GENUI_RESET_STATE);
        cx.on_action_notify_transient::<act::ApplyEditorSpec>(TRANSIENT_GENUI_APPLY_EDITOR_SPEC);
        cx.on_action_notify_transient::<act::ResetEditor>(TRANSIENT_GENUI_RESET_EDITOR);
        cx.on_action_notify_transient::<act::ApplyStream>(TRANSIENT_GENUI_APPLY_STREAM);
        cx.on_action_notify_transient::<act::ResetStream>(TRANSIENT_GENUI_RESET_STREAM);
        cx.on_action_notify_transient::<act::EnableLive>(TRANSIENT_GENUI_ENABLE_LIVE);
        cx.on_action_notify_transient::<act::AutoApplyToggled>(TRANSIENT_GENUI_AUTO_APPLY_TOGGLED);

        if cx.take_transient_on_action_root(TRANSIENT_GENUI_CLEAR_ACTIONS) {
            Self::handle_msg(cx.app, &mut self.st, Msg::ClearActions);
        }
        if cx.take_transient_on_action_root(TRANSIENT_GENUI_APPLY_QUEUED_ACTIONS) {
            Self::handle_msg(cx.app, &mut self.st, Msg::ApplyQueuedActions);
        }
        if cx.take_transient_on_action_root(TRANSIENT_GENUI_RESET_STATE) {
            Self::handle_msg(cx.app, &mut self.st, Msg::ResetState);
        }
        if cx.take_transient_on_action_root(TRANSIENT_GENUI_APPLY_EDITOR_SPEC) {
            Self::handle_msg(cx.app, &mut self.st, Msg::ApplyEditorSpec);
        }
        if cx.take_transient_on_action_root(TRANSIENT_GENUI_RESET_EDITOR) {
            Self::handle_msg(cx.app, &mut self.st, Msg::ResetEditor);
        }
        if cx.take_transient_on_action_root(TRANSIENT_GENUI_APPLY_STREAM) {
            Self::handle_msg(cx.app, &mut self.st, Msg::ApplyStream);
        }
        if cx.take_transient_on_action_root(TRANSIENT_GENUI_RESET_STREAM) {
            Self::handle_msg(cx.app, &mut self.st, Msg::ResetStream);
        }
        if cx.take_transient_on_action_root(TRANSIENT_GENUI_ENABLE_LIVE) {
            Self::handle_msg(cx.app, &mut self.st, Msg::SetAutoApply(true));
        }
        if cx.take_transient_on_action_root(TRANSIENT_GENUI_AUTO_APPLY_TOGGLED) {
            Self::handle_msg(cx.app, &mut self.st, Msg::AutoApplyToggled);
        }

        view(cx, &mut self.st)
    }
}

fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut GenUiState) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();

    let clear_cmd: fret_runtime::CommandId = act::ClearActions.into();
    let apply_queue_cmd: fret_runtime::CommandId = act::ApplyQueuedActions.into();
    let reset_cmd: fret_runtime::CommandId = act::ResetState.into();
    let apply_editor_cmd: fret_runtime::CommandId = act::ApplyEditorSpec.into();
    let reset_editor_cmd: fret_runtime::CommandId = act::ResetEditor.into();
    let apply_stream_cmd: fret_runtime::CommandId = act::ApplyStream.into();
    let reset_stream_cmd: fret_runtime::CommandId = act::ResetStream.into();
    let enable_live_cmd: fret_runtime::CommandId = act::EnableLive.into();
    let auto_apply_toggled_cmd: fret_runtime::CommandId = act::AutoApplyToggled.into();
    let apply_queue_cmd_toolbar = apply_queue_cmd.clone();
    let apply_queue_cmd_banner = apply_queue_cmd.clone();

    let auto_apply_model = st.auto_apply_standard_actions.clone();
    let auto_apply_enabled = cx
        .watch_model(&st.auto_apply_standard_actions)
        .layout()
        .read(|_host, v| *v)
        .ok()
        .unwrap_or(true);

    let auto_fix_model = st.auto_fix_on_apply.clone();
    let _auto_fix_enabled = cx
        .watch_model(&st.auto_fix_on_apply)
        .layout()
        .read(|_host, v| *v)
        .ok()
        .unwrap_or(true);

    let count_label: Arc<str> = cx
        .watch_model(&st.genui_state)
        .layout()
        .read(|_host, v| {
            let count = json_pointer::get_opt(v, "/count");
            let s = match count {
                Some(Value::Number(n)) => n.to_string(),
                Some(Value::String(s)) => s.clone(),
                Some(Value::Bool(b)) => b.to_string(),
                Some(Value::Null) | None => "null".to_string(),
                Some(other) => other.to_string(),
            };
            Arc::<str>::from(format!("count: {s}"))
        })
        .ok()
        .unwrap_or_else(|| Arc::<str>::from("count: <unavailable>"));

    let queue_snapshot: Vec<Arc<str>> = cx
        .watch_model(&st.action_queue)
        .layout()
        .read(|_host, q| {
            q.invocations
                .iter()
                .enumerate()
                .map(|(i, inv)| {
                    Arc::<str>::from(format!("{:02}  {}  params={}", i, inv.action, inv.params))
                })
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_default();
    let queue_len = queue_snapshot.len();
    let queue_lines: Vec<AnyElement> = queue_snapshot
        .into_iter()
        .map(|line| ui::text(line).text_sm().into_element(cx))
        .collect();

    let state_snapshot: Vec<Arc<str>> = cx
        .watch_model(&st.genui_state)
        .layout()
        .read(|_host, v| {
            let pretty = serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string());
            pretty
                .lines()
                .map(|line| Arc::<str>::from(line.to_string()))
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_else(|| vec![Arc::<str>::from("null")]);
    let state_lines: Vec<AnyElement> = state_snapshot
        .into_iter()
        .map(|line| ui::text(line).text_sm().into_element(cx))
        .collect();

    let (validation_issue_count, validation_snapshot): (usize, Vec<Arc<str>>) = cx
        .watch_model(&st.validation_state)
        .layout()
        .read(|_host, v| {
            let pretty =
                serde_json::to_string_pretty(v).unwrap_or_else(|_| "<validation>".to_string());
            (
                v.count(),
                pretty
                    .lines()
                    .map(|line| Arc::<str>::from(line.to_string()))
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap_or_else(|_| (0, vec![Arc::<str>::from("<validation unavailable>")]));
    let validation_lines: Vec<AnyElement> = validation_snapshot
        .into_iter()
        .map(|line| ui::text(line).text_sm().into_element(cx))
        .collect();

    let toolbar = ui::h_flex(move |cx| {
        let mode_label: Arc<str> = Arc::from(if auto_apply_enabled {
            "Mode: live (auto-apply on)"
        } else {
            "Mode: queue-only (auto-apply off)"
        });
        let mode_badge = shadcn::Badge::new(mode_label)
            .variant(if auto_apply_enabled {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Destructive
            })
            .into_element(cx);

        let mut items: Vec<AnyElement> = Vec::new();
        items.push(mode_badge);

        items.push(
            ui::text(Arc::<str>::from("auto-apply"))
                .text_sm()
                .into_element(cx),
        );
        items.push(
            shadcn::Switch::new(auto_apply_model.clone())
                .a11y_label("Auto-apply standard actions")
                .on_click(auto_apply_toggled_cmd)
                .into_element(cx),
        );

        items.push(
            ui::text(Arc::<str>::from("auto-fix on apply"))
                .text_sm()
                .into_element(cx),
        );
        items.push(
            shadcn::Switch::new(auto_fix_model.clone())
                .a11y_label("Auto-fix spec on apply")
                .into_element(cx),
        );

        items.push(ui::text(count_label.clone()).text_sm().into_element(cx));

        items.push(
            shadcn::Button::new("Clear queue")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(clear_cmd)
                .into_element(cx),
        );

        if !auto_apply_enabled {
            items.push(
                shadcn::Button::new("Apply queue (executor)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(apply_queue_cmd_toolbar)
                    .into_element(cx),
            );
        }

        items.push(
            shadcn::Button::new("Reset state")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(reset_cmd)
                .into_element(cx),
        );

        items
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let runtime = GenUiRuntime {
        state: st.genui_state.clone(),
        action_queue: Some(st.action_queue.clone()),
        auto_apply_standard_actions: auto_apply_enabled,
        limits: Default::default(),
        catalog: Some(st.catalog.clone()),
        catalog_validation: ValidationMode::Strict,
    };

    let mut resolver = ShadcnResolver::new();
    let rendered = render_spec(cx, &st.spec, &runtime, &mut resolver);

    let (spec_root, spec_issue_lines): (Option<AnyElement>, Vec<Arc<str>>) = match rendered {
        Ok(out) => (
            out.roots.into_iter().next(),
            out.issues
                .into_iter()
                .map(|i| Arc::<str>::from(format!("{:?}: {}", i.code, i.message)))
                .collect::<Vec<_>>(),
        ),
        Err(err) => {
            let err_el = shadcn::Alert::new([
                shadcn::AlertTitle::new("Render error").into_element(cx),
                shadcn::AlertDescription::new(Arc::<str>::from(err.to_string())).into_element(cx),
            ])
            .into_element(cx);
            (Some(err_el), Vec::new())
        }
    };
    let spec_issue_count = spec_issue_lines.len();

    let left = {
        let mut body: Vec<AnyElement> = Vec::new();
        body.push(toolbar);
        if !auto_apply_enabled {
            body.push(
                shadcn::Alert::new([
                    shadcn::AlertTitle::new("Queue-only mode").into_element(cx),
                    shadcn::AlertDescription::new(
                        "Auto-apply is off: pressing buttons will NOT change the UI. Actions are only appended to the queue. Apply the queue via executor, or switch back to live mode.",
                    )
                    .into_element(cx),
                    ui::h_flex( move |cx| {
                        vec![
                            shadcn::Button::new("Switch to live mode")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .on_click(enable_live_cmd)
                                .into_element(cx),
                            shadcn::Button::new("Apply queue (executor)")
                                .variant(shadcn::ButtonVariant::Outline)
                                .on_click(apply_queue_cmd_banner)
                                .into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx),
                ])
                .into_element(cx),
            );
        }
        if spec_issue_count != 0 {
            body.push(
                shadcn::Alert::new([
                    shadcn::AlertTitle::new("Spec issues").into_element(cx),
                    shadcn::AlertDescription::new("Fix the spec before rendering.")
                        .into_element(cx),
                ])
                .into_element(cx),
            );
            body.extend(
                spec_issue_lines
                    .iter()
                    .cloned()
                    .map(|s| ui::text(s).text_sm().into_element(cx)),
            );
        }
        if let Some(root) = spec_root {
            body.push(root);
        }
        let body = ui::v_flex(move |_cx| body)
            .gap(Space::N3)
            .items_start()
            .w_full()
            .p(Space::N6)
            .into_element(cx);
        shadcn::Card::new([body])
            .ui()
            .w_full()
            .h_full()
            .min_w(Px(420.0))
            .into_element(cx)
    };

    let spec_pretty = serde_json::to_string_pretty(&st.spec).unwrap_or_else(|_| {
        serde_json::to_string(&st.spec).unwrap_or_else(|_| "<spec>".to_string())
    });
    let spec_lines: Vec<AnyElement> = spec_pretty
        .lines()
        .map(|line| {
            ui::text(Arc::<str>::from(line.to_string()))
                .text_sm()
                .into_element(cx)
        })
        .collect();

    let schema_value = st.catalog.spec_json_schema();
    let schema_pretty =
        serde_json::to_string_pretty(&schema_value).unwrap_or_else(|_| schema_value.to_string());
    let schema_lines: Vec<AnyElement> = schema_pretty
        .lines()
        .map(|line| {
            ui::text(Arc::<str>::from(line.to_string()))
                .text_sm()
                .into_element(cx)
        })
        .collect();

    let prompt = st.catalog.system_prompt();
    let prompt_lines: Vec<AnyElement> = prompt
        .lines()
        .map(|line| {
            ui::text(Arc::<str>::from(line.to_string()))
                .text_sm()
                .into_element(cx)
        })
        .collect();

    let editor_model = st.editor_text.clone();
    let editor_error = st.editor_error.clone();
    let auto_fix_summary = st.auto_fix_summary.clone();
    let queue_summary = st.queue_summary.clone();
    let stream_model = st.stream_text.clone();
    let stream_patch_only_model = st.stream_patch_only.clone();
    let stream_patch_only = cx
        .watch_model(&st.stream_patch_only)
        .layout()
        .read(|_host, v| *v)
        .ok()
        .unwrap_or(false);
    let stream_summary = st.stream_summary.clone();
    let stream_error = st.stream_error.clone();

    let right = {
        let state_scroll = shadcn::ScrollArea::new([ui::v_flex(|_cx| state_lines)
            .gap(Space::N0)
            .w_full()
            .items_start()
            .into_element(cx)])
        .ui()
        .w_full()
        .h_full()
        .into_element(cx);

        let queue_scroll = shadcn::ScrollArea::new([ui::v_flex(|_cx| queue_lines)
            .gap(Space::N1)
            .w_full()
            .items_start()
            .into_element(cx)])
        .ui()
        .w_full()
        .h_full()
        .into_element(cx);
        let queue_panel = ui::v_flex(move |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            if let Some(summary) = queue_summary.clone() {
                out.push(
                    shadcn::Alert::new([
                        shadcn::AlertTitle::new("Queue summary").into_element(cx),
                        shadcn::AlertDescription::new(summary).into_element(cx),
                    ])
                    .into_element(cx),
                );
            }
            out.push(queue_scroll);
            out
        })
        .gap(Space::N2)
        .w_full()
        .h_full()
        .into_element(cx);

        let issues_body = if spec_issue_count == 0 {
            vec![
                ui::text(Arc::<str>::from("No spec issues."))
                    .text_sm()
                    .into_element(cx),
            ]
        } else {
            spec_issue_lines
                .iter()
                .cloned()
                .map(|s| ui::text(s).text_sm().into_element(cx))
                .collect()
        };
        let issues_scroll = shadcn::ScrollArea::new([ui::v_flex(|_cx| issues_body)
            .gap(Space::N1)
            .w_full()
            .items_start()
            .into_element(cx)])
        .ui()
        .w_full()
        .h_full()
        .into_element(cx);

        let validation_scroll = shadcn::ScrollArea::new([ui::v_flex(|_cx| validation_lines)
            .gap(Space::N0)
            .w_full()
            .items_start()
            .into_element(cx)])
        .ui()
        .w_full()
        .h_full()
        .into_element(cx);

        let spec_scroll = shadcn::ScrollArea::new([ui::v_flex(|_cx| spec_lines)
            .gap(Space::N0)
            .w_full()
            .items_start()
            .into_element(cx)])
        .ui()
        .w_full()
        .h_full()
        .into_element(cx);

        let schema_scroll = shadcn::ScrollArea::new([ui::v_flex(|_cx| schema_lines)
            .gap(Space::N0)
            .w_full()
            .items_start()
            .into_element(cx)])
        .ui()
        .w_full()
        .h_full()
        .into_element(cx);

        let prompt_scroll = shadcn::ScrollArea::new([ui::v_flex(|_cx| prompt_lines)
            .gap(Space::N0)
            .w_full()
            .items_start()
            .into_element(cx)])
        .ui()
        .w_full()
        .h_full()
        .into_element(cx);

        let mut editor_children: Vec<AnyElement> = Vec::new();
        editor_children.push(
            ui::h_flex(move |cx| {
                vec![
                    shadcn::Button::new("Apply spec")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .on_click(apply_editor_cmd)
                        .into_element(cx),
                    shadcn::Button::new("Reset editor")
                        .variant(shadcn::ButtonVariant::Outline)
                        .on_click(reset_editor_cmd)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
        );
        editor_children.push(ui::text("").text_sm().into_element(cx));
        if let Some(summary) = auto_fix_summary.clone() {
            editor_children.push(
                shadcn::Alert::new([
                    shadcn::AlertTitle::new("Auto-fix summary").into_element(cx),
                    shadcn::AlertDescription::new(summary).into_element(cx),
                ])
                .into_element(cx),
            );
        }
        editor_children.push(
            shadcn::Textarea::new(editor_model.clone())
                .a11y_label("Spec editor")
                .min_height(Px(280.0))
                .into_element(cx),
        );
        if let Some(err) = editor_error {
            editor_children.push(
                shadcn::Alert::new([
                    shadcn::AlertTitle::new("Editor error").into_element(cx),
                    shadcn::AlertDescription::new(err).into_element(cx),
                ])
                .into_element(cx),
            );
        }
        let editor_panel = ui::v_flex(move |_cx| editor_children)
            .gap(Space::N2)
            .items_start()
            .w_full()
            .into_element(cx);
        let editor_scroll = shadcn::ScrollArea::new([editor_panel])
            .ui()
            .w_full()
            .h_full()
            .into_element(cx);

        let mut stream_children: Vec<AnyElement> = Vec::new();
        stream_children.push(
            ui::h_flex(move |cx| {
                vec![
                    shadcn::Button::new("Apply stream → spec")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .on_click(apply_stream_cmd)
                        .into_element(cx),
                    shadcn::Button::new("Reset stream")
                        .variant(shadcn::ButtonVariant::Outline)
                        .on_click(reset_stream_cmd)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
        );
        stream_children.push(
            ui::h_flex(move |cx| {
                vec![
                    ui::text(Arc::<str>::from(format!(
                        "patch-only: {}",
                        if stream_patch_only { "on" } else { "off" }
                    )))
                    .text_sm()
                    .into_element(cx),
                    shadcn::Switch::new(stream_patch_only_model.clone())
                        .a11y_label("Patch-only stream mode")
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
        );
        stream_children.push(
            ui::text(Arc::<str>::from(
                    "Paste json-render-style mixed streams here (text + JSONL RFC6902 patches). Supports ```spec fences.",
                ),
            )
            .text_sm()
            .into_element(cx),
        );
        stream_children.push(
            shadcn::Textarea::new(stream_model.clone())
                .a11y_label("Mixed stream input")
                .min_height(Px(220.0))
                .into_element(cx),
        );
        if let Some(summary) = stream_summary {
            stream_children.push(ui::text(summary).text_sm().into_element(cx));
        }
        if let Some(summary) = auto_fix_summary {
            stream_children.push(
                shadcn::Alert::new([
                    shadcn::AlertTitle::new("Auto-fix summary").into_element(cx),
                    shadcn::AlertDescription::new(summary).into_element(cx),
                ])
                .into_element(cx),
            );
        }
        if let Some(err) = stream_error {
            stream_children.push(
                shadcn::Alert::new([
                    shadcn::AlertTitle::new("Stream error").into_element(cx),
                    shadcn::AlertDescription::new(err).into_element(cx),
                ])
                .into_element(cx),
            );
        }
        let stream_panel = ui::v_flex(move |_cx| stream_children)
            .gap(Space::N2)
            .items_start()
            .w_full()
            .into_element(cx);
        let stream_scroll = shadcn::ScrollArea::new([stream_panel])
            .ui()
            .w_full()
            .h_full()
            .into_element(cx);

        let tabs = shadcn::Tabs::uncontrolled(Some("state"))
            .content_fill_remaining(true)
            .items([
                shadcn::TabsItem::new("state", "State", [state_scroll]),
                shadcn::TabsItem::new("queue", format!("Queue ({queue_len})"), [queue_panel]),
                shadcn::TabsItem::new(
                    "issues",
                    format!("Issues ({spec_issue_count})"),
                    [issues_scroll],
                ),
                shadcn::TabsItem::new(
                    "validation",
                    format!("Validation ({validation_issue_count})"),
                    [validation_scroll],
                ),
                shadcn::TabsItem::new("spec", "Spec", [spec_scroll]),
                shadcn::TabsItem::new("schema", "Schema", [schema_scroll]),
                shadcn::TabsItem::new("prompt", "Prompt", [prompt_scroll]),
                shadcn::TabsItem::new("editor", "Editor", [editor_scroll]),
                shadcn::TabsItem::new("stream", "Stream", [stream_scroll]),
            ])
            .into_element(cx);

        let body = ui::v_flex(move |_cx| [tabs])
            .gap(Space::N0)
            .w_full()
            .h_full()
            .p(Space::N6)
            .into_element(cx);
        shadcn::Card::new([body])
            .ui()
            .w_full()
            .h_full()
            .min_w(Px(360.0))
            .into_element(cx)
    };

    let page = ui::container(move |cx| {
        [ui::h_flex(move |_cx| [left, right])
            .gap(Space::N3)
            .w_full()
            .h_full()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N4)
    .w_full()
    .h_full()
    .into_element(cx);

    ui::children![cx; page].into()
}

fn maybe_auto_fix_spec(enabled: bool, spec: &SpecV1) -> (SpecV1, SpecFixups) {
    if enabled {
        auto_fix_spec(spec)
    } else {
        (spec.clone(), SpecFixups::default())
    }
}

fn summarize_fixups(enabled: bool, fixups: &SpecFixups) -> Option<Arc<str>> {
    if !enabled {
        return None;
    }

    if fixups.fixes.is_empty() {
        return Some(Arc::<str>::from("No auto-fix changes were applied."));
    }

    const MAX_LINES: usize = 12;
    let mut lines = Vec::new();
    lines.push(format!(
        "Applied {} auto-fix change(s):",
        fixups.fixes.len()
    ));
    for fix in fixups.fixes.iter().take(MAX_LINES) {
        lines.push(format!("- {fix}"));
    }
    if fixups.fixes.len() > MAX_LINES {
        lines.push(format!(
            "... and {} more.",
            fixups.fixes.len().saturating_sub(MAX_LINES)
        ));
    }
    Some(Arc::<str>::from(lines.join("\n")))
}
