use std::sync::Arc;

use fret_genui_core::actions;
use fret_genui_core::render::{GenUiActionInvocation, GenUiActionQueue, GenUiRenderScope};
use fret_genui_core::spec::ElementKey;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::Value;

use super::ShadcnResolver;

impl ShadcnResolver {
    pub(super) fn text_element<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        text: Arc<str>,
    ) -> AnyElement {
        fret_ui_kit::ui::text(cx, text).into_element(cx)
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

    pub(super) fn ensure_string_model<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        initial: String,
    ) -> Model<String> {
        #[derive(Default)]
        struct ModelState {
            model: Option<Model<String>>,
        }
        let existing = cx.with_state(ModelState::default, |st| st.model.clone());
        if let Some(model) = existing {
            return model;
        }
        let model = cx.app.models_mut().insert(initial);
        cx.with_state(ModelState::default, |st| st.model = Some(model.clone()));
        model
    }

    pub(super) fn ensure_bool_model<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        initial: bool,
    ) -> Model<bool> {
        #[derive(Default)]
        struct ModelState {
            model: Option<Model<bool>>,
        }
        let existing = cx.with_state(ModelState::default, |st| st.model.clone());
        if let Some(model) = existing {
            return model;
        }
        let model = cx.app.models_mut().insert(initial);
        cx.with_state(ModelState::default, |st| st.model = Some(model.clone()));
        model
    }
}
