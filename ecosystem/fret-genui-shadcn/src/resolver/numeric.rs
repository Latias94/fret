use std::sync::Arc;

use fret_genui_core::props::ResolvedProps;
use fret_genui_core::spec::ElementKey;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::Value;

use super::ShadcnResolver;

fn parse_f32(v: Option<&serde_json::Value>) -> Option<f32> {
    let v = v?;
    if let Some(f) = v.as_f64() {
        return Some(f as f32);
    }
    if let Some(i) = v.as_i64() {
        return Some(i as f32);
    }
    if let Some(u) = v.as_u64() {
        return Some(u as f32);
    }
    if let Some(s) = v.as_str() {
        return s.parse::<f32>().ok();
    }
    None
}

impl ShadcnResolver {
    pub(super) fn render_slider<H: UiHost>(
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
        let min = parse_f32(resolved_props.get("min")).unwrap_or(0.0);
        let max = parse_f32(resolved_props.get("max")).unwrap_or(100.0);
        let step = parse_f32(resolved_props.get("step"))
            .unwrap_or(1.0)
            .max(0.000_1);

        let desired = parse_f32(resolved_props.get("value"))
            .unwrap_or(min)
            .clamp(min, max);
        let desired_values = vec![desired];
        let model = Self::ensure_vec_f32_model(cx, key, desired_values.clone());

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

        let mut slider = fret_ui_shadcn::Slider::new(model.clone())
            .range(min, max)
            .step(step)
            .disabled(disabled)
            .refine_layout(layout);

        if let (Some(scope), Some(path)) = (Self::genui_scope(cx), props.bindings.get("value")) {
            // Sync model from desired state when the resolved prop changes.
            #[derive(Default)]
            struct LastState {
                last_desired: Option<f32>,
            }
            let mut sync_model_to: Option<Vec<f32>> = None;
            cx.keyed_slot_state(key.clone(), LastState::default, |st| {
                let desired_changed = st.last_desired.is_some_and(|v| v != desired);
                st.last_desired = Some(desired);
                if desired_changed {
                    sync_model_to = Some(desired_values.clone());
                }
            });
            if let Some(v) = sync_model_to {
                let _ = cx.app.models_mut().update(&model, |m| *m = v);
            }

            let queue = scope.action_queue.clone();
            let state_model = scope.state.clone();
            let auto_apply = scope.auto_apply_standard_actions;
            let element_key: Arc<str> = Arc::from(key.0.as_str());
            let state_path: Arc<str> = Arc::from(path.as_str());

            slider = slider.on_value_commit(move |host: &mut dyn UiActionHost, cx: ActionCx, v| {
                let Some(raw) = v.first().copied() else {
                    return;
                };
                let clamped = raw.clamp(min, max);
                let Some(n) = serde_json::Number::from_f64(clamped as f64) else {
                    return;
                };

                let value = Value::Number(n);
                Self::emit_set_state_action(
                    host,
                    cx,
                    queue.as_ref(),
                    state_model.as_ref(),
                    auto_apply,
                    &element_key,
                    "commit",
                    &state_path,
                    value,
                );
            });
        } else {
            // Treat as a controlled prop when no binding is provided.
            let cur = cx.app.models().get_cloned(&model).unwrap_or_default();
            if cur != desired_values {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&model, |m| *m = desired_values.clone());
            }
        }

        let slider = slider.into_element(cx);
        if children.is_empty() {
            slider
        } else {
            fret_ui_kit::ui::v_flex(move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(slider);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }
}
