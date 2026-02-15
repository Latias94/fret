use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_reasoning_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};
    use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

    #[derive(Default)]
    struct DemoModels {
        streaming: Option<Model<bool>>,
    }

    let streaming = cx.with_state(DemoModels::default, |st| st.streaming.clone());
    let streaming = match streaming {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| st.streaming = Some(model.clone()));
            model
        }
    };

    let is_streaming = cx
        .get_model_copied(&streaming, Invalidation::Layout)
        .unwrap_or(false);

    let start = Button::new("Start streaming")
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .test_id("ui-ai-reasoning-start-streaming")
        .on_activate(Arc::new({
            let streaming = streaming.clone();
            move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&streaming, |v| *v = true);
                host.notify(action_cx);
            }
        }))
        .into_element(cx);

    let stop = Button::new("Stop streaming")
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .test_id("ui-ai-reasoning-stop-streaming")
        .on_activate(Arc::new({
            let streaming = streaming.clone();
            move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&streaming, |v| *v = false);
                host.notify(action_cx);
            }
        }))
        .into_element(cx);

    let reasoning = ui_ai::Reasoning::new(is_streaming)
        .test_id_root("ui-ai-reasoning-root")
        .into_element(
            cx,
            |cx| ui_ai::ReasoningTrigger::new().into_element(cx),
            |cx| {
                ui_ai::ReasoningContent::new(
                    "Reasoning content is markdown.\n\n- Opens automatically when streaming starts.\n- Auto-closes shortly after streaming ends.",
                )
                .test_id("ui-ai-reasoning-content")
                .into_element(cx)
            },
        );

    let controls = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |_cx| vec![start, stop],
    );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Reasoning (AI Elements)"),
                cx.text("Start streaming to auto-open; stop to auto-close."),
                controls,
                reasoning,
            ]
        },
    )]
}
