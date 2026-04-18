pub const SOURCE: &str = include_str!("audio_player_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui::element::SemanticsProps;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn demo_speech_data() -> ui_ai::AudioPlayerSpeechData {
    ui_ai::AudioPlayerSpeechData::new("ZGVtbw==", "mp3", "audio/mpeg")
}

fn render_demo_audio_player(
    cx: &mut AppComponentCx<'_>,
    element: ui_ai::AudioPlayerElement,
    model_key: &'static str,
    test_prefix: &'static str,
    duration_secs: f32,
) -> AnyElement {
    let _docs_surface_marker: Option<shadcn::ButtonGroupText> = None;

    let playing = cx.local_model_keyed(format!("{model_key}-playing"), || false);
    let muted = cx.local_model_keyed(format!("{model_key}-muted"), || false);
    let time = cx.local_model_keyed(format!("{model_key}-time"), || vec![0.0]);
    let duration = cx.local_model_keyed(format!("{model_key}-duration"), || duration_secs);
    let volume = cx.local_model_keyed(format!("{model_key}-volume"), || vec![0.8]);

    let playing_now = cx
        .get_model_copied(&playing, Invalidation::Paint)
        .unwrap_or(false);
    let muted_now = cx
        .get_model_copied(&muted, Invalidation::Paint)
        .unwrap_or(false);
    let time_now = cx
        .get_model_cloned(&time, Invalidation::Paint)
        .and_then(|values| values.first().copied())
        .unwrap_or(0.0);

    let playing_marker = playing_now.then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from(format!("{test_prefix}-playing-true"))),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    });
    let muted_marker = muted_now.then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from(format!("{test_prefix}-muted-true"))),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    });
    let time_marker = (time_now > 0.0).then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from(format!("{test_prefix}-time-nonzero"))),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    });

    let player = ui_ai::AudioPlayer::new()
        .playing_model(playing.clone())
        .muted_model(muted.clone())
        .time_model(time.clone())
        .duration_secs_model(duration.clone())
        .volume_model(volume.clone())
        .on_seek_to(Arc::new({
            let time = time.clone();
            move |host, _action_cx, next| {
                let _ = host.models_mut().update(&time, |values| {
                    if let Some(first) = values.first_mut() {
                        *first = next.max(0.0);
                    }
                });
            }
        }))
        .refine_style(ChromeRefinement::default().p(Space::N3))
        .element(element.test_id(format!("{test_prefix}-element")))
        .control_bar(
            ui_ai::AudioPlayerControlBar::empty()
                .play_button(
                    ui_ai::AudioPlayerPlayButton::new().test_id(format!("{test_prefix}-play")),
                )
                .seek_backward_button(ui_ai::AudioPlayerSeekBackwardButton::new())
                .seek_forward_button(
                    ui_ai::AudioPlayerSeekForwardButton::new()
                        .test_id(format!("{test_prefix}-seek-forward")),
                )
                .time_display(ui_ai::AudioPlayerTimeDisplay::new())
                .time_range(
                    ui_ai::AudioPlayerTimeRange::new().test_id(format!("{test_prefix}-time")),
                )
                .duration_display(ui_ai::AudioPlayerDurationDisplay::new())
                .mute_button(
                    ui_ai::AudioPlayerMuteButton::new().test_id(format!("{test_prefix}-mute")),
                )
                .volume_range(
                    ui_ai::AudioPlayerVolumeRange::new().test_id(format!("{test_prefix}-volume")),
                )
                .test_id(format!("{test_prefix}-controls")),
        )
        .into_element(cx);

    ui::v_flex(move |cx| {
        let player_row = ui::h_flex(move |_cx| [player])
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .justify_center()
            .into_element(cx);

        let mut out = vec![player_row];
        if let Some(marker) = playing_marker {
            out.push(marker);
        }
        if let Some(marker) = muted_marker {
            out.push(marker);
        }
        if let Some(marker) = time_marker {
            out.push(marker);
        }
        out
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}

pub fn render_speech_result(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    render_demo_audio_player(
        cx,
        ui_ai::AudioPlayerElement::new().speech_data(demo_speech_data()),
        "ui-ai-audio-player-docs-speech",
        "ui-ai-audio-player-demo",
        126.0,
    )
}
// endregion: example
