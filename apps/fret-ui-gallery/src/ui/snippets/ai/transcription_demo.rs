pub const SOURCE: &str = include_str!("transcription_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui::element::SemanticsProps;
use fret_ui_ai as ui_ai;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let current_time = cx.local_model_keyed("current_time", || 0.0_f32);
    let time = cx.local_model_keyed("time", || vec![0.0_f32]);
    let duration = cx.local_model_keyed("duration", || 3.1_f32);

    let segments: Arc<[ui_ai::TranscriptionSegmentData]> = Arc::from(vec![
        ui_ai::TranscriptionSegmentData::new(0.119, 0.219, "You"),
        ui_ai::TranscriptionSegmentData::new(0.219, 0.259, " "),
        ui_ai::TranscriptionSegmentData::new(0.259, 0.439, "can"),
        ui_ai::TranscriptionSegmentData::new(0.439, 0.459, " "),
        ui_ai::TranscriptionSegmentData::new(0.459, 0.699, "build"),
        ui_ai::TranscriptionSegmentData::new(0.699, 0.720, " "),
        ui_ai::TranscriptionSegmentData::new(0.720, 0.799, "and"),
        ui_ai::TranscriptionSegmentData::new(0.799, 0.879, " "),
        ui_ai::TranscriptionSegmentData::new(0.879, 1.339, "host"),
        ui_ai::TranscriptionSegmentData::new(1.339, 1.359, " "),
        ui_ai::TranscriptionSegmentData::new(1.360, 1.539, "many"),
        ui_ai::TranscriptionSegmentData::new(1.539, 1.600, " "),
        ui_ai::TranscriptionSegmentData::new(1.600, 1.860, "different"),
        ui_ai::TranscriptionSegmentData::new(1.860, 1.899, " "),
        ui_ai::TranscriptionSegmentData::new(1.899, 2.099, "types"),
        ui_ai::TranscriptionSegmentData::new(2.099, 2.119, " "),
        ui_ai::TranscriptionSegmentData::new(2.119, 2.200, "of"),
        ui_ai::TranscriptionSegmentData::new(2.200, 2.259, " "),
        ui_ai::TranscriptionSegmentData::new(2.259, 2.960, "applications"),
        ui_ai::TranscriptionSegmentData::new(2.960, 3.100, "."),
    ]);

    let time_now = cx
        .get_model_copied(&current_time, Invalidation::Paint)
        .unwrap_or(0.0);

    let time_marker = cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Text,
            test_id: Some(Arc::from(if time_now <= 0.0 {
                "ui-ai-transcription-demo-time-zero"
            } else {
                "ui-ai-transcription-demo-time-nonzero"
            })),
            ..Default::default()
        },
        |cx| vec![cx.text("")],
    );

    let active_index = segments
        .iter()
        .position(|s| time_now >= s.start_second && time_now < s.end_second)
        .unwrap_or(0);
    let active_test_id: Arc<str> =
        Arc::from(format!("ui-ai-transcription-demo-active-{active_index}"));
    let active_marker = cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Text,
            test_id: Some(active_test_id),
            ..Default::default()
        },
        |cx| vec![cx.text("")],
    );

    let theme = Theme::global(&*cx.app).clone();
    let transcript_style =
        typography::TypographyPreset::content_ui(typography::UiTextSize::Base).resolve(&theme);

    let transport = ui_ai::AudioPlayer::new()
        .time_model(time.clone())
        .duration_secs_model(duration)
        .on_seek_to(Arc::new({
            let current_time = current_time.clone();
            move |host, _action_cx, next| {
                let _ = host
                    .models_mut()
                    .update(&current_time, |v| *v = next.max(0.0));
            }
        }))
        .refine_style(ChromeRefinement::default().p(Space::N3))
        .into_element_with_children(cx, move |cx, _controller| {
            let transport_row = ui::h_flex(move |cx| {
                vec![
                    ui_ai::AudioPlayerSeekBackwardButton::new().into_element(cx),
                    ui_ai::AudioPlayerTimeDisplay::new().into_element(cx),
                    ui_ai::AudioPlayerTimeRange::new()
                        .test_id("ui-ai-transcription-demo-scrubber")
                        .into_element(cx),
                    ui_ai::AudioPlayerDurationDisplay::new().into_element(cx),
                    ui_ai::AudioPlayerSeekForwardButton::new().into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            vec![transport_row]
        });

    let transcript = ui_ai::Transcription::from_arc(segments.clone())
        .current_time_model(current_time.clone())
        .on_seek(Arc::new({
            let current_time = current_time.clone();
            let timeline = time.clone();
            move |host, action_cx, next| {
                let _ = host.models_mut().update(&current_time, |v| *v = next);
                let _ = host.models_mut().update(&timeline, |v| {
                    if let Some(first) = v.first_mut() {
                        *first = next;
                    } else {
                        v.push(next);
                    }
                });
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }
        }))
        .test_id_root("ui-ai-transcription-demo-root")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element_with_children(cx, move |cx, seg, index| {
            let test_id: Arc<str> = Arc::from(format!("ui-ai-transcription-demo-seg-{index}"));
            ui_ai::TranscriptionSegment::new(seg, index)
                .test_id(test_id)
                .text_style(transcript_style.clone())
                .into_element(cx)
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("App-owned timeline + interactive transcript"),
            cx.text(
                "Drag the scrubber or click a segment to seek. The transcript consumes app-owned current_time just like the official AI Elements example.",
            ),
            transport,
            transcript,
            time_marker,
            active_marker,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
