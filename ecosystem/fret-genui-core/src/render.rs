//! Spec renderer: `SpecV1` → `AnyElement` tree.
//!
//! This module is intentionally small and policy-light. It provides:
//! - deterministic rendering and stable identity via `ElementContext::keyed`,
//! - repeat and visibility semantics,
//! - event → action invocation emission (app-owned queue model).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnActivate, UiActionHostExt};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};
use serde_json::Value;

use crate::catalog::CatalogV1;
use crate::props::{PropResolutionContext, ResolvedProps};
use crate::spec::{ElementKey, ElementV1, OnBindingV1, SpecV1};
use crate::validate::{SpecIssue, ValidateSpecOptions, ValidationMode, validate_spec};
use crate::visibility::{RepeatScope, VisibilityContext};

#[derive(Debug, Clone, Default)]
pub struct GenUiRenderScope {
    pub state: Option<Model<Value>>,
    pub action_queue: Option<Model<GenUiActionQueue>>,
    pub auto_apply_standard_actions: bool,
}

#[derive(Debug, Clone)]
pub struct RenderLimits {
    pub max_elements: usize,
    pub max_depth: usize,
    pub max_repeat_items: usize,
}

impl Default for RenderLimits {
    fn default() -> Self {
        Self {
            max_elements: 2000,
            max_depth: 64,
            max_repeat_items: 200,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenUiRuntime {
    pub state: Model<Value>,
    pub action_queue: Option<Model<GenUiActionQueue>>,
    /// When enabled, apply recognized "standard actions" (see `crate::actions`) directly to
    /// `state` at dispatch time (in addition to emitting into `action_queue` when present).
    ///
    /// This is intended for demos and simple apps that want immediate, deterministic updates
    /// without building a separate executor loop. Default: `false`.
    pub auto_apply_standard_actions: bool,
    pub limits: RenderLimits,
    pub catalog: Option<Arc<CatalogV1>>,
    pub catalog_validation: ValidationMode,
}

#[derive(Debug, Clone, Default)]
pub struct GenUiActionQueue {
    pub invocations: Vec<GenUiActionInvocation>,
}

#[derive(Debug, Clone)]
pub struct GenUiActionInvocation {
    pub window: AppWindowId,
    pub source: GlobalElementId,
    pub element_key: Arc<str>,
    pub event: Arc<str>,
    pub action: Arc<str>,
    pub params: Value,
    /// App-owned forward-compat fields (mirrors `spec::ActionBindingV1`).
    pub confirm: Option<Value>,
    pub on_success: Option<Value>,
    pub on_error: Option<Value>,
    /// Repeat scope metadata (when dispatched from inside `repeat`).
    pub repeat_base_path: Option<Arc<str>>,
    pub repeat_index: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("spec is invalid")]
    InvalidSpec,
    #[error("component render failed for {component} at {key:?}: {message}")]
    Component {
        key: ElementKey,
        component: String,
        message: String,
    },
    #[error("render limits exceeded: {kind}")]
    LimitExceeded { kind: &'static str },
    #[error("cycle detected at element key: {key:?}")]
    Cycle { key: ElementKey },
    #[error("missing element key: {key:?}")]
    MissingElement { key: ElementKey },
    #[error("repeat statePath is not an array: {path}")]
    RepeatNotArray { path: String },
}

#[derive(Debug)]
pub struct RenderOutput {
    pub roots: Vec<AnyElement>,
    pub issues: Vec<SpecIssue>,
}

pub trait ComponentResolver<H: UiHost> {
    type Error: std::error::Error + Send + Sync + 'static;

    fn render_element(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        element: &ElementV1,
        props: &ResolvedProps,
        children: Vec<RenderedChildV1>,
        on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> Result<AnyElement, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct RenderedChildV1 {
    pub key: ElementKey,
    pub component: String,
    pub props: ResolvedProps,
    pub rendered: AnyElement,
}

#[derive(Clone)]
struct EventDispatcher {
    element_key: Arc<str>,
    on: Option<BTreeMap<String, OnBindingV1>>,
    state: Model<Value>,
    queue: Option<Model<GenUiActionQueue>>,
    auto_apply_standard_actions: bool,
    repeat_base_path: Option<Arc<str>>,
    repeat_index: Option<usize>,
}

impl EventDispatcher {
    fn on_event(&self, event: &str) -> Option<OnActivate> {
        let bindings = self.on.as_ref()?.get(event)?.clone();
        let element_key = self.element_key.clone();
        let event: Arc<str> = Arc::from(event);
        let state = self.state.clone();
        let queue = self.queue.clone();
        let auto_apply_standard_actions = self.auto_apply_standard_actions;
        let repeat_base_path = self.repeat_base_path.clone();
        let repeat_index = self.repeat_index;

        Some(Arc::new(move |host, cx: ActionCx, _reason| {
            let state_snapshot: Value = host
                .models_mut()
                .read(&state, Clone::clone)
                .unwrap_or(Value::Null);

            let repeat_item_value: Option<Value> = repeat_base_path
                .as_ref()
                .and_then(|p| crate::json_pointer::get_opt(&state_snapshot, p).cloned());

            let repeat_scope = RepeatScope {
                item: repeat_item_value.as_ref(),
                index: repeat_index,
                base_path: repeat_base_path.as_deref(),
            };
            let prop_ctx = PropResolutionContext {
                state: &state_snapshot,
                repeat: repeat_scope,
            };

            let mut invocations: Vec<GenUiActionInvocation> = Vec::new();
            for b in bindings.iter() {
                let params = b
                    .params
                    .as_ref()
                    .map(|map| {
                        Value::Object(
                            map.iter()
                                .map(|(k, v)| {
                                    (k.clone(), crate::props::resolve_action_param(v, &prop_ctx))
                                })
                                .collect(),
                        )
                    })
                    .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

                invocations.push(GenUiActionInvocation {
                    window: cx.window,
                    source: cx.target,
                    element_key: element_key.clone(),
                    event: event.clone(),
                    action: Arc::from(b.action.as_str()),
                    params,
                    confirm: b.confirm.clone(),
                    on_success: b.on_success.clone(),
                    on_error: b.on_error.clone(),
                    repeat_base_path: repeat_base_path.clone(),
                    repeat_index,
                });
            }

            if let Some(queue) = queue.as_ref() {
                let _ = host.update_model(queue, |q| q.invocations.extend(invocations.clone()));
            }

            if auto_apply_standard_actions {
                let _ = host.update_model(&state, |st| {
                    for inv in invocations.iter() {
                        let _ = crate::actions::apply_standard_action(
                            st,
                            inv.action.as_ref(),
                            &inv.params,
                        );
                    }
                });
            }

            host.request_redraw(cx.window);
        }))
    }
}

pub fn render_spec<H: UiHost, R: ComponentResolver<H>>(
    cx: &mut ElementContext<'_, H>,
    spec: &SpecV1,
    runtime: &GenUiRuntime,
    resolver: &mut R,
) -> Result<RenderOutput, RenderError> {
    let validate = validate_spec(
        spec,
        ValidateSpecOptions {
            check_orphans: false,
            supported_schema_versions: {
                let mut set = BTreeSet::new();
                set.insert(1);
                set
            },
            catalog: runtime.catalog.clone(),
            catalog_validation: runtime.catalog_validation,
        },
    );
    if !validate.valid {
        return Ok(RenderOutput {
            roots: Vec::new(),
            issues: validate.issues,
        });
    }

    let state_snapshot = cx
        .get_model_cloned(&runtime.state, Invalidation::Layout)
        .unwrap_or(Value::Null);

    let mut rendered_count: usize = 0;
    let mut stack: Vec<ElementKey> = Vec::new();
    let mut roots = Vec::new();

    let root_el = render_element_node(
        cx,
        spec,
        runtime,
        resolver,
        &state_snapshot,
        &spec.root,
        RepeatScope::default(),
        None,
        None,
        0,
        &mut rendered_count,
        &mut stack,
    )?;
    roots.push(root_el.rendered);

    if rendered_count > runtime.limits.max_elements {
        return Err(RenderError::LimitExceeded {
            kind: "max_elements",
        });
    }

    Ok(RenderOutput {
        roots,
        issues: validate.issues,
    })
}

#[allow(clippy::too_many_arguments)]
fn render_element_node<H: UiHost, R: ComponentResolver<H>>(
    cx: &mut ElementContext<'_, H>,
    spec: &SpecV1,
    runtime: &GenUiRuntime,
    resolver: &mut R,
    state_snapshot: &Value,
    key: &ElementKey,
    repeat_scope: RepeatScope<'_>,
    repeat_base_path: Option<Arc<str>>,
    repeat_index: Option<usize>,
    depth: usize,
    rendered_count: &mut usize,
    stack: &mut Vec<ElementKey>,
) -> Result<RenderedChildV1, RenderError> {
    if depth > runtime.limits.max_depth {
        return Err(RenderError::LimitExceeded { kind: "max_depth" });
    }

    if stack.iter().any(|k| k == key) {
        return Err(RenderError::Cycle { key: key.clone() });
    }
    stack.push(key.clone());

    let element = spec
        .elements
        .get(key)
        .ok_or_else(|| RenderError::MissingElement { key: key.clone() })?;

    let out = cx.keyed(&key.0, |cx| {
        let visible = element.visible.as_ref().map_or(true, |cond| {
            let vctx = VisibilityContext {
                state: state_snapshot,
                repeat: repeat_scope,
            };
            crate::visibility::evaluate(cond, &vctx)
        });

        let prop_ctx = PropResolutionContext {
            state: state_snapshot,
            repeat: repeat_scope,
        };
        let props = crate::props::resolve_props(&element.props, &prop_ctx);

        let dispatcher = EventDispatcher {
            element_key: Arc::from(key.0.as_str()),
            on: element.on.clone(),
            state: runtime.state.clone(),
            queue: runtime.action_queue.clone(),
            auto_apply_standard_actions: runtime.auto_apply_standard_actions,
            repeat_base_path: repeat_base_path.clone(),
            repeat_index,
        };

        cx.with_state(GenUiRenderScope::default, |st| {
            st.state = Some(runtime.state.clone());
            st.action_queue = runtime.action_queue.clone();
            st.auto_apply_standard_actions = runtime.auto_apply_standard_actions;
        });

        let children = render_children(
            cx,
            spec,
            runtime,
            resolver,
            state_snapshot,
            element,
            repeat_scope,
            repeat_base_path.clone(),
            repeat_index,
            depth + 1,
            rendered_count,
            stack,
        )?;

        let rendered = resolver
            .render_element(cx, key, element, &props, children, &|ev| {
                dispatcher.on_event(ev)
            })
            .map_err(|err| RenderError::Component {
                key: key.clone(),
                component: element.ty.clone(),
                message: err.to_string(),
            })?;

        *rendered_count = rendered_count.saturating_add(1);

        // Presence semantics: always render subtree, but gate layout/paint/input when invisible.
        let rendered = if visible {
            rendered
        } else {
            cx.interactivity_gate(false, false, |_cx| [rendered])
        };

        Ok::<_, RenderError>(RenderedChildV1 {
            key: key.clone(),
            component: element.ty.clone(),
            props,
            rendered,
        })
    })?;

    let _ = stack.pop();
    Ok(out)
}

fn render_children<H: UiHost, R: ComponentResolver<H>>(
    cx: &mut ElementContext<'_, H>,
    spec: &SpecV1,
    runtime: &GenUiRuntime,
    resolver: &mut R,
    state_snapshot: &Value,
    element: &ElementV1,
    repeat_scope: RepeatScope<'_>,
    repeat_base_path: Option<Arc<str>>,
    repeat_index: Option<usize>,
    depth: usize,
    rendered_count: &mut usize,
    stack: &mut Vec<ElementKey>,
) -> Result<Vec<RenderedChildV1>, RenderError> {
    if let Some(repeat) = element.repeat.as_ref() {
        let Some(list) = crate::json_pointer::get_opt(state_snapshot, &repeat.state_path) else {
            return Ok(Vec::new());
        };
        let Some(arr) = list.as_array() else {
            return Err(RenderError::RepeatNotArray {
                path: repeat.state_path.clone(),
            });
        };

        let mut out: Vec<RenderedChildV1> = Vec::new();
        for (index, item) in arr.iter().enumerate() {
            if index >= runtime.limits.max_repeat_items {
                return Err(RenderError::LimitExceeded {
                    kind: "max_repeat_items",
                });
            }

            let item_key = repeat
                .key
                .as_ref()
                .and_then(|k| item.as_object().and_then(|o| o.get(k)))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| index.to_string());

            let base_path = Arc::<str>::from(format!("{}/{}", repeat.state_path, index));
            let child_scope = RepeatScope {
                item: Some(item),
                index: Some(index),
                base_path: Some(base_path.as_ref()),
            };

            let item_children: Vec<RenderedChildV1> = cx.keyed(item_key, |cx| {
                let mut built: Vec<RenderedChildV1> = Vec::new();
                for child_key in element.children.iter() {
                    let child = render_element_node(
                        cx,
                        spec,
                        runtime,
                        resolver,
                        state_snapshot,
                        child_key,
                        child_scope,
                        Some(base_path.clone()),
                        Some(index),
                        depth,
                        rendered_count,
                        stack,
                    )?;
                    built.push(child);
                }
                Ok::<_, RenderError>(built)
            })?;

            out.extend(item_children);
        }
        return Ok(out);
    }

    let mut out: Vec<RenderedChildV1> = Vec::new();
    for child_key in element.children.iter() {
        out.push(render_element_node(
            cx,
            spec,
            runtime,
            resolver,
            state_snapshot,
            child_key,
            repeat_scope,
            repeat_base_path.clone(),
            repeat_index,
            depth,
            rendered_count,
            stack,
        )?);
    }
    Ok(out)
}
