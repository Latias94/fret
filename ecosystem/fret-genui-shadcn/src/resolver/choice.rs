use std::sync::Arc;

use fret_genui_core::props::ResolvedProps;
use fret_genui_core::spec::ElementKey;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::Value;

use super::ShadcnResolver;

impl ShadcnResolver {
    pub(super) fn render_checkbox<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;
        let desired = resolved_props
            .get("checked")
            .and_then(|v| v.as_bool())
            .or_else(|| {
                resolved_props
                    .get("defaultChecked")
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(false);

        let model = Self::ensure_bool_model(cx, desired);

        if let (Some(scope), Some(path)) = (Self::genui_scope(cx), props.bindings.get("checked")) {
            #[derive(Default)]
            struct LastState {
                last_model: Option<bool>,
                last_desired: Option<bool>,
            }

            let cur = cx.app.models().get_copied(&model).unwrap_or(false);
            let mut to_emit: Option<bool> = None;
            let mut sync_model_to: Option<bool> = None;
            cx.with_state(LastState::default, |st| {
                let model_changed = st.last_model.is_some_and(|v| v != cur);
                let desired_changed = st.last_desired.is_some_and(|v| v != desired);

                if model_changed && cur != desired {
                    to_emit = Some(cur);
                } else if desired_changed && !model_changed && cur != desired {
                    sync_model_to = Some(desired);
                }

                st.last_model = Some(cur);
                st.last_desired = Some(desired);
            });

            if let Some(v) = sync_model_to {
                let _ = cx.app.models_mut().update(&model, |m| *m = v);
            }
            if let Some(v) = to_emit {
                Self::emit_set_state(cx, &scope, key, "change", path.as_str(), Value::Bool(v));
            }
        } else {
            let cur = cx.app.models().get_copied(&model).unwrap_or(false);
            if cur != desired {
                let _ = cx.app.models_mut().update(&model, |m| *m = desired);
            }
        }

        let disabled = resolved_props
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let checkbox = fret_ui_shadcn::Checkbox::new(model)
            .disabled(disabled)
            .into_element(cx);

        let label = resolved_props
            .get("label")
            .and_then(|v| (!v.is_null()).then(|| Self::json_to_label(Some(v))));

        let mut out_children: Vec<AnyElement> = Vec::new();
        if let Some(label) = label {
            out_children.push(
                fret_ui_kit::ui::h_flex(cx, move |_cx| {
                    vec![
                        checkbox.clone(),
                        fret_ui_kit::ui::text(_cx, label).into_element(_cx),
                    ]
                })
                .gap(fret_ui_kit::Space::N2)
                .items_center()
                .into_element(cx),
            );
        } else {
            out_children.push(checkbox);
        }
        out_children.extend(children);

        if out_children.len() == 1 {
            out_children.pop().expect("single child")
        } else {
            fret_ui_kit::ui::v_flex(cx, move |_cx| out_children)
                .gap(fret_ui_kit::Space::N2)
                .items_start()
                .into_element(cx)
        }
    }

    pub(super) fn render_select<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;

        let desired = Self::json_to_option_arc_str(resolved_props.get("value"));
        let model = Self::ensure_optional_arc_str_model(cx, desired.clone());
        let open = Self::ensure_bool_model(cx, false);

        let cur = cx.app.models().get_cloned(&model).unwrap_or(None);
        if let (Some(scope), Some(path)) = (Self::genui_scope(cx), props.bindings.get("value")) {
            #[derive(Default)]
            struct LastState {
                last_model: Option<Option<String>>,
                last_desired: Option<Option<String>>,
            }

            let cur_s = cur.as_deref().map(|s| s.to_string());
            let desired_s = desired.as_deref().map(|s| s.to_string());
            let mut to_emit: Option<Option<Arc<str>>> = None;
            let mut sync_model_to: Option<Option<Arc<str>>> = None;
            cx.with_state(LastState::default, |st| {
                let model_changed = st.last_model.as_ref().is_some_and(|v| v != &cur_s);
                let desired_changed = st.last_desired.as_ref().is_some_and(|v| v != &desired_s);

                if model_changed && cur_s != desired_s {
                    to_emit = Some(cur.clone());
                } else if desired_changed && !model_changed && cur_s != desired_s {
                    sync_model_to = Some(desired.clone());
                }

                st.last_model = Some(cur_s.clone());
                st.last_desired = Some(desired_s.clone());
            });

            if let Some(v) = sync_model_to {
                let _ = cx.app.models_mut().update(&model, |m| *m = v);
            }
            if let Some(v) = to_emit {
                let value = v
                    .as_deref()
                    .map(|s| Value::String(s.to_string()))
                    .unwrap_or(Value::Null);
                Self::emit_set_state(cx, &scope, key, "change", path.as_str(), value);
            }
        } else if cur != desired {
            // Treat as a controlled prop when no binding is provided.
            let _ = cx.app.models_mut().update(&model, |m| *m = desired.clone());
        }

        let placeholder = resolved_props
            .get("placeholder")
            .and_then(|v| v.as_str())
            .map(Arc::<str>::from);
        let disabled = resolved_props
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let aria_invalid = resolved_props
            .get("ariaInvalid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let w_full = resolved_props
            .get("wFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let flex_1 = resolved_props
            .get("flex1")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_w_0 = resolved_props
            .get("minW0")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut layout = fret_ui_kit::LayoutRefinement::default();
        if w_full {
            layout = layout.w_full();
        }
        if flex_1 {
            layout = layout.flex_1();
        }
        if min_w_0 {
            layout = layout.min_w_0();
        }

        let options: Vec<fret_ui_shadcn::SelectItem> = resolved_props
            .get("options")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_object())
                    .filter_map(|obj| {
                        let value = obj.get("value")?.as_str()?;
                        let label = Self::json_to_label(obj.get("label"));
                        let disabled = obj
                            .get("disabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        Some(
                            fret_ui_shadcn::SelectItem::new(Arc::<str>::from(value), label)
                                .disabled(disabled),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut select = fret_ui_shadcn::Select::new(model, open)
            .items(options)
            .disabled(disabled)
            .refine_layout(layout);
        if let Some(placeholder) = placeholder {
            select = select.placeholder(placeholder);
        }
        select = select.aria_invalid(aria_invalid);

        let select = select.into_element(cx);

        if children.is_empty() {
            select
        } else {
            fret_ui_kit::ui::v_flex(cx, move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(select);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }

    pub(super) fn render_radio_group<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;

        let desired = Self::json_to_option_arc_str(resolved_props.get("value"));
        let model = Self::ensure_optional_arc_str_model(cx, desired.clone());
        let cur = cx.app.models().get_cloned(&model).unwrap_or(None);

        if let (Some(scope), Some(path)) = (Self::genui_scope(cx), props.bindings.get("value")) {
            #[derive(Default)]
            struct LastState {
                last_model: Option<Option<String>>,
                last_desired: Option<Option<String>>,
            }

            let cur_s = cur.as_deref().map(|s| s.to_string());
            let desired_s = desired.as_deref().map(|s| s.to_string());
            let mut to_emit: Option<Option<Arc<str>>> = None;
            let mut sync_model_to: Option<Option<Arc<str>>> = None;
            cx.with_state(LastState::default, |st| {
                let model_changed = st.last_model.as_ref().is_some_and(|v| v != &cur_s);
                let desired_changed = st.last_desired.as_ref().is_some_and(|v| v != &desired_s);

                if model_changed && cur_s != desired_s {
                    to_emit = Some(cur.clone());
                } else if desired_changed && !model_changed && cur_s != desired_s {
                    sync_model_to = Some(desired.clone());
                }

                st.last_model = Some(cur_s.clone());
                st.last_desired = Some(desired_s.clone());
            });

            if let Some(v) = sync_model_to {
                let _ = cx.app.models_mut().update(&model, |m| *m = v);
            }
            if let Some(v) = to_emit {
                let value = v
                    .as_deref()
                    .map(|s| Value::String(s.to_string()))
                    .unwrap_or(Value::Null);
                Self::emit_set_state(cx, &scope, key, "change", path.as_str(), value);
            }
        } else if cur != desired {
            // Treat as a controlled prop when no binding is provided.
            let _ = cx.app.models_mut().update(&model, |m| *m = desired.clone());
        }

        let options: Vec<fret_ui_shadcn::RadioGroupItem> = resolved_props
            .get("options")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_object())
                    .filter_map(|obj| {
                        let value = obj.get("value")?.as_str()?;
                        let label = Self::json_to_label(obj.get("label"));
                        let disabled = obj
                            .get("disabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        Some(
                            fret_ui_shadcn::RadioGroupItem::new(Arc::<str>::from(value), label)
                                .disabled(disabled),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        let disabled = resolved_props
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let aria_invalid = resolved_props
            .get("ariaInvalid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let orientation = resolved_props
            .get("orientation")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "vertical" => Some(fret_ui_shadcn::radio_group::RadioGroupOrientation::Vertical),
                "horizontal" => {
                    Some(fret_ui_shadcn::radio_group::RadioGroupOrientation::Horizontal)
                }
                _ => None,
            })
            .unwrap_or(fret_ui_shadcn::radio_group::RadioGroupOrientation::Vertical);

        let w_full = resolved_props
            .get("wFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let flex_1 = resolved_props
            .get("flex1")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_w_0 = resolved_props
            .get("minW0")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut layout = fret_ui_kit::LayoutRefinement::default();
        if w_full {
            layout = layout.w_full();
        }
        if flex_1 {
            layout = layout.flex_1();
        }
        if min_w_0 {
            layout = layout.min_w_0();
        }

        let mut group = fret_ui_shadcn::RadioGroup::new(model)
            .disabled(disabled)
            .orientation(orientation)
            .refine_layout(layout);
        for item in options.into_iter() {
            group = group.item(if aria_invalid {
                item.aria_invalid(true)
            } else {
                item
            });
        }
        let group = group.into_element(cx);

        if children.is_empty() {
            group
        } else {
            fret_ui_kit::ui::v_flex(cx, move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(group);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }
}
