pub const SOURCE: &str = include_str!("stack_trace_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui::element::SemanticsDecoration;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let status = cx.local_model_keyed("status", || {
        Arc::<str>::from("Ready for copy or file-open actions.")
    });

    let status_text = cx
        .get_model_cloned(&status, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("Ready for copy or file-open actions."));

    let on_copy = Arc::new({
        let status = status.clone();
        move |host: &mut dyn fret_ui::action::UiActionHost, action_cx: fret_ui::action::ActionCx| {
            let _ = host.models_mut().update(&status, |text| {
                *text = Arc::<str>::from("Copied stack trace")
            });
            host.notify(action_cx);
        }
    });
    let on_error = Arc::new({
        let status = status.clone();
        move |host: &mut dyn fret_ui::action::UiActionHost,
              action_cx: fret_ui::action::ActionCx,
              error: fret_core::ClipboardAccessError| {
            let label = error
                .message
                .unwrap_or_else(|| format!("Copy failed: {:?}", error.kind));
            let _ = host
                .models_mut()
                .update(&status, |text| *text = Arc::<str>::from(label));
            host.notify(action_cx);
        }
    });

    let on_file_path_click = Arc::new({
        let status = status.clone();
        move |host: &mut dyn fret_ui::action::UiActionHost,
              action_cx: fret_ui::action::ActionCx,
              path: Arc<str>,
              line: Option<u32>,
              column: Option<u32>| {
            let label = match (line, column) {
                (Some(line), Some(column)) => format!("Open file seam: {path}:{line}:{column}"),
                (Some(line), None) => format!("Open file seam: {path}:{line}"),
                _ => format!("Open file seam: {path}"),
            };
            let _ = host
                .models_mut()
                .update(&status, |text| *text = Arc::<str>::from(label));
            host.notify(action_cx);
        }
    });

    let trace: Arc<str> = Arc::from(
        "TypeError: Cannot read properties of undefined (reading 'map')\n    at UserList (/app/components/UserList.tsx:15:23)\n    at renderWithHooks (node_modules/react-dom/cjs/react-dom.development.js:14985:18)\n    at mountIndeterminateComponent (node_modules/react-dom/cjs/react-dom.development.js:17811:13)\n    at beginWork (node_modules/react-dom/cjs/react-dom.development.js:19049:16)\n    at HTMLUnknownElement.callCallback (node_modules/react-dom/cjs/react-dom.development.js:3945:14)\n",
    );

    let stack = ui_ai::StackTrace::new(trace)
        .default_open(true)
        .on_file_path_click(on_file_path_click)
        .test_id_root("ui-ai-stack-trace-root")
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
                            .on_copy(on_copy)
                            .on_error(on_error)
                            .test_id("ui-ai-stack-trace-copy")
                            .copied_marker_test_id("ui-ai-stack-trace-copied-marker")
                            .into_element(cx),
                        ui_ai::StackTraceExpandButton::default().into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .test_id("ui-ai-stack-trace-header")
                .into_element(cx),
                ui_ai::StackTraceContent::new([ui_ai::StackTraceFrames::default()
                    .test_id("ui-ai-stack-trace-frames")
                    .into_element(cx)])
                .test_id("ui-ai-stack-trace-content")
                .into_element(cx),
            ]
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("StackTrace (AI Elements)"),
            cx.text("Docs-aligned compound parts API with copy + file-open seams."),
            stack,
            cx.text(format!("Status: {status_text}")).attach_semantics(
                SemanticsDecoration::default().test_id("ui-ai-stack-trace-status"),
            ),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
