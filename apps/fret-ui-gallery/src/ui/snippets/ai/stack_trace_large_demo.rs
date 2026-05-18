pub const SOURCE: &str = include_str!("stack_trace_large_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui::element::{AnyElement, Length, SemanticsProps, SpacerProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Generic,
            test_id: Some(Arc::<str>::from(test_id)),
            ..Default::default()
        },
        |cx| {
            vec![cx.spacer(SpacerProps {
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(fret_core::Px(0.0)),
                        height: Length::Px(fret_core::Px(0.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                min: fret_core::Px(0.0),
            })]
        },
    )
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let opened = cx.local_model_keyed("opened", || false);

    let opened_now = cx
        .get_model_copied(&opened, Invalidation::Paint)
        .unwrap_or(false);
    let marker = opened_now.then(|| state_marker(cx, "ui-ai-stack-trace-large-opened-marker"));

    let mut trace = String::new();
    trace.push_str("Error: synthetic large stack\n");
    for index in 0..220usize {
        trace.push_str(&format!(
            "    at f{index} (src/module_{index:04}.rs:{line}:{col})\n",
            line = 10 + (index % 97),
            col = 1 + (index % 13)
        ));
    }

    let stack = ui_ai::StackTrace::new(trace)
        .default_open(false)
        .test_id_root("ui-ai-stack-trace-large-root")
        .test_id_header_trigger("ui-ai-stack-trace-large-header")
        .test_id_content("ui-ai-stack-trace-large-content")
        .test_id_frames_viewport("ui-ai-stack-trace-large-frames-viewport")
        .frame_test_id_prefix("ui-ai-stack-trace-large-frame")
        .on_file_path_click(Arc::new({
            let opened = opened.clone();
            move |host, _action_cx, _path, _line, _col| {
                let _ = host.models_mut().update(&opened, |v| *v = true);
            }
        }))
        .into_element(cx);

    ui::v_flex(move |cx| {
        let mut out = vec![
            decl_text::text_section_chrome_label(cx, "StackTrace (Large)"),
            decl_text::text_paragraph(cx, "Scroll in the frames viewport and click a file path."),
            stack,
        ];
        if let Some(marker) = marker {
            out.push(marker);
        }
        out
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
