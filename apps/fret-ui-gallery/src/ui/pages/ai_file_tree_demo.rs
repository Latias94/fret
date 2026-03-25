use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_file_tree_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::file_tree_demo::render(cx);
    let basic = snippets::file_tree_basic::preview(cx);
    let selection = snippets::file_tree_selection::preview(cx);
    let expanded = snippets::file_tree_expanded::preview(cx);
    let large = snippets::file_tree_large::preview(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::build(cx, "Demo", demo)
                .test_id_prefix("ui-gallery-ai-file-tree-demo")
                .description(
                    "Controlled selection with initial expanded folders, mirroring the official overview example.",
                )
                .code_rust_from_file_region(snippets::file_tree_demo::SOURCE, "example"),
            DocSection::build(cx, "Basic Usage", basic)
                .description(
                    "Uncontrolled expansion state, matching the upstream default behavior.",
                )
                .code_rust_from_file_region(snippets::file_tree_basic::SOURCE, "example"),
            DocSection::build(cx, "With Selection", selection)
                .description("Controlled selection state, matching the official selection example.")
                .code_rust_from_file_region(snippets::file_tree_selection::SOURCE, "example"),
            DocSection::build(cx, "Default Expanded", expanded)
                .description("Uncontrolled mode with initial expanded paths.")
                .code_rust_from_file_region(snippets::file_tree_expanded::SOURCE, "example"),
            DocSection::build(cx, "Large (Virtualized)", large)
                .description(
                    "Fret extension: a large tree under height constraints to exercise virtualization + scrolling.",
                )
                .code_rust_from_file_region(snippets::file_tree_large::SOURCE, "example"),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
