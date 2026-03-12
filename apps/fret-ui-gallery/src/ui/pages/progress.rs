use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::progress as snippets;

pub(super) fn preview_progress(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let label = snippets::label::render(cx);
    let controlled = snippets::controlled::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/progress.rs` (Progress).",
            "Progress is a leaf display control, so the main parity gap here is usage clarity rather than missing composition APIs.",
            "The demo uses a one-shot timer (500ms) to update the progress value from 13 to 66.",
            "For labeled progress, prefer composing `FieldLabel` + `Progress` instead of adding one-off widget APIs.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Progress docs flow: Demo -> Usage. Gallery adds labeled, controlled, and RTL variants.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-progress-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Progress`.")
                .test_id_prefix("ui-gallery-progress-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Label", label)
                .test_id_prefix("ui-gallery-progress-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("Controlled", controlled)
                .test_id_prefix("ui-gallery-progress-controlled")
                .code_rust_from_file_region(snippets::controlled::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-progress-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).test_id_prefix("ui-gallery-progress-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-progress")]
}
