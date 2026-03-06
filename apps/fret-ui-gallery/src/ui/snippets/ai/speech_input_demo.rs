pub const SOURCE: &str = include_str!("speech_input_demo.rs");

// region: example
use fret_core::{SemanticsRole, TimerToken};
use fret_runtime::{Effect, Model};
use fret_ui::Invalidation;
use fret_ui::element::{SemanticsDecoration, SemanticsProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant, Card, CardContent, prelude::*};
use std::sync::Arc;
use std::time::Duration;

const TRANSCRIPT_SAMPLES: &[&str] = &[
    "Please summarize the latest build warnings.",
    "Open the deployment checklist and highlight blockers.",
    "Draft a short status update for the release channel.",
];

#[derive(Default, Clone)]
struct DemoModels {
    listening: Option<Model<bool>>,
    processing: Option<Model<bool>>,
    transcript: Option<Model<String>>,
    next_sample: Option<Model<usize>>,
    timer_token: Option<Model<Option<TimerToken>>>,
}

fn ensure_models<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
) -> (
    Model<bool>,
    Model<bool>,
    Model<String>,
    Model<usize>,
    Model<Option<TimerToken>>,
) {
    let state = cx.with_state(DemoModels::default, |st| st.clone());
    match (
        state.listening,
        state.processing,
        state.transcript,
        state.next_sample,
        state.timer_token,
    ) {
        (
            Some(listening),
            Some(processing),
            Some(transcript),
            Some(next_sample),
            Some(timer_token),
        ) => (listening, processing, transcript, next_sample, timer_token),
        _ => {
            let models = cx.app.models_mut();
            let listening = models.insert(false);
            let processing = models.insert(false);
            let transcript = models.insert(String::new());
            let next_sample = models.insert(0usize);
            let timer_token = models.insert(None::<TimerToken>);
            cx.with_state(DemoModels::default, |st| {
                st.listening = Some(listening.clone());
                st.processing = Some(processing.clone());
                st.transcript = Some(transcript.clone());
                st.next_sample = Some(next_sample.clone());
                st.timer_token = Some(timer_token.clone());
            });
            (listening, processing, transcript, next_sample, timer_token)
        }
    }
}

fn push_transcript_line(current: &mut String, sample_ix: usize) {
    let next = TRANSCRIPT_SAMPLES[sample_ix % TRANSCRIPT_SAMPLES.len()];
    if !current.is_empty() {
        current.push(' ');
    }
    current.push_str(next);
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (listening, processing, transcript, next_sample, timer_token) = ensure_models(cx);

    cx.keyed("ui-gallery.ai.speech-input.demo", move |cx| {
        let listening_for_timer = listening.clone();
        let processing_for_timer = processing.clone();
        let transcript_for_timer = transcript.clone();
        let next_sample_for_timer = next_sample.clone();
        let timer_token_for_timer = timer_token.clone();

        cx.semantics_with_id(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-ai-speech-input-demo")),
                ..Default::default()
            },
            move |cx, id| {
                cx.timer_on_timer_for(
                    id,
                    Arc::new(move |host, action_cx, token| {
                        let expected = host
                            .models_mut()
                            .read(&timer_token_for_timer, Clone::clone)
                            .ok()
                            .flatten();
                        if expected != Some(token) {
                            return false;
                        }

                        let sample_ix = host
                            .models_mut()
                            .read(&next_sample_for_timer, |value| *value)
                            .unwrap_or(0);
                        let _ = host.models_mut().update(&transcript_for_timer, |value| {
                            push_transcript_line(value, sample_ix);
                        });
                        let _ = host
                            .models_mut()
                            .update(&next_sample_for_timer, |value| *value = sample_ix + 1);
                        let _ = host
                            .models_mut()
                            .update(&processing_for_timer, |value| *value = false);
                        let _ = host
                            .models_mut()
                            .update(&listening_for_timer, |value| *value = false);
                        let _ = host
                            .models_mut()
                            .update(&timer_token_for_timer, |value| *value = None);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );

                let listening_now = cx
                    .get_model_copied(&listening, Invalidation::Paint)
                    .unwrap_or(false);
                let processing_now = cx
                    .get_model_copied(&processing, Invalidation::Paint)
                    .unwrap_or(false);
                let transcript_now = cx
                    .get_model_cloned(&transcript, Invalidation::Paint)
                    .unwrap_or_default();

                let speech_input = ui_ai::SpeechInput::new()
                    .listening_model(listening.clone())
                    .processing_model(processing.clone())
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Icon)
                    .test_id("ui-ai-speech-input-demo-btn")
                    .on_listening_change(Arc::new({
                        let listening = listening.clone();
                        let processing = processing.clone();
                        let timer_token = timer_token.clone();
                        move |host, action_cx, next| {
                            if let Some(prev) = host
                                .models_mut()
                                .read(&timer_token, Clone::clone)
                                .ok()
                                .flatten()
                            {
                                host.push_effect(Effect::CancelTimer { token: prev });
                            }

                            let _ = host.models_mut().update(&listening, |value| *value = next);

                            if next {
                                let _ = host
                                    .models_mut()
                                    .update(&processing, |value| *value = false);
                                let _ = host
                                    .models_mut()
                                    .update(&timer_token, |value| *value = None);
                            } else {
                                let token = host.next_timer_token();
                                let _ =
                                    host.models_mut().update(&processing, |value| *value = true);
                                let _ = host
                                    .models_mut()
                                    .update(&timer_token, |value| *value = Some(token));
                                host.push_effect(Effect::SetTimer {
                                    window: Some(action_cx.window),
                                    token,
                                    after: Duration::from_millis(650),
                                    repeat: None,
                                });
                            }

                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                        }
                    }))
                    .into_element(cx);

                let clear_button = (!transcript_now.is_empty()).then(|| {
                    Button::new("Clear")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .test_id("ui-ai-speech-input-demo-clear")
                        .on_activate(Arc::new({
                            let transcript = transcript.clone();
                            move |host, action_cx, _reason| {
                                let _ =
                                    host.models_mut().update(&transcript, |value| value.clear());
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            }
                        }))
                        .into_element(cx)
                });

                let mut control_row = vec![speech_input];
                if let Some(clear_button) = clear_button {
                    control_row.push(clear_button);
                }

                let transcript_surface = if transcript_now.is_empty() {
                    let message = if listening_now {
                        "Listening… click again to stop."
                    } else if processing_now {
                        "Transcribing audio…"
                    } else {
                        "Click the microphone to start speaking."
                    };
                    cx.text(message)
                        .attach_semantics(SemanticsDecoration::default().test_id(
                            if processing_now {
                                "ui-ai-speech-input-demo-processing"
                            } else {
                                "ui-ai-speech-input-demo-hint"
                            },
                        ))
                } else {
                    let content = CardContent::new([
                        cx.text("Transcript:"),
                        cx.text(transcript_now.clone()).attach_semantics(
                            SemanticsDecoration::default()
                                .test_id("ui-ai-speech-input-demo-transcript"),
                        ),
                    ])
                    .into_element(cx);

                    Card::new([content])
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_full()
                                .min_w_0()
                                .max_w(fret_core::Px(460.0)),
                        )
                        .into_element(cx)
                };

                vec![
                    ui::v_flex(move |cx| {
                        let row = ui::h_flex(move |_cx| control_row)
                            .gap(Space::N2)
                            .items_center()
                            .justify_center()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .into_element(cx);

                        vec![row, transcript_surface]
                    })
                    .gap(Space::N4)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                ]
            },
        )
    })
}
// endregion: example
