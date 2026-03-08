pub const SOURCE: &str = include_str!("voice_selector_demo.rs");

// region: example
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::SemanticsDecoration;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{Items, LayoutRefinement, Space};
use fret_ui_shadcn::{self as shadcn, prelude::*};
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

#[derive(Default)]
struct DemoModels {
    open: Option<Model<bool>>,
    value: Option<Model<Option<Arc<str>>>>,
    playing: Option<Model<Option<Arc<str>>>>,
}

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

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.open.is_none() || st.value.is_none() || st.playing.is_none()
    });
    if needs_init {
        let open = cx.app.models_mut().insert(false);
        let value = cx.app.models_mut().insert(None::<Arc<str>>);
        let playing = cx.app.models_mut().insert(None::<Arc<str>>);
        cx.with_state(DemoModels::default, move |st| {
            st.open = Some(open.clone());
            st.value = Some(value.clone());
            st.playing = Some(playing.clone());
        });
    }

    let (open, value, playing) = cx.with_state(DemoModels::default, |st| {
        (
            st.open.clone().expect("open"),
            st.value.clone().expect("value"),
            st.playing.clone().expect("playing"),
        )
    });

    let voices = selector_voices();

    let selected = cx.app.models().read(&value, |v| v.clone()).ok().flatten();
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

    let playing_now = cx.app.models().read(&playing, |v| v.clone()).ok().flatten();

    let selector = ui_ai::VoiceSelector::from_arc(voices)
        .open_model(open.clone())
        .value_model(value.clone())
        .into_element_with_children(cx, move |cx| {
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

            let button = shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .min_w_0()
                        .max_w(Px(320.0)),
                )
                .children([trigger_content])
                .into_element(cx);

            let trigger = ui_ai::VoiceSelectorTrigger::new(button)
                .test_id("ui-ai-voice-selector-demo-trigger")
                .into_element(cx);

            let mut items = Vec::new();
            for voice in DEMO_VOICES {
                let value_model = value.clone();
                let open_model = open.clone();
                let playing_model = playing.clone();
                let id = Arc::<str>::from(voice.id);
                let voice_id = voice.id;

                let on_select: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _| {
                    let _ = host
                        .models_mut()
                        .update(&value_model, |v| *v = Some(id.clone()));
                    let _ = host.models_mut().update(&open_model, |v| *v = false);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                });

                let on_preview: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _| {
                        let _ = host.models_mut().update(&playing_model, |v| {
                            if v.as_deref() == Some(voice_id) {
                                *v = None;
                            } else {
                                *v = Some(Arc::<str>::from(voice_id));
                            }
                        });
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                    });

                let is_playing = playing_now.as_deref() == Some(voice.id);
                let row = ui::h_row(move |cx| {
                    vec![
                        ui_ai::VoiceSelectorPreview::new()
                            .playing(is_playing)
                            .on_play_action(on_preview)
                            .test_id(format!("ui-ai-voice-selector-demo-preview-{}", voice.id))
                            .into_element(cx),
                        ui_ai::VoiceSelectorName::new(voice.name).into_element(cx),
                        ui_ai::VoiceSelectorDescription::new(voice.description).into_element(cx),
                        ui_ai::VoiceSelectorBullet::new().into_element(cx),
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
                .into_element(cx);

                items.push(
                    ui_ai::VoiceSelectorItem::new(voice.name)
                        .value(voice.id)
                        .keywords([voice.description, voice.gender, voice.accent, voice.age])
                        .test_id(format!("ui-ai-voice-selector-demo-item-{}", voice.id))
                        .on_select_action(on_select)
                        .children([row]),
                );
            }

            let content = ui_ai::VoiceSelectorContent::new([
                ui_ai::VoiceSelectorInput::new()
                    .test_id("ui-ai-voice-selector-demo-input")
                    .into_element(cx),
                ui_ai::VoiceSelectorList::new_entries(items)
                    .empty_text("No voices found.")
                    .refine_scroll_layout(LayoutRefinement::default().max_h(Px(280.0)))
                    .into_element(cx),
            ])
            .test_id_root("ui-ai-voice-selector-demo-content")
            .refine_layout(LayoutRefinement::default().max_w(Px(448.0)))
            .into_element(cx);

            (trigger, content)
        });

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
