use std::sync::Arc;

use fret_genui_core::actions;
use fret_genui_core::render::{GenUiActionInvocation, GenUiActionQueue, GenUiRenderScope};
use fret_genui_core::spec::ElementKey;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost, UiActionHostExt};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::Value;

use super::ShadcnResolver;

impl ShadcnResolver {
    pub(super) fn text_element<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        text: Arc<str>,
    ) -> AnyElement {
        fret_ui_kit::ui::text(text).into_element(cx)
    }

    pub(super) fn json_to_label(v: Option<&serde_json::Value>) -> Arc<str> {
        let Some(v) = v else {
            return Arc::<str>::from("");
        };
        if let Some(s) = v.as_str() {
            return Arc::<str>::from(s);
        }
        Arc::<str>::from(v.to_string())
    }

    pub(super) fn json_to_option_arc_str(v: Option<&serde_json::Value>) -> Option<Arc<str>> {
        let Some(v) = v else {
            return None;
        };
        if v.is_null() {
            return None;
        }
        if let Some(s) = v.as_str() {
            return Some(Arc::<str>::from(s));
        }
        Some(Arc::<str>::from(v.to_string()))
    }

    pub(super) fn unknown_component<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        component: &str,
    ) -> AnyElement {
        let msg = Arc::<str>::from(format!("Unknown GenUI component: {component} ({:?})", key));
        fret_ui_shadcn::Card::new([
            fret_ui_shadcn::CardContent::new([Self::text_element(cx, msg)]).into_element(cx),
        ])
        .into_element(cx)
    }

    pub(super) fn parse_space(v: Option<&serde_json::Value>) -> Option<fret_ui_kit::Space> {
        let s = v?.as_str()?;
        use fret_ui_kit::Space;
        Some(match s {
            "N0" => Space::N0,
            "N0p5" => Space::N0p5,
            "N1" => Space::N1,
            "N1p5" => Space::N1p5,
            "N2" => Space::N2,
            "N2p5" => Space::N2p5,
            "N3" => Space::N3,
            "N3p5" => Space::N3p5,
            "N4" => Space::N4,
            "N5" => Space::N5,
            "N6" => Space::N6,
            "N8" => Space::N8,
            "N10" => Space::N10,
            "N11" => Space::N11,
            "N12" => Space::N12,
            _ => return None,
        })
    }

    pub(super) fn parse_badge_variant(
        v: Option<&serde_json::Value>,
    ) -> Option<fret_ui_shadcn::BadgeVariant> {
        let s = v?.as_str()?;
        use fret_ui_shadcn::BadgeVariant;
        Some(match s {
            "default" => BadgeVariant::Default,
            "secondary" => BadgeVariant::Secondary,
            "destructive" => BadgeVariant::Destructive,
            "outline" => BadgeVariant::Outline,
            _ => return None,
        })
    }

    pub(super) fn parse_button_variant(
        v: Option<&serde_json::Value>,
    ) -> Option<fret_ui_shadcn::ButtonVariant> {
        let s = v?.as_str()?;
        use fret_ui_shadcn::ButtonVariant;
        Some(match s {
            "default" => ButtonVariant::Default,
            "destructive" => ButtonVariant::Destructive,
            "outline" => ButtonVariant::Outline,
            "secondary" => ButtonVariant::Secondary,
            "ghost" => ButtonVariant::Ghost,
            "link" => ButtonVariant::Link,
            _ => return None,
        })
    }

    pub(super) fn parse_button_size(
        v: Option<&serde_json::Value>,
    ) -> Option<fret_ui_shadcn::ButtonSize> {
        let s = v?.as_str()?;
        use fret_ui_shadcn::ButtonSize;
        Some(match s {
            "default" => ButtonSize::Default,
            "sm" => ButtonSize::Sm,
            "lg" => ButtonSize::Lg,
            "icon" => ButtonSize::Icon,
            "iconSm" => ButtonSize::IconSm,
            "iconLg" => ButtonSize::IconLg,
            _ => return None,
        })
    }

    pub(super) fn genui_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<GenUiRenderScope> {
        cx.inherited_state::<GenUiRenderScope>().cloned()
    }

    pub(super) fn emit_set_state<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        scope: &GenUiRenderScope,
        element_key: &ElementKey,
        event: &str,
        state_path: &str,
        value: Value,
    ) {
        let element_id = cx.root_id();
        let params = Value::Object(
            [
                (
                    "statePath".to_string(),
                    Value::String(state_path.to_string()),
                ),
                ("value".to_string(), value),
            ]
            .into_iter()
            .collect(),
        );

        // Preferred path: emit into the queue (app decides when/how to apply).
        if let Some(queue) = scope.action_queue.as_ref() {
            if scope.auto_apply_standard_actions {
                if let Some(state_model) = scope.state.as_ref() {
                    let _ = cx.app.models_mut().update(state_model, |state| {
                        actions::apply_standard_action(state, "setState", &params)
                    });
                }
            }

            let inv = GenUiActionInvocation {
                window: cx.window,
                source: element_id,
                element_key: Arc::from(element_key.0.as_str()),
                event: Arc::from(event),
                action: Arc::from("setState"),
                params,
                confirm: None,
                on_success: None,
                on_error: None,
                repeat_base_path: None,
                repeat_index: None,
            };

            let _ = cx
                .app
                .models_mut()
                .update(queue, |q: &mut GenUiActionQueue| q.invocations.push(inv));
            cx.app.request_redraw(cx.window);
            return;
        }

        // Fallback: apply directly if no queue is available.
        let Some(state_model) = scope.state.as_ref() else {
            return;
        };
        let _ = cx.app.models_mut().update(state_model, |state| {
            actions::apply_standard_action(state, "setState", &params)
        });
        cx.app.request_redraw(cx.window);
    }

    pub(super) fn emit_set_state_action(
        host: &mut dyn UiActionHost,
        cx: ActionCx,
        queue: Option<&Model<GenUiActionQueue>>,
        state_model: Option<&Model<Value>>,
        auto_apply_standard_actions: bool,
        element_key: &Arc<str>,
        event: &str,
        state_path: &Arc<str>,
        value: Value,
    ) {
        let params = Value::Object(
            [
                (
                    "statePath".to_string(),
                    Value::String(state_path.to_string()),
                ),
                ("value".to_string(), value),
            ]
            .into_iter()
            .collect(),
        );

        // Preferred path: emit into the queue (app decides when/how to apply).
        if let Some(queue) = queue {
            if auto_apply_standard_actions {
                if let Some(state_model) = state_model {
                    let _ = host.update_model(state_model, |state| {
                        actions::apply_standard_action(state, "setState", &params)
                    });
                }
            }

            let inv = GenUiActionInvocation {
                window: cx.window,
                source: cx.target,
                element_key: element_key.clone(),
                event: Arc::from(event),
                action: Arc::from("setState"),
                params,
                confirm: None,
                on_success: None,
                on_error: None,
                repeat_base_path: None,
                repeat_index: None,
            };

            let _ = host.update_model(queue, |q| q.invocations.push(inv));
            host.request_redraw(cx.window);
            return;
        }

        // Fallback: apply directly if no queue is available.
        let Some(state_model) = state_model else {
            return;
        };
        let _ = host.update_model(state_model, |state| {
            actions::apply_standard_action(state, "setState", &params)
        });
        host.request_redraw(cx.window);
    }

    pub(super) fn emit_action_invocation_action(
        host: &mut dyn UiActionHost,
        cx: ActionCx,
        queue: Option<&Model<GenUiActionQueue>>,
        state_model: Option<&Model<Value>>,
        auto_apply_standard_actions: bool,
        element_key: &Arc<str>,
        event: &str,
        action: &Arc<str>,
        params: Value,
    ) {
        if let Some(queue) = queue {
            if auto_apply_standard_actions {
                if let Some(state_model) = state_model {
                    let _ = host.update_model(state_model, |state| {
                        actions::apply_standard_action(state, action.as_ref(), &params)
                    });
                }
            }

            let inv = GenUiActionInvocation {
                window: cx.window,
                source: cx.target,
                element_key: element_key.clone(),
                event: Arc::from(event),
                action: action.clone(),
                params,
                confirm: None,
                on_success: None,
                on_error: None,
                repeat_base_path: None,
                repeat_index: None,
            };
            let _ = host.update_model(queue, |q| q.invocations.push(inv));
            host.request_redraw(cx.window);
            return;
        }

        if auto_apply_standard_actions {
            if let Some(state_model) = state_model {
                let _ = host.update_model(state_model, |state| {
                    actions::apply_standard_action(state, action.as_ref(), &params)
                });
            }
        }

        host.request_redraw(cx.window);
    }

    pub(super) fn ensure_string_model<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        initial: String,
    ) -> Model<String> {
        cx.local_model_keyed(key.clone(), move || initial)
    }

    pub(super) fn ensure_optional_arc_str_model<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        initial: Option<Arc<str>>,
    ) -> Model<Option<Arc<str>>> {
        cx.local_model_keyed(key.clone(), move || initial)
    }

    pub(super) fn ensure_vec_f32_model<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        initial: Vec<f32>,
    ) -> Model<Vec<f32>> {
        cx.local_model_keyed(key.clone(), move || initial)
    }

    pub(super) fn ensure_f32_model<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        initial: f32,
    ) -> Model<f32> {
        cx.local_model_keyed(key.clone(), move || initial)
    }

    pub(super) fn ensure_bool_model<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        initial: bool,
    ) -> Model<bool> {
        cx.local_model_keyed(key.clone(), move || initial)
    }

    pub(super) fn ensure_vec_arc_str_model<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        initial: Vec<Arc<str>>,
    ) -> Model<Vec<Arc<str>>> {
        cx.local_model_keyed(key.clone(), move || initial)
    }
}
