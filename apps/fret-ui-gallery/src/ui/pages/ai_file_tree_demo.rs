use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_file_tree_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::file_tree_demo::render(cx);
    let basic = snippets::file_tree_basic::preview(cx).into_element(cx);
    let expanded = snippets::file_tree_expanded::preview(cx).into_element(cx);
    let large = snippets::file_tree_large::preview(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-ai-file-tree-demo")
                .code_rust_from_file_region(snippets::file_tree_demo::SOURCE, "example"),
            DocSection::new("Basic Usage", basic)
                .description(
                    "Uncontrolled expansion state, matching the upstream default behavior.",
                )
                .code_rust_from_file_region(snippets::file_tree_basic::SOURCE, "example"),
            DocSection::new("Default Expanded", expanded)
                .description("Uncontrolled mode with initial expanded paths.")
                .code_rust_from_file_region(snippets::file_tree_expanded::SOURCE, "example"),
            DocSection::new("Large (Virtualized)", large)
                .description(
                    "Large tree under height constraints to exercise virtualization + scrolling.",
                )
                .code_rust_from_file_region(snippets::file_tree_large::SOURCE, "example"),
        ],
    );

    vec![body]
}
