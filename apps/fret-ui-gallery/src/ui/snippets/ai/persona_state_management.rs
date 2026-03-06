pub const SOURCE: &str = include_str!("persona_state_management.rs");

// region: example
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{self as shadcn, ButtonSize, ButtonVariant, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    state: Option<Model<ui_ai::PersonaState>>,
}

fn state_button(
    state_model: Model<ui_ai::PersonaState>,
    current_state: ui_ai::PersonaState,
    next_state: ui_ai::PersonaState,
    label: &'static str,
) -> shadcn::Button {
    shadcn::Button::new(label)
        .size(ButtonSize::Sm)
        .variant(if current_state == next_state {
            ButtonVariant::Default
        } else {
            ButtonVariant::Outline
        })
        .on_activate(Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&state_model, |value| *value = next_state);
            host.notify(action_cx);
        }))
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state_model = cx.with_state(DemoModels::default, |st| st.state.clone());
    let state_model = match state_model {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(ui_ai::PersonaState::Idle);
            cx.with_state(DemoModels::default, |st| st.state = Some(model.clone()));
            model
        }
    };

    let current_state = cx
        .get_model_copied(&state_model, Invalidation::Layout)
        .unwrap_or(ui_ai::PersonaState::Idle);

    ui::v_flex(move |cx| {
        vec![
            ui_ai::Persona::new(current_state)
                .variant(ui_ai::PersonaVariant::Opal)
                .size(Px(128.0))
                .test_id("ui-ai-persona-state-management-current")
                .into_element(cx),
            ui::h_flex(move |cx| {
                vec![
                    state_button(
                        state_model.clone(),
                        current_state,
                        ui_ai::PersonaState::Listening,
                        "Listen",
                    )
                    .into_element(cx),
                    state_button(
                        state_model.clone(),
                        current_state,
                        ui_ai::PersonaState::Thinking,
                        "Think",
                    )
                    .into_element(cx),
                    state_button(
                        state_model.clone(),
                        current_state,
                        ui_ai::PersonaState::Speaking,
                        "Speak",
                    )
                    .into_element(cx),
                    state_button(
                        state_model.clone(),
                        current_state,
                        ui_ai::PersonaState::Asleep,
                        "Sleep",
                    )
                    .into_element(cx),
                    state_button(
                        state_model,
                        current_state,
                        ui_ai::PersonaState::Idle,
                        "Reset",
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .wrap()
            .justify_center()
            .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
