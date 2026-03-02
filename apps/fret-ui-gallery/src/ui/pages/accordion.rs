use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::accordion as snippets;

pub(super) fn preview_accordion(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let _ = value;

    let demo = snippets::demo::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows `accordion-demo.tsx` (new-york-v4).",
            "Extras keep additional stress cases and local regression gates (not part of upstream).",
            "API reference: `ecosystem/fret-ui-shadcn/src/accordion.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "A vertically stacked set of interactive headings that each reveal a section of content.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Upstream shadcn AccordionDemo structure.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-accordion-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .description("Fret-specific variants + RTL coverage.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-accordion-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("Notes", notes).description("Parity notes and references."),
        ],
    );

    vec![body.test_id("ui-gallery-accordion")]
}

