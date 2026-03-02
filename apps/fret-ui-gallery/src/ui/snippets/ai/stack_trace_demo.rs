pub const SOURCE: &str = include_str!("stack_trace_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let trace: Arc<str> = Arc::from(
        "Error: failed to render\n    at render (src/render.rs:42:9)\n    at main (src/main.rs:10:1)\n",
    );

    let stack = ui_ai::StackTrace::new(trace)
        .default_open(false)
        .test_id_root("ui-ai-stack-trace-root")
        .test_id_header_trigger("ui-ai-stack-trace-header")
        .test_id_copy_button("ui-ai-stack-trace-copy")
        .test_id_copy_copied_marker("ui-ai-stack-trace-copied-marker")
        .test_id_content("ui-ai-stack-trace-content")
        .test_id_frames("ui-ai-stack-trace-frames")
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("StackTrace (AI Elements)"),
                cx.text("Disclosure + copy surface; file paths are link-like."),
                stack,
            ]
        },
    )
}
// endregion: example

