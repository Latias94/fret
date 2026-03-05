use std::sync::Arc;

use fret_genui_core::props::ResolvedProps;
use fret_genui_core::spec::ElementKey;
use fret_ui::action::{ActivateReason, OnActivate, OnKeyDown};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::Value;

use super::ShadcnResolver;

impl ShadcnResolver {
    pub(super) fn render_label<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let text = Self::json_to_label(resolved_props.get("text"));
        let label = fret_ui_shadcn::Label::new(text).into_element(cx);
        if children.is_empty() {
            label
        } else {
            fret_ui_kit::ui::v_flex(move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(label);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }

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
        let disabled = resolved_props
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let aria_invalid = resolved_props
            .get("ariaInvalid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

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
        input = input.disabled(disabled).aria_invalid(aria_invalid);
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

        input = input.a11y_role(fret_core::SemanticsRole::TextField);
        input = input.refine_layout(layout);

        let input = input.into_element(cx);
        let label = resolved_props
            .get("label")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(Arc::<str>::from);

        if children.is_empty() && label.is_none() {
            return input;
        }

        fret_ui_kit::ui::v_flex(move |_cx| {
            let mut out = Vec::with_capacity(children.len().saturating_add(2));
            if let Some(label) = label.clone() {
                out.push(fret_ui_shadcn::Label::new(label).into_element(_cx));
            }
            out.push(input);
            out.extend(children);
            out
        })
        .gap(fret_ui_kit::Space::N2)
        .items_start()
        .into_element(cx)
    }

    pub(super) fn render_textarea<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;
        let disabled = resolved_props
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let aria_invalid = resolved_props
            .get("ariaInvalid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_height_px = resolved_props
            .get("minHeightPx")
            .and_then(|v| v.as_u64())
            .unwrap_or(64) as f32;

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

        let textarea = fret_ui_shadcn::Textarea::new(model)
            .disabled(disabled)
            .aria_invalid(aria_invalid)
            .min_height(fret_core::Px(min_height_px))
            .refine_layout(layout)
            .into_element(cx);

        let label = resolved_props
            .get("label")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(Arc::<str>::from);

        if children.is_empty() && label.is_none() {
            return textarea;
        }

        fret_ui_kit::ui::v_flex(move |_cx| {
            let mut out = Vec::with_capacity(children.len().saturating_add(2));
            if let Some(label) = label.clone() {
                out.push(fret_ui_shadcn::Label::new(label).into_element(_cx));
            }
            out.push(textarea);
            out.extend(children);
            out
        })
        .gap(fret_ui_kit::Space::N2)
        .items_start()
        .into_element(cx)
    }

    pub(super) fn render_form<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        _key: &ElementKey,
        children: Vec<AnyElement>,
        on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> AnyElement {
        let on_submit = on_event("submit");

        let out = fret_ui_kit::ui::v_flex(move |_cx| children)
            .gap(fret_ui_kit::Space::N4)
            .items_start()
            .w_full()
            .into_element(cx);

        if let Some(on_submit) = on_submit {
            let handler: OnKeyDown = Arc::new(move |host, acx, down| {
                let enter = matches!(
                    down.key,
                    fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter
                );
                if !enter || down.repeat || down.modifiers != fret_core::Modifiers::default() {
                    return false;
                }
                on_submit(host, acx, ActivateReason::Keyboard);
                true
            });
            cx.key_on_key_down_for(out.id, handler);
        }

        out
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
            .or_else(|| {
                resolved_props
                    .get("defaultChecked")
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(false);
        let disabled = resolved_props
            .get("disabled")
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

        let sw = fret_ui_shadcn::Switch::new(model)
            .disabled(disabled)
            .into_element(cx);

        let label = resolved_props
            .get("label")
            .and_then(|v| (!v.is_null()).then(|| Self::json_to_label(Some(v))));

        let mut out_children: Vec<AnyElement> = Vec::new();
        if let Some(label) = label {
            out_children.push(
                fret_ui_kit::ui::h_flex(move |_cx| {
                    vec![sw, fret_ui_kit::ui::text(label).into_element(_cx)]
                })
                .gap(fret_ui_kit::Space::N2)
                .items_center()
                .into_element(cx),
            );
        } else {
            out_children.push(sw);
        }
        out_children.extend(children);

        if out_children.len() == 1 {
            out_children.pop().expect("single child")
        } else {
            fret_ui_kit::ui::v_flex(move |_cx| out_children)
                .gap(fret_ui_kit::Space::N2)
                .items_start()
                .into_element(cx)
        }
    }
}
