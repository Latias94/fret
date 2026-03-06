use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_test_results_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let overview = snippets::test_results_demo::render(cx);
    let basic = snippets::test_results_basic::render(cx);
    let suites = snippets::test_results_suites::render(cx);
    let errors = snippets::test_results_errors::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("Display test suite results with pass/fail/skip status, progress, and error details."),
        vec![
            DocSection::new("Overview", overview)
                .description("Rust/Fret analogue of the official AI Elements all-in-one preview.")
                .test_id_prefix("ui-gallery-ai-test-results-overview")
                .code_rust_from_file_region(snippets::test_results_demo::SOURCE, "example"),
            DocSection::new("Basic Usage", basic)
                .description(
                    "Summary badges and duration only, matching the official basic example.",
                )
                .test_id_prefix("ui-gallery-ai-test-results-basic")
                .code_rust_from_file_region(snippets::test_results_basic::SOURCE, "example"),
            DocSection::new("With Test Suites", suites)
                .description(
                    "Expandable suites with per-suite status counts and individual test rows.",
                )
                .test_id_prefix("ui-gallery-ai-test-results-suites")
                .code_rust_from_file_region(snippets::test_results_suites::SOURCE, "example"),
            DocSection::new("With Error Details", errors)
                .description(
                    "Failed tests render inline error panels and stack traces for copyable repros.",
                )
                .test_id_prefix("ui-gallery-ai-test-results-errors")
                .code_rust_from_file_region(snippets::test_results_errors::SOURCE, "example"),
        ],
    );

    vec![body]
}
