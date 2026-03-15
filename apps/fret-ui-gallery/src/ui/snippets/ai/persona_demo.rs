pub const SOURCE: &str = include_str!("persona_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn state_button(
    cx: &mut UiCx<'_>,
    state_model: Model<ui_ai::PersonaState>,
    current_state: ui_ai::PersonaState,
    state: ui_ai::PersonaState,
    icon: &'static str,
    label: &'static str,
) -> shadcn::Button {
    shadcn::Button::new("")
        .children([decl_icon::icon(cx, IconId::new_static(icon))])
        .size(shadcn::ButtonSize::IconSm)
        .variant(if current_state == state {
            shadcn::ButtonVariant::Default
        } else {
            shadcn::ButtonVariant::Outline
        })
        .a11y_label(label)
        .test_id(format!("ui-ai-persona-demo-state-{}", state.as_str()))
        .on_activate(Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&state_model, |value| *value = state);
            host.notify(action_cx);
        }))
}

fn variant_button(
    variant_model: Model<ui_ai::PersonaVariant>,
    current_variant: ui_ai::PersonaVariant,
    variant: ui_ai::PersonaVariant,
) -> shadcn::Button {
    shadcn::Button::new(variant.label())
        .size(shadcn::ButtonSize::Sm)
        .variant(if current_variant == variant {
            shadcn::ButtonVariant::Default
        } else {
            shadcn::ButtonVariant::Outline
        })
        .test_id(format!("ui-ai-persona-demo-variant-{}", variant.as_str()))
        .on_activate(Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&variant_model, |value| *value = variant);
            host.notify(action_cx);
        }))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let state_model = cx.local_model_keyed("state", || ui_ai::PersonaState::Idle);
    let variant_model = cx.local_model_keyed("variant", || ui_ai::PersonaVariant::Obsidian);

    let current_state = cx
        .get_model_copied(&state_model, Invalidation::Layout)
        .unwrap_or(ui_ai::PersonaState::Idle);
    let current_variant = cx
        .get_model_copied(&variant_model, Invalidation::Layout)
        .unwrap_or(ui_ai::PersonaVariant::Obsidian);

    let state_controls = shadcn::ButtonGroup::new([
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
            "lucide.volume-2",
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
    .into_element(cx);

    let variant_controls = ui::h_flex(move |cx| {
        vec![
            variant_button(
                variant_model.clone(),
                current_variant,
                ui_ai::PersonaVariant::Obsidian,
            )
            .into_element(cx),
            variant_button(
                variant_model.clone(),
                current_variant,
                ui_ai::PersonaVariant::Mana,
            )
            .into_element(cx),
            variant_button(
                variant_model.clone(),
                current_variant,
                ui_ai::PersonaVariant::Opal,
            )
            .into_element(cx),
            variant_button(
                variant_model.clone(),
                current_variant,
                ui_ai::PersonaVariant::Halo,
            )
            .into_element(cx),
            variant_button(
                variant_model.clone(),
                current_variant,
                ui_ai::PersonaVariant::Glint,
            )
            .into_element(cx),
            variant_button(
                variant_model,
                current_variant,
                ui_ai::PersonaVariant::Command,
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .wrap()
    .w_full()
    .items_center()
    .justify_center()
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
            [preview, state_controls, variant_controls]
        })
        .test_id("ui-ai-persona-demo-shell");

    ui::v_flex(move |_cx| vec![shell])
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
// endregion: example
