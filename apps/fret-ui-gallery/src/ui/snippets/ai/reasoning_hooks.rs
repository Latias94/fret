pub const SOURCE: &str = include_str!("reasoning_hooks.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{Items, LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("reasoning_hooks.open", || true);

    ui_ai::Reasoning::new(false)
        .open(open)
        .duration_secs(Some(4))
        .test_id_root("ui-ai-reasoning-hooks-root")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(
            cx,
            |cx| {
                let Some(reasoning) = ui_ai::use_reasoning_controller(cx) else {
                    return cx.text("Reasoning controller unavailable");
                };

                let open_model = reasoning.open.clone();
                let button_label = if reasoning.is_open {
                    "Hide reasoning"
                } else {
                    "Show reasoning"
                };
                let status = if reasoning.is_streaming {
                    "Thinking...".to_string()
                } else if let Some(duration_secs) = reasoning.duration_secs {
                    format!("Thought for {duration_secs} seconds")
                } else {
                    "Reasoning complete".to_string()
                };

                let toggle = shadcn::Button::new(button_label)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-ai-reasoning-hooks-trigger")
                    .on_activate(cx.actions().listen(move |host, action_cx| {
                        let _ = host.models_mut().update(&open_model, |value| *value = !*value);
                        host.notify(action_cx);
                    }))
                    .into_element(cx);

                ui::h_row(move |cx| vec![toggle, cx.text(status)])
                    .items(Items::Center)
                    .gap(Space::N2)
                    .into_element(cx)
            },
            |cx| {
                ui_ai::ReasoningContent::new(
                    "Custom trigger logic can read `use_reasoning_controller(cx)` and still reuse the stock content part.",
                )
                .test_id("ui-ai-reasoning-hooks-content")
                .into_element(cx)
            },
        )
}
// endregion: example
