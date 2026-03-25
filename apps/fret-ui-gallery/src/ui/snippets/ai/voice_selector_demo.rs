pub const SOURCE: &str = include_str!("voice_selector_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui::element::SemanticsDecoration;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{Items, LayoutRefinement, Space};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

#[derive(Clone, Copy)]
struct DemoVoice {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    gender: &'static str,
    accent: &'static str,
    age: &'static str,
}

const DEMO_VOICES: &[DemoVoice] = &[
    DemoVoice {
        id: "liam",
        name: "Liam",
        description: "Energetic, Social Media Creator",
        gender: "male",
        accent: "american",
        age: "20-30",
    },
    DemoVoice {
        id: "adam",
        name: "Adam",
        description: "Dominant, Firm",
        gender: "male",
        accent: "american",
        age: "30-40",
    },
    DemoVoice {
        id: "alice",
        name: "Alice",
        description: "Clear, Engaging Educator",
        gender: "female",
        accent: "british",
        age: "30-40",
    },
    DemoVoice {
        id: "bill",
        name: "Bill",
        description: "Wise, Mature, Balanced",
        gender: "male",
        accent: "american",
        age: "50-60",
    },
    DemoVoice {
        id: "jessica",
        name: "Jessica",
        description: "Playful, Bright, Warm",
        gender: "female",
        accent: "american",
        age: "20-30",
    },
    DemoVoice {
        id: "lily",
        name: "Lily",
        description: "Velvety Actress",
        gender: "female",
        accent: "british",
        age: "30-40",
    },
];

fn selector_voices() -> Arc<[ui_ai::VoiceSelectorVoice]> {
    Arc::from(
        DEMO_VOICES
            .iter()
            .map(|voice| {
                ui_ai::VoiceSelectorVoice::new(voice.id, voice.name).description(voice.description)
            })
            .collect::<Vec<_>>(),
    )
}

fn demo_voice(id: &str) -> DemoVoice {
    DEMO_VOICES
        .iter()
        .copied()
        .find(|voice| voice.id == id)
        .expect("demo voice metadata should exist for every selector voice")
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let playing = cx.local_model_keyed("playing", || None::<Arc<str>>);

    let voices = selector_voices();

    let selected = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or(None);
    let selected_voice = selected
        .as_deref()
        .and_then(|id| DEMO_VOICES.iter().copied().find(|voice| voice.id == id));

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

    let playing_now = cx
        .get_model_cloned(&playing, Invalidation::Paint)
        .unwrap_or(None);

    let trigger_content = if let Some(voice) = selected_voice {
        ui::h_row(move |cx| {
            vec![
                ui_ai::VoiceSelectorName::new(voice.name).into_element(cx),
                ui_ai::VoiceSelectorAccent::new()
                    .value(voice.accent)
                    .into_element(cx),
                ui_ai::VoiceSelectorBullet::new().into_element(cx),
                ui_ai::VoiceSelectorAge::new(voice.age).into_element(cx),
                ui_ai::VoiceSelectorBullet::new().into_element(cx),
                ui_ai::VoiceSelectorGender::new()
                    .value(voice.gender)
                    .into_element(cx),
            ]
        })
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N2)
        .items(Items::Center)
        .into_element(cx)
    } else {
        ui_ai::VoiceSelectorValue::new()
            .placeholder("Select a voice...")
            .into_element(cx)
    };

    let trigger_button = shadcn::Button::new("")
        .variant(shadcn::ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(320.0)),
        )
        .children([trigger_content])
        .into_element(cx);

    let voice_list = ui_ai::VoiceSelectorList::new()
        .children({
            let playing_model = playing.clone();
            let playing_now = playing_now.clone();
            move |voices: Arc<[ui_ai::VoiceSelectorVoice]>| {
                voices
                    .iter()
                    .map(|voice| {
                        let details = demo_voice(voice.id.as_ref());
                        let preview_voice_id = voice.id.clone();
                        let playing_model = playing_model.clone();
                        let is_playing = playing_now.as_deref() == Some(voice.id.as_ref());

                        let on_preview: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _| {
                                let _ = host.models_mut().update(&playing_model, |v| {
                                    if v.as_deref() == Some(preview_voice_id.as_ref()) {
                                        *v = None;
                                    } else {
                                        *v = Some(preview_voice_id.clone());
                                    }
                                });
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        ui_ai::VoiceSelectorItem::new(voice.name.clone())
                            .value(voice.id.clone())
                            .keywords([
                                details.description,
                                details.gender,
                                details.accent,
                                details.age,
                            ])
                            .test_id(format!(
                                "ui-ai-voice-selector-demo-item-{}",
                                voice.id.as_ref()
                            ))
                            .child(
                                ui_ai::VoiceSelectorPreview::new()
                                    .playing(is_playing)
                                    .on_play_action(on_preview)
                                    .test_id(format!(
                                        "ui-ai-voice-selector-demo-preview-{}",
                                        voice.id.as_ref()
                                    )),
                            )
                            .child(ui_ai::VoiceSelectorName::new(voice.name.clone()))
                            .child(ui_ai::VoiceSelectorDescription::new(details.description))
                            .child(ui_ai::VoiceSelectorBullet::new())
                            .child(ui_ai::VoiceSelectorAccent::new().value(details.accent))
                            .child(ui_ai::VoiceSelectorBullet::new())
                            .child(ui_ai::VoiceSelectorAge::new(details.age))
                            .child(ui_ai::VoiceSelectorBullet::new())
                            .child(ui_ai::VoiceSelectorGender::new().value(details.gender))
                    })
                    .collect::<Vec<_>>()
            }
        })
        .empty_text("No voices found.")
        .refine_scroll_layout(LayoutRefinement::default().max_h(Px(280.0)));

    let selector = ui_ai::VoiceSelector::from_arc(voices)
        .open_model(open.clone())
        .value_model(value.clone())
        .children([
            ui_ai::VoiceSelectorChild::Trigger(
                ui_ai::VoiceSelectorTrigger::new(trigger_button)
                    .test_id("ui-ai-voice-selector-demo-trigger"),
            ),
            ui_ai::VoiceSelectorChild::Content(
                ui_ai::VoiceSelectorContent::new([])
                    .input(
                        ui_ai::VoiceSelectorInput::new().test_id("ui-ai-voice-selector-demo-input"),
                    )
                    .list(voice_list)
                    .test_id_root("ui-ai-voice-selector-demo-content")
                    .refine_layout(LayoutRefinement::default().max_w(Px(448.0))),
            ),
        ])
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("VoiceSelector (AI Elements)"),
            cx.text("Composable dialog + command recipe. Apps still own inventory and preview playback state."),
            marker,
            open_marker,
            selector,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
