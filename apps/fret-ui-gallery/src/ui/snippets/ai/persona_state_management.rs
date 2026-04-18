pub const SOURCE: &str = include_str!("persona_state_management.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::model_watch::TrackedModelExt as _;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn state_from_value(value: Option<&str>) -> ui_ai::PersonaState {
    match value {
        Some(value) if value == ui_ai::PersonaState::Listening.as_str() => {
            ui_ai::PersonaState::Listening
        }
        Some(value) if value == ui_ai::PersonaState::Thinking.as_str() => {
            ui_ai::PersonaState::Thinking
        }
        Some(value) if value == ui_ai::PersonaState::Speaking.as_str() => {
            ui_ai::PersonaState::Speaking
        }
        Some(value) if value == ui_ai::PersonaState::Asleep.as_str() => ui_ai::PersonaState::Asleep,
        _ => ui_ai::PersonaState::Idle,
    }
}

fn state_item(
    next_state: ui_ai::PersonaState,
    icon: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::icon(next_state.as_str(), IconId::new_static(icon))
        .a11y_label(label)
        .test_id(format!(
            "ui-ai-persona-state-management-state-{}",
            next_state.as_str()
        ))
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let state_model = cx.local_model_keyed("state", || {
        Some(Arc::<str>::from(ui_ai::PersonaState::Idle.as_str()))
    });

    let current_state = state_from_value(
        state_model
            .layout_in(cx)
            .value_or(Some(Arc::<str>::from(ui_ai::PersonaState::Idle.as_str())))
            .as_deref(),
    );

    ui::v_flex(move |cx| {
        vec![
            ui_ai::Persona::new(current_state)
                .variant(ui_ai::PersonaVariant::Opal)
                .size(Px(128.0))
                .test_id("ui-ai-persona-state-management-current")
                .into_element(cx),
            shadcn::ToggleGroup::single(&state_model)
                .deselectable(false)
                .size(shadcn::ToggleSize::Sm)
                .items([
                    state_item(ui_ai::PersonaState::Idle, "lucide.circle", "Idle"),
                    state_item(ui_ai::PersonaState::Listening, "lucide.mic", "Listening"),
                    state_item(ui_ai::PersonaState::Thinking, "lucide.brain", "Thinking"),
                    state_item(
                        ui_ai::PersonaState::Speaking,
                        "lucide.megaphone",
                        "Speaking",
                    ),
                    state_item(ui_ai::PersonaState::Asleep, "lucide.eye-closed", "Asleep"),
                ])
                .refine_layout(LayoutRefinement::default().flex_none())
                .into_element(cx)
                .test_id("ui-ai-persona-state-management-state-group"),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
