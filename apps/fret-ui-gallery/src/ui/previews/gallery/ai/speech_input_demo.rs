use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_speech_input_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui::element::SemanticsDecoration;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct DemoModels {
        listening: Option<Model<bool>>,
        processing: Option<Model<bool>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.listening.is_none() || st.processing.is_none()
    });
    if needs_init {
        let listening = cx.app.models_mut().insert(false);
        let processing = cx.app.models_mut().insert(false);
        cx.with_state(DemoModels::default, move |st| {
            st.listening = Some(listening.clone());
            st.processing = Some(processing.clone());
        });
    }

    let (listening, processing) = cx.with_state(DemoModels::default, |st| {
        (
            st.listening.clone().expect("listening"),
            st.processing.clone().expect("processing"),
        )
    });

    let listening_now = cx
        .get_model_copied(&listening, Invalidation::Paint)
        .unwrap_or(false);

    let marker = cx
        .text(format!("listening={listening_now}"))
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Generic)
                .test_id(if listening_now {
                    "ui-ai-speech-input-demo-listening-true"
                } else {
                    "ui-ai-speech-input-demo-listening-false"
                }),
        );

    let btn = ui_ai::SpeechInput::new()
        .listening_model(listening.clone())
        .processing_model(processing.clone())
        .test_id("ui-ai-speech-input-demo-btn")
        .into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("SpeechInput (AI Elements)"),
                cx.text("UI-only chrome. Apps own capture + ASR backends."),
                marker,
                btn,
            ]
        },
    )]
}
