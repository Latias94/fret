use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::separator as snippets;

pub(super) fn preview_separator(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);

    let notes = doc_layout::notes(cx, ["Preview follows shadcn Separator demo (new-york-v4)."]);

    let body = doc_layout::render_doc_page(
        cx,
        Some("Visually or semantically separates content."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Notes", notes).no_shell(),
        ],
    );

    vec![body.test_id("ui-gallery-separator")]
}
