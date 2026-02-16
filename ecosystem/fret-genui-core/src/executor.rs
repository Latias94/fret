//! Action execution helpers (app-owned policy hooks).
//!
//! GenUI core emits action invocations (event → `{ action, params }`) as data. Apps can:
//! - keep the invocations for logging/diagnostics only,
//! - auto-apply standard actions (see `GenUiRuntime.auto_apply_standard_actions`),
//! - or execute invocations through a registry/executor (this module).
//!
//! This module is intentionally conservative:
//! - it provides a minimal handler registry,
//! - supports basic `confirm` gating and `onSuccess` / `onError` chaining,
//! - and keeps all non-trivial policies (dialogs, async, permissions) app-owned.

use std::collections::BTreeMap;
use std::sync::Arc;

use fret_runtime::Effect;
use fret_runtime::Model;
use fret_ui::action::UiActionHost;
use serde_json::Value;

use crate::props::PropResolutionContext;
use crate::render::GenUiActionInvocation;
use crate::spec::{ActionBindingV1, OnBindingV1};
use crate::visibility::RepeatScope;

#[derive(Debug, Clone, Copy)]
pub struct GenUiExecutionLimits {
    pub max_chain_depth: usize,
    pub max_total_actions: usize,
}

impl Default for GenUiExecutionLimits {
    fn default() -> Self {
        Self {
            max_chain_depth: 16,
            max_total_actions: 256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenUiActionOutcome {
    Applied,
    Skipped,
    Error(GenUiExecError),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GenUiExecError {
    #[error("unknown action: {action}")]
    UnknownAction { action: Arc<str> },
    #[error("action handler failed: {message}")]
    HandlerFailed { message: String },
    #[error("invalid chained action bindings: {message}")]
    InvalidChain { message: String },
    #[error("execution limits exceeded: {kind}")]
    LimitExceeded { kind: &'static str },
}

pub type GenUiActionHandlerV1 = Arc<
    dyn Fn(
            &mut dyn UiActionHost,
            &Model<Value>,
            &GenUiActionInvocation,
        ) -> Result<(), GenUiExecError>
        + 'static,
>;

pub type GenUiConfirmPolicyV1 =
    Box<dyn Fn(&mut dyn UiActionHost, &GenUiActionInvocation, &Value) -> bool + 'static>;

pub struct GenUiActionExecutorV1 {
    state: Model<Value>,
    handlers: BTreeMap<Arc<str>, GenUiActionHandlerV1>,
    confirm: Option<GenUiConfirmPolicyV1>,
    limits: GenUiExecutionLimits,
}

impl GenUiActionExecutorV1 {
    pub fn new(state: Model<Value>) -> Self {
        Self {
            state,
            handlers: BTreeMap::new(),
            confirm: None,
            limits: GenUiExecutionLimits::default(),
        }
    }

    pub fn limits(mut self, limits: GenUiExecutionLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn confirm_policy(mut self, policy: GenUiConfirmPolicyV1) -> Self {
        self.confirm = Some(policy);
        self
    }

    pub fn register_handler(&mut self, action: impl Into<Arc<str>>, handler: GenUiActionHandlerV1) {
        self.handlers.insert(action.into(), handler);
    }

    pub fn with_standard_actions(mut self) -> Self {
        fn handler(
            host: &mut dyn UiActionHost,
            state: &Model<Value>,
            inv: &GenUiActionInvocation,
        ) -> Result<(), GenUiExecError> {
            let ok = host
                .models_mut()
                .update(state, |st| {
                    crate::actions::apply_standard_action(st, inv.action.as_ref(), &inv.params)
                })
                .ok()
                .unwrap_or(false);
            if ok {
                Ok(())
            } else {
                Err(GenUiExecError::HandlerFailed {
                    message: "standard action rejected (invalid params or statePath)".to_string(),
                })
            }
        }

        let h: GenUiActionHandlerV1 = Arc::new(handler);
        self.handlers.insert(Arc::from("setState"), h.clone());
        self.handlers.insert(Arc::from("incrementState"), h.clone());
        self.handlers.insert(Arc::from("pushState"), h.clone());
        self.handlers.insert(Arc::from("removeState"), h);
        self
    }

    pub fn with_portable_effect_actions(mut self) -> Self {
        fn open_url(
            host: &mut dyn UiActionHost,
            _state: &Model<Value>,
            inv: &GenUiActionInvocation,
        ) -> Result<(), GenUiExecError> {
            let Some(obj) = inv.params.as_object() else {
                return Err(GenUiExecError::HandlerFailed {
                    message: "openUrl params must be an object".to_string(),
                });
            };
            let Some(url) = obj.get("url").and_then(|v| v.as_str()) else {
                return Err(GenUiExecError::HandlerFailed {
                    message: "openUrl requires params.url".to_string(),
                });
            };
            let target = obj
                .get("target")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let rel = obj
                .get("rel")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            host.push_effect(Effect::OpenUrl {
                url: url.to_string(),
                target,
                rel,
            });
            Ok(())
        }

        fn clipboard_set_text(
            host: &mut dyn UiActionHost,
            _state: &Model<Value>,
            inv: &GenUiActionInvocation,
        ) -> Result<(), GenUiExecError> {
            let Some(obj) = inv.params.as_object() else {
                return Err(GenUiExecError::HandlerFailed {
                    message: "clipboardSetText params must be an object".to_string(),
                });
            };
            let Some(text) = obj.get("text").and_then(|v| v.as_str()) else {
                return Err(GenUiExecError::HandlerFailed {
                    message: "clipboardSetText requires params.text".to_string(),
                });
            };
            host.push_effect(Effect::ClipboardSetText {
                text: text.to_string(),
            });
            Ok(())
        }

        self.handlers
            .insert(Arc::from("openUrl"), Arc::new(open_url));
        self.handlers
            .insert(Arc::from("clipboardSetText"), Arc::new(clipboard_set_text));
        self
    }

    pub fn execute_invocation(
        &mut self,
        host: &mut dyn UiActionHost,
        inv: &GenUiActionInvocation,
    ) -> GenUiActionOutcome {
        let mut budget = self.limits.max_total_actions;
        self.execute_invocation_inner(host, inv, 0, &mut budget)
    }

    fn execute_invocation_inner(
        &mut self,
        host: &mut dyn UiActionHost,
        inv: &GenUiActionInvocation,
        depth: usize,
        budget: &mut usize,
    ) -> GenUiActionOutcome {
        if depth > self.limits.max_chain_depth {
            return GenUiActionOutcome::Error(GenUiExecError::LimitExceeded {
                kind: "max_chain_depth",
            });
        }
        if *budget == 0 {
            return GenUiActionOutcome::Error(GenUiExecError::LimitExceeded {
                kind: "max_total_actions",
            });
        }
        *budget -= 1;

        if let Some(confirm) = inv.confirm.as_ref() {
            if confirm == &Value::Bool(false) {
                return GenUiActionOutcome::Skipped;
            }
            if let Some(policy) = self.confirm.as_ref() {
                if !policy(host, inv, confirm) {
                    return GenUiActionOutcome::Skipped;
                }
            }
        }

        let Some(handler) = self.handlers.get(&inv.action).cloned() else {
            let err = GenUiExecError::UnknownAction {
                action: inv.action.clone(),
            };
            let _ = self.execute_chain_value(host, inv, inv.on_error.as_ref(), depth + 1, budget);
            return GenUiActionOutcome::Error(err);
        };

        let result = handler(host, &self.state, inv);
        match result {
            Ok(()) => {
                let chain_res =
                    self.execute_chain_value(host, inv, inv.on_success.as_ref(), depth + 1, budget);
                if let GenUiActionOutcome::Error(err) = chain_res {
                    GenUiActionOutcome::Error(err)
                } else {
                    GenUiActionOutcome::Applied
                }
            }
            Err(err) => {
                let chain_res =
                    self.execute_chain_value(host, inv, inv.on_error.as_ref(), depth + 1, budget);
                match chain_res {
                    GenUiActionOutcome::Applied | GenUiActionOutcome::Skipped => {
                        GenUiActionOutcome::Error(err)
                    }
                    GenUiActionOutcome::Error(chain_err) => GenUiActionOutcome::Error(chain_err),
                }
            }
        }
    }

    fn execute_chain_value(
        &mut self,
        host: &mut dyn UiActionHost,
        inv: &GenUiActionInvocation,
        chain: Option<&Value>,
        depth: usize,
        budget: &mut usize,
    ) -> GenUiActionOutcome {
        let Some(chain) = chain else {
            return GenUiActionOutcome::Skipped;
        };

        let Some(bindings) = parse_chain_bindings(chain) else {
            return GenUiActionOutcome::Error(GenUiExecError::InvalidChain {
                message: "expected ActionBinding or [ActionBinding]".to_string(),
            });
        };

        for b in bindings {
            let nested = match self.binding_to_invocation(host, inv, &b) {
                Ok(v) => v,
                Err(err) => return GenUiActionOutcome::Error(err),
            };
            let out = self.execute_invocation_inner(host, &nested, depth, budget);
            if let GenUiActionOutcome::Error(err) = out {
                return GenUiActionOutcome::Error(err);
            }
        }
        GenUiActionOutcome::Applied
    }

    fn binding_to_invocation(
        &self,
        host: &mut dyn UiActionHost,
        parent: &GenUiActionInvocation,
        binding: &ActionBindingV1,
    ) -> Result<GenUiActionInvocation, GenUiExecError> {
        let state_snapshot: Value = host
            .models_mut()
            .read(&self.state, Clone::clone)
            .unwrap_or(Value::Null);

        let repeat_scope = invocation_repeat_scope(&state_snapshot, parent);
        let prop_ctx = PropResolutionContext {
            state: &state_snapshot,
            repeat: repeat_scope,
        };

        let params = binding
            .params
            .as_ref()
            .map(|map| {
                Value::Object(
                    map.iter()
                        .map(|(k, v)| (k.clone(), crate::props::resolve_action_param(v, &prop_ctx)))
                        .collect(),
                )
            })
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        Ok(GenUiActionInvocation {
            window: parent.window,
            source: parent.source,
            element_key: parent.element_key.clone(),
            event: parent.event.clone(),
            action: Arc::from(binding.action.as_str()),
            params,
            confirm: binding.confirm.clone(),
            on_success: binding.on_success.clone(),
            on_error: binding.on_error.clone(),
            repeat_base_path: parent.repeat_base_path.clone(),
            repeat_index: parent.repeat_index,
        })
    }
}

fn parse_chain_bindings(v: &Value) -> Option<Vec<ActionBindingV1>> {
    serde_json::from_value::<OnBindingV1>(v.clone())
        .ok()
        .map(|b| match b {
            OnBindingV1::One(x) => vec![x],
            OnBindingV1::Many(xs) => xs,
        })
}

fn invocation_repeat_scope<'a>(
    state: &'a Value,
    inv: &'a GenUiActionInvocation,
) -> RepeatScope<'a> {
    let repeat_item = inv
        .repeat_base_path
        .as_deref()
        .and_then(|p| crate::json_pointer::get_opt(state, p));
    RepeatScope {
        item: repeat_item,
        index: inv.repeat_index,
        base_path: inv.repeat_base_path.as_deref(),
    }
}
