use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::separator as snippets;

pub(super) fn preview_separator(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/separator.rs` (Separator, SeparatorOrientation).",
            "Separator is a minimal leaf primitive, so the main parity gap here is usage clarity rather than missing composition APIs.",
            "Vertical separators often want `flex_stretch_cross_axis(true)` when placed inside fixed-height rows.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Separator docs flow: Demo -> Usage."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Separator`.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Notes", notes).no_shell(),
        ],
    );

    vec![body.test_id("ui-gallery-separator")]
}
