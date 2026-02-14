use std::sync::Arc;

use fret_genui_core::actions;
use fret_genui_core::props::ResolvedProps;
use fret_genui_core::render::{
    ComponentResolver, GenUiActionInvocation, GenUiActionQueue, GenUiRenderScope,
};
use fret_genui_core::spec::{ElementKey, ElementV1};
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum ShadcnResolverError {
    #[error("invalid props for component: {component}")]
    InvalidProps { component: String },
}

#[derive(Clone, Default)]
pub struct ShadcnResolver;

impl ShadcnResolver {
    pub fn new() -> Self {
        Self
    }

    fn text_element<H: UiHost>(cx: &mut ElementContext<'_, H>, text: Arc<str>) -> AnyElement {
        fret_ui_kit::ui::text(cx, text).into_element(cx)
    }

    fn json_to_label(v: Option<&serde_json::Value>) -> Arc<str> {
        let Some(v) = v else {
            return Arc::<str>::from("");
        };
        if let Some(s) = v.as_str() {
            return Arc::<str>::from(s);
        }
        Arc::<str>::from(v.to_string())
    }

    fn unknown_component<H: UiHost>(
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

    fn parse_space(v: Option<&serde_json::Value>) -> Option<fret_ui_kit::Space> {
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

    fn parse_badge_variant(v: Option<&serde_json::Value>) -> Option<fret_ui_shadcn::BadgeVariant> {
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

    fn genui_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<GenUiRenderScope> {
        cx.inherited_state::<GenUiRenderScope>().cloned()
    }

    fn emit_set_state<H: UiHost>(
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
            let inv = GenUiActionInvocation {
                window: cx.window,
                source: element_id,
                element_key: Arc::from(element_key.0.as_str()),
                event: Arc::from(event),
                action: Arc::from("setState"),
                params,
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

    fn ensure_string_model<H: UiHost>(
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

    fn ensure_bool_model<H: UiHost>(cx: &mut ElementContext<'_, H>, initial: bool) -> Model<bool> {
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

impl<H: UiHost> ComponentResolver<H> for ShadcnResolver {
    type Error = ShadcnResolverError;

    fn render_element(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        element: &ElementV1,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
        on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> Result<AnyElement, ShadcnResolverError> {
        let resolved_props = &props.props;
        match element.ty.as_str() {
            "Card" => Ok(fret_ui_shadcn::Card::new([
                fret_ui_shadcn::CardContent::new(children).into_element(cx)
            ])
            .into_element(cx)),
            "Text" => {
                let text = Self::json_to_label(resolved_props.get("text"));
                Ok(fret_ui_kit::ui::text(cx, text).into_element(cx))
            }
            "VStack" => {
                let gap =
                    Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
                Ok(fret_ui_kit::ui::v_flex(cx, move |_cx| children)
                    .gap(gap)
                    .items_start()
                    .into_element(cx))
            }
            "HStack" => {
                let gap =
                    Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
                Ok(fret_ui_kit::ui::h_flex(cx, move |_cx| children)
                    .gap(gap)
                    .items_center()
                    .into_element(cx))
            }
            "Button" => {
                let label = Self::json_to_label(resolved_props.get("label"));
                let mut button = fret_ui_shadcn::Button::new(label).children(children);
                if let Some(on_activate) = on_event("press") {
                    button = button.on_activate(on_activate);
                }
                Ok(button.into_element(cx))
            }
            "Input" => {
                let placeholder = resolved_props
                    .get("placeholder")
                    .and_then(|v| v.as_str())
                    .map(Arc::<str>::from);

                let desired = Self::json_to_label(resolved_props.get("value")).to_string();

                let model = Self::ensure_string_model(cx, desired.clone());

                let cur = cx.app.models().get_cloned(&model).unwrap_or_default();

                if let (Some(scope), Some(path)) =
                    (Self::genui_scope(cx), props.bindings.get("value"))
                {
                    #[derive(Default)]
                    struct LastState {
                        last_model: Option<String>,
                        last_desired: Option<String>,
                    }

                    let mut to_emit: Option<String> = None;
                    let mut sync_model_to: Option<String> = None;
                    cx.with_state(LastState::default, |st| {
                        let model_changed =
                            st.last_model.as_deref().is_some_and(|v| v != cur.as_str());
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
                        Self::emit_set_state(
                            cx,
                            &scope,
                            key,
                            "change",
                            path.as_str(),
                            Value::String(v),
                        );
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
                    Ok(input)
                } else {
                    Ok(fret_ui_kit::ui::v_flex(cx, move |_cx| {
                        let mut out = Vec::with_capacity(children.len().saturating_add(1));
                        out.push(input);
                        out.extend(children);
                        out
                    })
                    .gap(fret_ui_kit::Space::N2)
                    .items_start()
                    .into_element(cx))
                }
            }
            "Switch" => {
                let desired = resolved_props
                    .get("checked")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let model = Self::ensure_bool_model(cx, desired);

                if let (Some(scope), Some(path)) =
                    (Self::genui_scope(cx), props.bindings.get("checked"))
                {
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
                        Self::emit_set_state(
                            cx,
                            &scope,
                            key,
                            "change",
                            path.as_str(),
                            Value::Bool(cur),
                        );
                    }
                }

                let sw = fret_ui_shadcn::Switch::new(model).into_element(cx);
                if children.is_empty() {
                    Ok(sw)
                } else {
                    Ok(fret_ui_kit::ui::v_flex(cx, move |_cx| {
                        let mut out = Vec::with_capacity(children.len().saturating_add(1));
                        out.push(sw);
                        out.extend(children);
                        out
                    })
                    .gap(fret_ui_kit::Space::N2)
                    .items_start()
                    .into_element(cx))
                }
            }
            "Badge" => {
                let label = Self::json_to_label(resolved_props.get("label"));
                let variant =
                    Self::parse_badge_variant(resolved_props.get("variant")).unwrap_or_default();
                Ok(fret_ui_shadcn::Badge::new(label)
                    .variant(variant)
                    .children(children)
                    .into_element(cx))
            }
            other => Ok(self.unknown_component(cx, key, other)),
        }
    }
}
