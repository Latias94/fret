pub const SOURCE: &str = include_str!("persona_state_management.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::facade as shadcn;

fn state_button(
    cx: &mut UiCx<'_>,
    state_model: Model<ui_ai::PersonaState>,
    current_state: ui_ai::PersonaState,
    next_state: ui_ai::PersonaState,
    icon: &'static str,
    label: &'static str,
) -> shadcn::Button {
    shadcn::Button::new("")
        .children([decl_icon::icon(cx, IconId::new_static(icon))])
        .size(shadcn::ButtonSize::IconSm)
        .variant(if current_state == next_state {
            shadcn::ButtonVariant::Default
        } else {
            shadcn::ButtonVariant::Outline
        })
        .a11y_label(label)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            let _ = host
                .models_mut()
                .update(&state_model, |value| *value = next_state);
            host.notify(action_cx);
        }))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let state_model = cx.local_model_keyed("state", || ui_ai::PersonaState::Idle);

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
            shadcn::ButtonGroup::new([
                state_button(
                    cx,
                    state_model.clone(),
                    current_state,
                    ui_ai::PersonaState::Idle,
                    "lucide.circle",
                    "Idle",
                )
                .into(),
                state_button(
                    cx,
                    state_model.clone(),
                    current_state,
                    ui_ai::PersonaState::Listening,
                    "lucide.mic",
                    "Listening",
                )
                .into(),
                state_button(
                    cx,
                    state_model.clone(),
                    current_state,
                    ui_ai::PersonaState::Thinking,
                    "lucide.brain",
                    "Thinking",
                )
                .into(),
                state_button(
                    cx,
                    state_model.clone(),
                    current_state,
                    ui_ai::PersonaState::Speaking,
                    "lucide.megaphone",
                    "Speaking",
                )
                .into(),
                state_button(
                    cx,
                    state_model,
                    current_state,
                    ui_ai::PersonaState::Asleep,
                    "lucide.eye-closed",
                    "Asleep",
                )
                .into(),
            ])
            .a11y_label("Persona state")
            .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
