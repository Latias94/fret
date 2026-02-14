use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_voice_selector_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui::element::SemanticsDecoration;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct DemoModels {
        open: Option<Model<bool>>,
        value: Option<Model<Option<Arc<str>>>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.open.is_none() || st.value.is_none()
    });
    if needs_init {
        let open = cx.app.models_mut().insert(false);
        let value = cx.app.models_mut().insert(None::<Arc<str>>);
        cx.with_state(DemoModels::default, move |st| {
            st.open = Some(open.clone());
            st.value = Some(value.clone());
        });
    }

    let (open, value) = cx.with_state(DemoModels::default, |st| {
        (
            st.open.clone().expect("open"),
            st.value.clone().expect("value"),
        )
    });

    let voices: Arc<[ui_ai::VoiceSelectorVoice]> = Arc::from(vec![
        ui_ai::VoiceSelectorVoice::new("alloy", "Alloy").description("Balanced, clear tone"),
        ui_ai::VoiceSelectorVoice::new("verse", "Verse").description("Warm, conversational"),
        ui_ai::VoiceSelectorVoice::new("orbit", "Orbit").description("Crisp, energetic"),
    ]);

    let selected = cx.app.models().read(&value, |v| v.clone()).ok().flatten();

    let marker = cx
        .text(format!(
            "selected={}",
            selected.as_deref().unwrap_or("<none>")
        ))
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Generic)
                .test_id(if selected.is_some() {
                    "ui-ai-voice-selector-demo-selected"
                } else {
                    "ui-ai-voice-selector-demo-none"
                }),
        );

    let selector = ui_ai::VoiceSelector::from_arc(voices)
        .open_model(open.clone())
        .value_model(value.clone())
        .into_element(
            cx,
            |cx| {
                ui_ai::VoiceSelectorButton::new()
                    .test_id("ui-ai-voice-selector-demo-trigger")
                    .into_element(cx)
            },
            |cx| {
                ui_ai::VoiceSelectorContent::new([
                    ui_ai::VoiceSelectorInput::new()
                        .test_id("ui-ai-voice-selector-demo-input")
                        .into_element(cx),
                    ui_ai::VoiceSelectorList::new()
                        .test_id_prefix("ui-ai-voice-selector-demo-item")
                        .into_element(cx),
                ])
                .test_id_root("ui-ai-voice-selector-demo-content")
                .into_element(cx)
            },
        );

    let open_now = cx
        .get_model_copied(&open, Invalidation::Paint)
        .unwrap_or(false);
    let open_marker = cx.text(format!("open={open_now}")).attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Generic)
            .test_id(if open_now {
                "ui-ai-voice-selector-demo-open-true"
            } else {
                "ui-ai-voice-selector-demo-open-false"
            }),
    );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("VoiceSelector (AI Elements)"),
                cx.text("UI-only chrome. Apps own voice inventory + preview playback."),
                marker,
                open_marker,
                selector,
            ]
        },
    )]
}
