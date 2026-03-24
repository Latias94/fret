use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn status_colors_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Status", "Color", "Use case"],
        [
            ["passed", "Green", "Test succeeded"],
            ["failed", "Red", "Test failed"],
            ["skipped", "Yellow", "Test skipped"],
            ["running", "Blue", "Test in progress"],
        ],
        false,
    )
}

fn parts_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Key inputs", "Notes"],
        [
            [
                "TestResults",
                "summary",
                "Root surface; summary-driven parts can read from the root provider.",
            ],
            [
                "TestResultsHeader",
                "children",
                "Header row with border-bottom + padding.",
            ],
            [
                "TestResultsSummary",
                "summary | from_context()",
                "Renders pass/fail/skip badges.",
            ],
            [
                "TestResultsDuration",
                "summary | from_context()",
                "Renders formatted duration when present.",
            ],
            [
                "TestResultsProgress",
                "summary | from_context()",
                "Progress bar + pass ratio labels.",
            ],
            [
                "TestResultsContent",
                "children",
                "Padded wrapper for suites and rows.",
            ],
            [
                "TestSuite",
                "new(trigger, content) | named(name, status)",
                "Collapsible suite shell; root can now provide context for trigger/content parts.",
            ],
            [
                "TestSuiteName",
                "new(name, status) | from_context() | children(...)",
                "Trigger row; can read suite context from the root and accept custom label children.",
            ],
            [
                "TestSuiteStats",
                "passed, failed, skipped",
                "Optional trailing stats helper for custom suite rows.",
            ],
            [
                "Test",
                "name, status, duration_ms, details, on_activate",
                "Row surface with optional error details and activation seam.",
            ],
            [
                "TestStatus / TestName / TestDuration",
                "new(...) | from_context()",
                "Composable row parts for custom `Test::children(...)` layouts.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_test_results_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let overview = snippets::test_results_demo::render(cx);
    let features = crate::ui::doc_layout::notes_block([
        "Summary statistics (passed/failed/skipped)",
        "Progress bar visualization",
        "Collapsible test suites",
        "Individual test status and duration",
        "Error messages with stack traces",
        "Color-coded status indicators",
    ]);
    let status_colors = status_colors_table(cx);
    let basic = snippets::test_results_basic::render(cx);
    let suites = snippets::test_results_suites::render(cx);
    let errors = snippets::test_results_errors::render(cx);
    let props = parts_props_table(cx);
    let overview_section = DocSection::build(cx, "Overview", overview)
        .description("Rust/Fret analogue of the official AI Elements all-in-one preview.")
        .test_id_prefix("ui-gallery-ai-test-results-overview")
        .code_rust_from_file_region(snippets::test_results_demo::SOURCE, "example");
    let features_section = DocSection::build(cx, "Features", features).no_shell();
    let status_colors_section = DocSection::build(cx, "Status Colors", status_colors).no_shell();
    let basic_section = DocSection::build(cx, "Basic Usage", basic)
        .description("Summary badges and duration only, matching the official basic example.")
        .test_id_prefix("ui-gallery-ai-test-results-basic")
        .code_rust_from_file_region(snippets::test_results_basic::SOURCE, "example");
    let suites_section = DocSection::build(cx, "With Test Suites", suites)
        .description("Expandable suites with individual test rows, matching the docs sample.")
        .test_id_prefix("ui-gallery-ai-test-results-suites")
        .code_rust_from_file_region(snippets::test_results_suites::SOURCE, "example");
    let errors_section = DocSection::build(cx, "With Error Details", errors)
        .description(
            "Failed tests render inline error panels and stack traces for copyable repros.",
        )
        .test_id_prefix("ui-gallery-ai-test-results-errors")
        .code_rust_from_file_region(snippets::test_results_errors::SOURCE, "example");
    let props_section = DocSection::build(cx, "Parts & Props", props).no_shell();

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "The TestResults component displays test suite results including summary statistics, progress, individual tests, and error details.",
        ),
        vec![
            overview_section,
            features_section,
            status_colors_section,
            basic_section,
            suites_section,
            errors_section,
            props_section,
        ],
    );

    vec![body.into_element(cx)]
}
