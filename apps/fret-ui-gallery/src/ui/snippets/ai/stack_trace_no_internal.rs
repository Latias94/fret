pub const SOURCE: &str = include_str!("stack_trace_no_internal.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let trace: Arc<str> = Arc::from(
        "TypeError: Cannot read properties of undefined (reading 'map')\n    at UserList (/app/src/components/UserList.tsx:15:23)\n    at App (/app/src/App.tsx:42:5)\n    at renderWithHooks (node_modules/react-dom/cjs/react-dom.development.js:14985:18)\n    at mountIndeterminateComponent (node_modules/react-dom/cjs/react-dom.development.js:17811:13)\n    at beginWork (node_modules/react-dom/cjs/react-dom.development.js:19049:16)\n",
    );

    let stack = ui_ai::StackTrace::new(trace)
        .default_open(true)
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::StackTraceHeader::new([
                    ui_ai::StackTraceError::new([
                        ui_ai::StackTraceErrorType::default().into_element(cx),
                        ui_ai::StackTraceErrorMessage::default().into_element(cx),
                    ])
                    .into_element(cx),
                    ui_ai::StackTraceActions::new([
                        ui_ai::StackTraceCopyButton::default().into_element(cx),
                        ui_ai::StackTraceExpandButton::default().into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                ui_ai::StackTraceContent::new([ui_ai::StackTraceFrames::default()
                    .show_internal_frames(false)
                    .frame_test_id_prefix("ui-ai-stack-trace-no-internal-frame")
                    .into_element(cx)])
                .test_id("ui-ai-stack-trace-no-internal-content")
                .into_element(cx),
            ]
        });

    ui::v_flex(move |_cx| vec![stack])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .into_element(cx)
}
// endregion: example
