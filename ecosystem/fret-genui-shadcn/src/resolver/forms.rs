use std::sync::Arc;

use fret_genui_core::props::ResolvedProps;
use fret_genui_core::spec::ElementKey;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::Value;

use super::ShadcnResolver;

impl ShadcnResolver {
    pub(super) fn render_input<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;
        let placeholder = resolved_props
            .get("placeholder")
            .and_then(|v| v.as_str())
            .map(Arc::<str>::from);

        let desired = Self::json_to_label(resolved_props.get("value")).to_string();

        let model = Self::ensure_string_model(cx, desired.clone());

        let cur = cx.app.models().get_cloned(&model).unwrap_or_default();

        if let (Some(scope), Some(path)) = (Self::genui_scope(cx), props.bindings.get("value")) {
            #[derive(Default)]
            struct LastState {
                last_model: Option<String>,
                last_desired: Option<String>,
            }

            let mut to_emit: Option<String> = None;
            let mut sync_model_to: Option<String> = None;
            cx.with_state(LastState::default, |st| {
                let model_changed = st.last_model.as_deref().is_some_and(|v| v != cur.as_str());
                let desired_changed = st
                    .last_desired
                    .as_deref()
                    .is_some_and(|v| v != desired.as_str());

                if model_changed && cur != desired {
                    to_emit = Some(cur.clone());
                } else if desired_changed && !model_changed && cur != desired {
                    sync_model_to = Some(desired.clone());
                }

                st.last_model = Some(cur.clone());
                st.last_desired = Some(desired.clone());
            });

            if let Some(v) = sync_model_to {
                let _ = cx.app.models_mut().update(&model, |m| *m = v);
            }
            if let Some(v) = to_emit {
                Self::emit_set_state(cx, &scope, key, "change", path.as_str(), Value::String(v));
            }
        } else if cur != desired {
            // Treat as a controlled prop when no binding is provided.
            let _ = cx.app.models_mut().update(&model, |v| *v = desired.clone());
        }

        let mut input = fret_ui_shadcn::Input::new(model);
        if let Some(placeholder) = placeholder {
            input = input.placeholder(placeholder);
        }
        input = input.a11y_role(fret_core::SemanticsRole::TextField);

        let input = input.into_element(cx);
        if children.is_empty() {
            input
        } else {
            fret_ui_kit::ui::v_flex(cx, move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(input);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }

    pub(super) fn render_switch<H: UiHost>(
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
            if let Some(cur) = to_emit {
                Self::emit_set_state(cx, &scope, key, "change", path.as_str(), Value::Bool(cur));
            }
        }

        let sw = fret_ui_shadcn::Switch::new(model).into_element(cx);
        if children.is_empty() {
            sw
        } else {
            fret_ui_kit::ui::v_flex(cx, move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(sw);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }
}
