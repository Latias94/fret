pub const SOURCE: &str = include_str!("persona_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::model_watch::TrackedModelExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
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

fn variant_from_value(value: Option<&str>) -> ui_ai::PersonaVariant {
    match value {
        Some(value) if value == ui_ai::PersonaVariant::Mana.as_str() => ui_ai::PersonaVariant::Mana,
        Some(value) if value == ui_ai::PersonaVariant::Opal.as_str() => ui_ai::PersonaVariant::Opal,
        Some(value) if value == ui_ai::PersonaVariant::Halo.as_str() => ui_ai::PersonaVariant::Halo,
        Some(value) if value == ui_ai::PersonaVariant::Glint.as_str() => {
            ui_ai::PersonaVariant::Glint
        }
        Some(value) if value == ui_ai::PersonaVariant::Command.as_str() => {
            ui_ai::PersonaVariant::Command
        }
        _ => ui_ai::PersonaVariant::Obsidian,
    }
}

fn state_item(
    state: ui_ai::PersonaState,
    icon: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::icon(state.as_str(), IconId::new_static(icon))
        .a11y_label(label)
        .test_id(format!("ui-ai-persona-demo-state-{}", state.as_str()))
}

fn variant_item(cx: &mut UiCx<'_>, variant: ui_ai::PersonaVariant) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(variant.as_str(), [cx.text(variant.label())])
        .a11y_label(variant.label())
        .test_id(format!("ui-ai-persona-demo-variant-{}", variant.as_str()))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let state_model = cx.local_model_keyed("state", || {
        Some(Arc::<str>::from(ui_ai::PersonaState::Idle.as_str()))
    });
    let variant_model = cx.local_model_keyed("variant", || {
        Some(Arc::<str>::from(ui_ai::PersonaVariant::Obsidian.as_str()))
    });

    let current_state = state_from_value(
        state_model
            .layout_in(cx)
            .value_or(Some(Arc::<str>::from(ui_ai::PersonaState::Idle.as_str())))
            .as_deref(),
    );
    let current_variant = variant_from_value(
        variant_model
            .layout_in(cx)
            .value_or(Some(Arc::<str>::from(
                ui_ai::PersonaVariant::Obsidian.as_str(),
            )))
            .as_deref(),
    );

    let state_controls = shadcn::ToggleGroup::single(&state_model)
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
        .test_id("ui-ai-persona-demo-state-group");

    let variant_controls = shadcn::ToggleGroup::single(&variant_model)
        .deselectable(false)
        .variant(shadcn::ToggleVariant::Outline)
        .size(shadcn::ToggleSize::Sm)
        .spacing(Space::N2)
        .items([
            variant_item(cx, ui_ai::PersonaVariant::Obsidian),
            variant_item(cx, ui_ai::PersonaVariant::Mana),
            variant_item(cx, ui_ai::PersonaVariant::Opal),
            variant_item(cx, ui_ai::PersonaVariant::Halo),
            variant_item(cx, ui_ai::PersonaVariant::Glint),
            variant_item(cx, ui_ai::PersonaVariant::Command),
        ])
        .refine_layout(LayoutRefinement::default().flex_none().min_w_0())
        .into_element(cx)
        .test_id("ui-ai-persona-demo-variant-group");

    let state_controls = ui::h_row(move |_cx| [state_controls])
        .justify_center()
        .w_full()
        .into_element(cx);

    let variant_controls = ui::h_row(move |_cx| [variant_controls])
        .justify_center()
        .w_full()
        .into_element(cx);

    let reset_state = state_model.clone();
    let reset_variant = variant_model.clone();
    let reset = shadcn::Button::new("Reset")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            let _ = host.models_mut().update(&reset_state, |value| {
                *value = Some(Arc::<str>::from(ui_ai::PersonaState::Idle.as_str()));
            });
            let _ = host.models_mut().update(&reset_variant, |value| {
                *value = Some(Arc::<str>::from(ui_ai::PersonaVariant::Obsidian.as_str()));
            });
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }))
        .test_id("ui-ai-persona-demo-reset")
        .into_element(cx);

    let reset = ui::h_row(move |_cx| [reset])
        .justify_center()
        .w_full()
        .into_element(cx);

    let preview = ui::v_flex(move |cx| {
        vec![
            ui_ai::Persona::new(current_state)
                .variant(current_variant)
                .size(Px(128.0))
                .test_id("ui-ai-persona-demo-current")
                .into_element(cx),
            cx.text(format!(
                "{} / {}",
                current_variant.label(),
                current_state.label()
            ))
            .test_id("ui-ai-persona-demo-current-label"),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .justify_center()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .h_px(Px(240.0))
            .min_w_0(),
    )
    .into_element(cx);

    let shell_props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .p(Space::N6),
            LayoutRefinement::default()
                .w_full()
                .max_w(MetricRef::Px(Px(760.0)))
                .min_w_0(),
        )
    });

    let shell = cx
        .container(shell_props, move |_cx| {
            [preview, state_controls, variant_controls, reset]
        })
        .test_id("ui-ai-persona-demo-shell");

    ui::v_flex(move |_cx| vec![shell])
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
// endregion: example
