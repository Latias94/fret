pub const SOURCE: &str = include_str!("submit_validation.rs");

// region: example
use fret::app::AppRenderActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::form::{FormRegistry, FormRegistryOptions, FormRevalidateMode};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::headless::form_state::{FormState, FormValidateMode};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let form_state = cx.local_model_keyed("submit_validation_form_state", || FormState {
        validate_mode: FormValidateMode::OnSubmit,
        ..FormState::default()
    });
    let username = cx.local_model_keyed("submit_validation_username", String::new);
    let plan = cx.local_model_keyed("submit_validation_plan", || None::<Arc<str>>);
    let submit_result = cx.local_model_keyed("submit_validation_result", || {
        Arc::<str>::from("not_submitted")
    });

    let mut registry = FormRegistry::new().options(FormRegistryOptions {
        touch_on_change: true,
        revalidate_mode: FormRevalidateMode::OnChange,
    });
    registry.register_field("username", username.clone(), String::new(), |value| {
        if value.trim().is_empty() {
            Some(Arc::from("Username is required."))
        } else {
            None
        }
    });
    registry.register_field("plan", plan.clone(), None::<Arc<str>>, |value| {
        if value.is_none() {
            Some(Arc::from("Choose a plan."))
        } else {
            None
        }
    });
    registry.register_into_form_state(&mut *cx.app, &form_state);
    registry.handle_model_changes(&mut *cx.app, &form_state, &[username.id(), plan.id()]);

    let username_field = shadcn::FormField::new(
        form_state.clone(),
        "username",
        [shadcn::Input::new(username.clone())
            .placeholder("Ada Lovelace")
            .test_id("ui-gallery-form-submit-validation-username-control")
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)],
    )
    .label("Username")
    .required(true)
    .description("Submit with an empty value to see FormState decorate the concrete input.")
    .into_element(cx)
    .test_id("ui-gallery-form-submit-validation-username-field");

    let plan_field = shadcn::FormField::new(
        form_state.clone(),
        "plan",
        [shadcn::radio_group(
            plan.clone(),
            vec![
                shadcn::RadioGroupItem::new("free", "Free"),
                shadcn::RadioGroupItem::new("pro", "Pro"),
            ],
        )
        .a11y_label("Plan")
        .test_id_prefix("ui-gallery-form-submit-validation-plan")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)],
    )
    .label("Plan")
    .required(true)
    .description("The RadioGroup root receives required/invalid semantics from FormField.")
    .into_element(cx)
    .test_id("ui-gallery-form-submit-validation-plan-field");

    let submit = {
        let registry = registry.clone();
        let form_state = form_state.clone();
        let submit_result = submit_result.clone();
        shadcn::Button::new("Submit")
            .on_activate(cx.actions().listen(move |host, action_cx| {
                let valid = registry.submit_action_host(host, &form_state);
                let status = if valid {
                    Arc::<str>::from("valid")
                } else {
                    Arc::<str>::from("invalid")
                };
                let _ = host
                    .models_mut()
                    .update(&submit_result, move |value| *value = status);
                host.request_redraw(action_cx.window);
            }))
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-form-submit-validation-submit")
    };

    let result_status = cx
        .app
        .models()
        .read(&submit_result, |status| status.to_string())
        .unwrap_or_else(|_| "not_submitted".to_string());
    let result = decl_text::text_control_readout(cx, format!("status={result_status}"))
        .test_id("ui-gallery-form-submit-validation-result");

    shadcn::Form::new([username_field, plan_field, submit, result])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
        .into_element(cx)
        .test_id("ui-gallery-form-submit-validation")
}
// endregion: example
