pub const SOURCE: &str = include_str!("speech_input_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{
    AttributedText, Color, DecorationLineStyle, FontWeight, Px, SemanticsRole, TextAlign,
    TextOverflow, TextPaintStyle, TextSpan, TextStyle, TextWrap, TimerToken, UnderlineStyle,
};
use fret_runtime::{Effect, Model};
use fret_ui::element::{
    LayoutStyle, Length, PressableA11y, PressableProps, SemanticsDecoration, SemanticsProps,
    StyledTextProps, TextInkOverflow, TextProps,
};
use fret_ui::{Invalidation, Theme};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

const TRANSCRIPT_SAMPLES: &[&str] = &[
    "Please summarize the latest build warnings.",
    "Open the deployment checklist and highlight blockers.",
    "Draft a short status update for the release channel.",
];

fn speech_models(
    cx: &mut UiCx<'_>,
) -> (
    Model<bool>,
    Model<bool>,
    Model<String>,
    Model<usize>,
    Model<Option<TimerToken>>,
) {
    (
        cx.local_model_keyed("listening", || false),
        cx.local_model_keyed("processing", || false),
        cx.local_model_keyed("transcript", String::new),
        cx.local_model_keyed("next_sample", || 0usize),
        cx.local_model_keyed("timer_token", || None::<TimerToken>),
    )
}

fn push_transcript_line(current: &mut String, sample_ix: usize) {
    let next = TRANSCRIPT_SAMPLES[sample_ix % TRANSCRIPT_SAMPLES.len()];
    if !current.is_empty() {
        current.push(' ');
    }
    current.push_str(next);
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn text_sm(theme: &Theme, weight: FontWeight) -> TextStyle {
    let mut style =
        typography::TypographyPreset::control_ui(typography::UiTextSize::Sm).resolve(theme);
    style.weight = weight;
    style
}

fn fill_text_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout
}

fn body_text(
    cx: &mut UiCx<'_>,
    text: impl Into<Arc<str>>,
    style: TextStyle,
    color: Color,
    align: TextAlign,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: fill_text_layout(),
        text: text.into(),
        style: Some(style),
        color: Some(color),
        wrap: TextWrap::WordBreak,
        overflow: TextOverflow::Clip,
        align,
        ink_overflow: TextInkOverflow::None,
    })
}

fn underlined_text(text: Arc<str>) -> AttributedText {
    let spans: Arc<[TextSpan]> = Arc::from([TextSpan {
        len: text.len(),
        shaping: Default::default(),
        paint: TextPaintStyle {
            underline: Some(UnderlineStyle {
                color: None,
                style: DecorationLineStyle::Solid,
            }),
            ..Default::default()
        },
    }]);
    AttributedText::new(text, spans)
}

fn clear_action(cx: &mut UiCx<'_>, transcript: Model<String>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app);
    let muted = muted_fg(theme);
    let foreground = theme.color_token("foreground");
    let text_style = text_sm(theme, FontWeight::NORMAL);
    let focus_ring = decl_style::focus_ring(
        theme,
        theme.metric_by_key("metric.radius.sm").unwrap_or(Px(4.0)),
    );
    let label = Arc::<str>::from("Clear");

    cx.pressable(
        PressableProps {
            focus_ring: Some(focus_ring),
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::clone(&label)),
                test_id: Some(Arc::from("ui-ai-speech-input-demo-clear")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, state| {
            let transcript = transcript.clone();
            cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&transcript, |value| value.clear());
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));

            let color = if state.hovered || state.pressed {
                foreground
            } else {
                muted
            };

            vec![cx.styled_text_props(StyledTextProps {
                layout: LayoutStyle::default(),
                rich: underlined_text(Arc::clone(&label)),
                style: Some(text_style.clone()),
                color: Some(color),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: TextAlign::Start,
                ink_overflow: TextInkOverflow::None,
            })]
        },
    )
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let (listening, processing, transcript, next_sample, timer_token) = speech_models(cx);

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

                let theme = Theme::global(&*cx.app);
                let muted = muted_fg(theme);
                let foreground = theme.color_token("foreground");
                let muted_text_style = text_sm(theme, FontWeight::NORMAL);
                let transcript_label_style = text_sm(theme, FontWeight::SEMIBOLD);
                let transcript_body_style = text_sm(theme, FontWeight::NORMAL);

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
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Icon)
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

                let clear_button = (!transcript_now.is_empty())
                    .then(|| clear_action(cx, transcript.clone()).into_element(cx));

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
                        "Click the microphone to start speaking"
                    };
                    body_text(cx, message, muted_text_style, muted, TextAlign::Center)
                        .into_element(cx)
                        .attach_semantics(SemanticsDecoration::default().test_id(
                            if processing_now {
                                "ui-ai-speech-input-demo-processing"
                            } else {
                                "ui-ai-speech-input-demo-hint"
                            },
                        ))
                } else {
                    let content = ui::v_flex(move |cx| {
                        vec![
                            body_text(
                                cx,
                                "Transcript:",
                                transcript_label_style,
                                muted,
                                TextAlign::Start,
                            )
                            .into_element(cx),
                            body_text(
                                cx,
                                transcript_now.clone(),
                                transcript_body_style,
                                foreground,
                                TextAlign::Start,
                            )
                            .into_element(cx)
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .test_id("ui-ai-speech-input-demo-transcript"),
                            ),
                        ]
                    })
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx);

                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_content(|cx| ui::children![cx; content]),
                        ]
                    })
                    .size(shadcn::CardSize::Sm)
                    .refine_style(ChromeRefinement::default().shadow_none())
                    .refine_layout(
                        LayoutRefinement::default()
                            .w_full()
                            .min_w_0()
                            .max_w(Px(448.0)),
                    )
                    .into_element(cx)
                };

                vec![
                    ui::v_flex(move |cx| {
                        let row = ui::h_flex(move |_cx| control_row)
                            .gap(Space::N2)
                            .items_center()
                            .justify_center()
                            .layout(LayoutRefinement::default().min_w_0())
                            .into_element(cx);

                        vec![row, transcript_surface]
                    })
                    .gap(Space::N4)
                    .items_center()
                    .justify_center()
                    .layout(
                        LayoutRefinement::default()
                            .w_full()
                            .min_w_0()
                            .min_h(Px(280.0)),
                    )
                    .into_element(cx),
                ]
            },
        )
    })
}
// endregion: example
