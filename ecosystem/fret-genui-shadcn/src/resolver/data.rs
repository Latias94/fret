use std::sync::Arc;

use fret_genui_core::props::{PropResolutionContext, resolve_action_param};
use fret_genui_core::render::GenUiActionQueue;
use fret_genui_core::{actions, json_pointer};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnActivate, UiActionHost, UiActionHostExt};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::facade as shadcn;
use serde_json::{Map, Value};

use super::ShadcnResolver;

#[derive(Debug, Clone)]
struct TableColumnDef {
    key: Arc<str>,
    label: Arc<str>,
}

#[derive(Debug, Clone)]
struct TableRowActionDef {
    label: Arc<str>,
    action: Arc<str>,
    params: Map<String, Value>,
    variant: Option<Arc<str>>,
    disabled: bool,
}

fn coerce_items(data: &Value) -> Vec<Value> {
    match data {
        Value::Array(a) => a.clone(),
        Value::Object(o) => {
            if let Some(Value::Array(a)) = o.get("data") {
                return a.clone();
            }
            if let Some(Value::Array(a)) = o.get("items") {
                return a.clone();
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

fn parse_columns(v: Option<&Value>) -> Vec<TableColumnDef> {
    let Some(Value::Array(cols)) = v else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for col in cols {
        let Some(obj) = col.as_object() else {
            continue;
        };
        let Some(key) = obj.get("key").and_then(|v| v.as_str()) else {
            continue;
        };
        let label = obj.get("label").and_then(|v| v.as_str()).unwrap_or(key);
        out.push(TableColumnDef {
            key: Arc::<str>::from(key),
            label: Arc::<str>::from(label),
        });
    }
    out
}

fn parse_row_actions(v: Option<&Value>) -> Vec<TableRowActionDef> {
    let Some(Value::Array(items)) = v else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let Some(label) = obj.get("label").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(action) = obj.get("action").and_then(|v| v.as_str()) else {
            continue;
        };
        let params = obj
            .get("params")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(Map::new);
        let variant = obj
            .get("variant")
            .and_then(|v| v.as_str())
            .map(Arc::<str>::from);
        let disabled = obj
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        out.push(TableRowActionDef {
            label: Arc::<str>::from(label),
            action: Arc::<str>::from(action),
            params,
            variant,
            disabled,
        });
    }
    out
}

fn cell_text_for_value(v: Option<&Value>) -> Arc<str> {
    match v {
        None => Arc::<str>::from(""),
        Some(Value::Null) => Arc::<str>::from(""),
        Some(Value::String(s)) => Arc::<str>::from(s.as_str()),
        Some(other) => Arc::<str>::from(other.to_string()),
    }
}

fn emit_row_action(
    host: &mut dyn UiActionHost,
    cx: ActionCx,
    element_key: Arc<str>,
    event: Arc<str>,
    action: Arc<str>,
    params_raw: Map<String, Value>,
    state_model: Model<Value>,
    queue: Option<Model<GenUiActionQueue>>,
    auto_apply_standard_actions: bool,
    data_path: Option<Arc<str>>,
    row_index: usize,
) {
    let state_snapshot: Value = host
        .models_mut()
        .read(&state_model, Clone::clone)
        .unwrap_or(Value::Null);

    let repeat_item_value: Option<Value> = data_path.as_ref().and_then(|p| {
        json_pointer::get_opt(&state_snapshot, p.as_ref())
            .and_then(|v| v.as_array())
            .and_then(|a| a.get(row_index))
            .cloned()
    });

    let repeat_base_path: Option<Arc<str>> = data_path
        .as_ref()
        .map(|p| Arc::<str>::from(format!("{}/{row_index}", p.as_ref())));

    let prop_ctx = PropResolutionContext {
        state: &state_snapshot,
        repeat: fret_genui_core::visibility::RepeatScope {
            item: repeat_item_value.as_ref(),
            index: data_path.as_ref().map(|_| row_index),
            base_path: repeat_base_path.as_deref(),
        },
    };

    let params = Value::Object(
        params_raw
            .iter()
            .map(|(k, v)| (k.clone(), resolve_action_param(v, &prop_ctx)))
            .collect(),
    );

    let inv = fret_genui_core::render::GenUiActionInvocation {
        window: cx.window,
        source: cx.target,
        element_key,
        event,
        action: action.clone(),
        params: params.clone(),
        confirm: None,
        on_success: None,
        on_error: None,
        repeat_base_path,
        repeat_index: data_path.as_ref().map(|_| row_index),
    };

    if let Some(queue) = queue.as_ref() {
        let _ = host.update_model(queue, |q| q.invocations.push(inv));
    }

    if auto_apply_standard_actions {
        let _ = host.update_model(&state_model, |st| {
            actions::apply_standard_action(st, action.as_ref(), &params)
        });
    }

    host.request_redraw(cx.window);
}

impl ShadcnResolver {
    pub(super) fn render_table<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &fret_genui_core::spec::ElementKey,
        props: &fret_genui_core::props::ResolvedProps,
    ) -> AnyElement {
        let resolved = &props.props;

        let columns = parse_columns(resolved.get("columns"));
        if columns.is_empty() {
            return self.unknown_component(cx, key, "Table (missing columns)");
        }

        let data = resolved.get("data").cloned().unwrap_or(Value::Null);
        let items = coerce_items(&data);

        let empty_message = resolved
            .get("emptyMessage")
            .and_then(|v| v.as_str())
            .unwrap_or("No data");

        if items.is_empty() {
            let msg = Arc::<str>::from(empty_message);
            return shadcn::Card::new([shadcn::CardContent::new([
                shadcn::raw::typography::muted(msg).into_element(cx),
            ])
            .into_element(cx)])
            .into_element(cx);
        }

        let row_actions = parse_row_actions(resolved.get("rowActions"));
        let cols =
            u16::try_from(
                columns
                    .len()
                    .saturating_add(if row_actions.is_empty() { 0 } else { 1 }),
            )
            .unwrap_or(u16::MAX);

        let data_path = resolved
            .get("dataPath")
            .and_then(|v| v.as_str())
            .map(Arc::<str>::from);

        let scope = Self::genui_scope(cx);
        let state_model = scope.as_ref().and_then(|s| s.state.clone());
        let queue = scope.as_ref().and_then(|s| s.action_queue.clone());
        let auto_apply = scope
            .as_ref()
            .map(|s| s.auto_apply_standard_actions)
            .unwrap_or(false);

        shadcn::Table::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::TableHeader::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::TableRow::build(cols, |cx, out| {
                            for c in columns.iter() {
                                out.push_ui(cx, shadcn::TableHead::new(c.label.clone()));
                            }
                            if !row_actions.is_empty() {
                                out.push_ui(
                                    cx,
                                    shadcn::TableHead::new(Arc::<str>::from("Actions")),
                                );
                            }
                        }),
                    );
                }),
            );

            out.push_ui(
                cx,
                shadcn::TableBody::build(|cx, out| {
                    for (row_index, item) in items.iter().enumerate() {
                        let obj = item.as_object();
                        out.push_ui(
                            cx,
                            shadcn::TableRow::build(cols, |cx, out| {
                                for col in columns.iter() {
                                    let v = obj.and_then(|o| o.get(col.key.as_ref()));
                                    let text = cell_text_for_value(v);
                                    out.push_ui(
                                        cx,
                                        shadcn::TableCell::build(fret_ui_kit::ui::text(text)),
                                    );
                                }

                                if !row_actions.is_empty() {
                                    out.push_ui(
                                        cx,
                                        shadcn::TableCell::build(
                                            fret_ui_kit::ui::h_flex_build(|cx, out| {
                                                for ra in row_actions.iter() {
                                                    let label = ra.label.clone();
                                                    let action = ra.action.clone();
                                                    let params_raw = ra.params.clone();
                                                    let disabled = ra.disabled;
                                                    let element_key: Arc<str> = Arc::<str>::from(
                                                        format!("{}/row/{row_index}", key.0),
                                                    );
                                                    let event: Arc<str> = Arc::<str>::from(
                                                        format!("rowAction.{}", action.as_ref()),
                                                    );
                                                    let data_path = data_path.clone();

                                                    let mut btn =
                                                        shadcn::Button::new(label.clone())
                                                            .disabled(disabled)
                                                            .size(shadcn::ButtonSize::Sm);
                                                    if let Some(variant) = ra.variant.as_ref() {
                                                        let parsed = match variant.as_ref() {
                                                            "default" => {
                                                                Some(shadcn::ButtonVariant::Default)
                                                            }
                                                            "destructive" => Some(
                                                                shadcn::ButtonVariant::Destructive,
                                                            ),
                                                            "outline" => {
                                                                Some(shadcn::ButtonVariant::Outline)
                                                            }
                                                            "secondary" => Some(
                                                                shadcn::ButtonVariant::Secondary,
                                                            ),
                                                            "ghost" => {
                                                                Some(shadcn::ButtonVariant::Ghost)
                                                            }
                                                            "link" => {
                                                                Some(shadcn::ButtonVariant::Link)
                                                            }
                                                            _ => None,
                                                        };
                                                        if let Some(parsed) = parsed {
                                                            btn = btn.variant(parsed);
                                                        }
                                                    }

                                                    if let Some(state_model) = state_model.clone() {
                                                        let queue = queue.clone();
                                                        let auto_apply = auto_apply;
                                                        let on_activate: OnActivate =
                                                            Arc::new(move |host, acx, _reason| {
                                                                emit_row_action(
                                                                    host,
                                                                    acx,
                                                                    element_key.clone(),
                                                                    event.clone(),
                                                                    action.clone(),
                                                                    params_raw.clone(),
                                                                    state_model.clone(),
                                                                    queue.clone(),
                                                                    auto_apply,
                                                                    data_path.clone(),
                                                                    row_index,
                                                                );
                                                            });
                                                        btn = btn.on_activate(on_activate);
                                                    }
                                                    out.push(btn.into_element(cx));
                                                }
                                            })
                                            .gap(fret_ui_kit::Space::N1)
                                            .wrap(),
                                        ),
                                    );
                                }
                            }),
                        );
                    }
                }),
            );
        })
        .into_element(cx)
    }
}
