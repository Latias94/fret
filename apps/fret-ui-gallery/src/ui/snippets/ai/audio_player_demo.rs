pub const SOURCE: &str = include_str!("audio_player_demo.rs");

// region: example
use fret_ui::Invalidation;
use fret_ui::element::SemanticsProps;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let playing = cx.local_model_keyed("playing", || false);
    let muted = cx.local_model_keyed("muted", || false);
    let time = cx.local_model_keyed("time", || vec![0.0]);
    let duration = cx.local_model_keyed("duration", || 120.0f32);
    let volume = cx.local_model_keyed("volume", || vec![0.8]);

    let playing_now = cx
        .get_model_copied(&playing, Invalidation::Paint)
        .unwrap_or(false);
    let muted_now = cx
        .get_model_copied(&muted, Invalidation::Paint)
        .unwrap_or(false);
    let time_now = cx
        .get_model_cloned(&time, Invalidation::Paint)
        .and_then(|v| v.first().copied())
        .unwrap_or(0.0);

    let playing_marker = playing_now.then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-ai-audio-player-demo-playing-true")),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    });
    let muted_marker = muted_now.then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-ai-audio-player-demo-muted-true")),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    });
    let time_marker = (time_now > 0.0).then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-ai-audio-player-demo-time-nonzero")),
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
                let _ = host.models_mut().update(&time, |v| {
                    if let Some(first) = v.first_mut() {
                        *first = next.max(0.0);
                    }
                });
            }
        }))
        .refine_style(ChromeRefinement::default().p(Space::N3))
        .into_element_with_children(cx, move |cx, _controller| {
            let controls = ui_ai::AudioPlayerControlBar::new([
                shadcn::ButtonGroupItem::from(
                    ui_ai::AudioPlayerPlayButton::new()
                        .test_id("ui-ai-audio-player-demo-play")
                        .into_element(cx),
                ),
                shadcn::ButtonGroupItem::from(
                    ui_ai::AudioPlayerSeekBackwardButton::new().into_element(cx),
                ),
                shadcn::ButtonGroupItem::from(
                    ui_ai::AudioPlayerSeekForwardButton::new()
                        .test_id("ui-ai-audio-player-demo-seek-forward")
                        .into_element(cx),
                ),
                shadcn::ButtonGroupItem::from(
                    ui_ai::AudioPlayerMuteButton::new()
                        .test_id("ui-ai-audio-player-demo-mute")
                        .into_element(cx),
                ),
            ])
            .test_id("ui-ai-audio-player-demo-controls")
            .into_element(cx);

            let time_row = ui::h_flex(move |cx| {
                vec![
                    ui_ai::AudioPlayerTimeDisplay::new().into_element(cx),
                    ui_ai::AudioPlayerTimeRange::new()
                        .test_id("ui-ai-audio-player-demo-time")
                        .into_element(cx),
                    ui_ai::AudioPlayerDurationDisplay::new().into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            vec![controls, time_row]
        });

    ui::v_flex(move |cx| {
        let mut out = vec![
            cx.text("AudioPlayer (AI Elements)"),
            cx.text("Chrome-only controls: apps own playback + time driving."),
            player,
        ];
        if let Some(m) = playing_marker {
            out.push(m);
        }
        if let Some(m) = muted_marker {
            out.push(m);
        }
        if let Some(m) = time_marker {
            out.push(m);
        }
        out
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
