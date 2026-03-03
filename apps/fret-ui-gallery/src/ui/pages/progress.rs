use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::progress as snippets;

pub(super) fn preview_progress(
    cx: &mut ElementContext<'_, App>,
    _progress: Model<f32>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let rtl = snippets::rtl::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Progress demo (new-york-v4).",
            "The demo uses a one-shot timer (500ms) to update the progress value from 13 → 66.",
            "For labeled progress, prefer composing `FieldLabel` + `Progress` instead of adding one-off widget APIs.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Progress demo: value update after 500ms."),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-progress-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-progress-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-progress-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-progress")]
}
