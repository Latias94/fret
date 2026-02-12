use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_transcription_demo(
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
        current_time: Option<Model<f32>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| st.current_time.is_none());
    if needs_init {
        let current_time = cx.app.models_mut().insert(0.0_f32);
        cx.with_state(DemoModels::default, move |st| {
            st.current_time = Some(current_time.clone());
        });
    }

    let current_time = cx.with_state(DemoModels::default, |st| {
        st.current_time.clone().expect("current_time")
    });

    let segments: Arc<[ui_ai::TranscriptionSegmentData]> = Arc::from(vec![
        ui_ai::TranscriptionSegmentData::new(0.0, 4.0, "Hello,"),
        ui_ai::TranscriptionSegmentData::new(4.0, 8.0, "world."),
        ui_ai::TranscriptionSegmentData::new(8.0, 12.0, "Click"),
        ui_ai::TranscriptionSegmentData::new(12.0, 16.0, "a"),
        ui_ai::TranscriptionSegmentData::new(16.0, 20.0, "segment"),
        ui_ai::TranscriptionSegmentData::new(20.0, 24.0, "to"),
        ui_ai::TranscriptionSegmentData::new(24.0, 28.0, "seek."),
    ]);

    let time_now = cx
        .get_model_copied(&current_time, Invalidation::Paint)
        .unwrap_or(0.0);

    let time_marker = cx
        .text(format!("current_time={time_now:.0}"))
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Generic)
                .test_id(if time_now < 0.5 {
                    "ui-ai-transcription-demo-time-zero"
                } else {
                    "ui-ai-transcription-demo-time-nonzero"
                }),
        );

    let active_index = segments
        .iter()
        .position(|s| time_now >= s.start_second && time_now < s.end_second)
        .unwrap_or(0);
    let active_test_id: Arc<str> =
        Arc::from(format!("ui-ai-transcription-demo-active-{active_index}"));
    let active_marker = cx.text(format!("active={active_index}")).attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Generic)
            .test_id(active_test_id),
    );

    let transcript = ui_ai::Transcription::from_arc(segments.clone())
        .current_time_model(current_time.clone())
        .on_seek(Arc::new({
            let current_time = current_time.clone();
            move |host, action_cx, time| {
                let _ = host.models_mut().update(&current_time, |v| *v = time);
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }
        }))
        .test_id_root("ui-ai-transcription-demo-root")
        .into_element_with_children(cx, move |cx, seg, index| {
            let test_id: Arc<str> = Arc::from(format!("ui-ai-transcription-demo-seg-{index}"));
            ui_ai::TranscriptionSegment::new(seg, index)
                .test_id(test_id)
                .into_element(cx)
        });

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Transcription (AI Elements)"),
                cx.text(
                    "Segment click triggers `on_seek`; this demo mirrors it into `current_time`.",
                ),
                time_marker,
                active_marker,
                transcript,
            ]
        },
    )]
}
