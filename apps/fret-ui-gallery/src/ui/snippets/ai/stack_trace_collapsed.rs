pub const SOURCE: &str = include_str!("stack_trace_collapsed.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let trace: Arc<str> = Arc::from(
        "TypeError: Cannot read properties of undefined (reading 'map')\n    at UserList (/app/src/components/UserList.tsx:15:23)\n    at renderWithHooks (node_modules/react-dom/cjs/react-dom.development.js:14985:18)\n    at mountIndeterminateComponent (node_modules/react-dom/cjs/react-dom.development.js:17811:13)\n",
    );

    let stack = ui_ai::StackTrace::new(trace)
        .default_open(false)
        .test_id_root("ui-ai-stack-trace-collapsed-root")
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::StackTraceHeader::new([
                    ui_ai::StackTraceError::new([
                        ui_ai::StackTraceErrorType::default().into_element(cx),
                        ui_ai::StackTraceErrorMessage::default().into_element(cx),
                    ])
                    .into_element(cx),
                    ui_ai::StackTraceActions::new([
                        ui_ai::StackTraceCopyButton::default()
                            .test_id("ui-ai-stack-trace-collapsed-copy")
                            .into_element(cx),
                        ui_ai::StackTraceExpandButton::default().into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .test_id("ui-ai-stack-trace-collapsed-header")
                .into_element(cx),
                ui_ai::StackTraceContent::new([ui_ai::StackTraceFrames::default()
                    .test_id("ui-ai-stack-trace-collapsed-frames")
                    .into_element(cx)])
                .test_id("ui-ai-stack-trace-collapsed-content")
                .into_element(cx),
            ]
        });

    ui::v_flex(move |_cx| vec![stack])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .into_element(cx)
}
// endregion: example
