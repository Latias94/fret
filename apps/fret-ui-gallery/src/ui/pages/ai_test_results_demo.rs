use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn status_colors_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Status", "Color", "Use case"],
        [
            ["passed", "Green", "Test succeeded"],
            ["failed", "Red", "Test failed"],
            ["skipped", "Yellow", "Test skipped"],
            ["running", "Blue (animated)", "Test in progress"],
        ],
        false,
    )
}

fn parts_props_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Key inputs", "Notes"],
        [
            [
                "TestResults",
                "summary | into_element_with_children(...)",
                "Root surface; use the closure-based lane when summary-driven child parts read from root context.",
            ],
            [
                "TestResultsHeader",
                "children",
                "Header row with border-bottom + padding.",
            ],
            [
                "TestResultsSummary",
                "summary | from_context() | children(...)",
                "Renders pass/fail/skip badges, or a custom summary row.",
            ],
            [
                "TestResultsDuration",
                "summary | from_context() | children(...)",
                "Renders formatted duration when present, or custom duration content.",
            ],
            [
                "TestResultsProgress",
                "summary | from_context() | children(...)",
                "Progress bar + pass ratio labels, or a custom progress body.",
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
                "passed, failed, skipped | children(...)",
                "Optional trailing stats helper, or a custom trailing suite summary.",
            ],
            [
                "TestSuiteContent",
                "children",
                "Collapsible content wrapper with dividers between test rows.",
            ],
            [
                "Test",
                "name, status, duration_ms, details, on_activate | into_element_with_children(...)",
                "Row surface with optional custom content, error details, and activation seam.",
            ],
            [
                "TestStatus / TestName / TestDuration",
                "new(...) | from_context() | children(...)",
                "Composable row parts for custom `Test::children(...)` layouts.",
            ],
            [
                "TestError",
                "children",
                "Error panel wrapper for message + optional stack trace content.",
            ],
            [
                "TestErrorMessage",
                "new(text)",
                "Primary failed-test copy block.",
            ],
            [
                "TestErrorStack",
                "new(text) | max_height(...)",
                "Scrollable monospace stack trace body.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_test_results_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::test_results_demo::render(cx);
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
    let composable = snippets::test_results_composable::render(cx);
    let props = parts_props_table(cx);
    let notes = crate::ui::doc_layout::notes_block([
        "Mechanism health looks good after the audit: the existing toggle diag covers the interaction path, and the remaining mismatch was a `fret-ui-ai` public-surface/docs-teaching gap rather than a `crates/fret-ui` runtime contract bug.",
        "This pass closes the biggest upstream drift by adding `children(...)` overrides to the summary, duration, progress, suite stats, and test leaf parts, so custom composition can stay compound-first instead of rebuilding the whole card.",
        "Fret keeps one intentional divergence: `Test::on_activate(...)` remains an explicit app-owned effect seam for editor-style \"open failing test output\" flows.",
        "This detail page is feature-gated behind `gallery-dev`, which also enables the wider `fret-ui-ai` surfaces in UI Gallery.",
    ]);
    let usage_section = DocSection::build(cx, "Usage", usage)
        .description("Rust/Fret analogue of the official AI Elements full Test Results preview.")
        .test_id_prefix("ui-gallery-ai-test-results-usage")
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
    let composable_section = DocSection::build(cx, "Composable Children", composable)
        .description(
            "Fret-specific custom composition lane: override only the leaf content you need while keeping the shared Test Results chrome and context wiring.",
        )
        .test_id_prefix("ui-gallery-ai-test-results-composable")
        .code_rust_from_file_region(snippets::test_results_composable::SOURCE, "example");
    let props_section = DocSection::build(cx, "Parts & Props", props)
        .description(
            "Official props-table coverage translated into Rust builder APIs and compound parts.",
        )
        .no_shell();
    let notes_section = DocSection::build(cx, "Notes", notes)
        .description("Parity findings, ownership, and the remaining Fret-specific seam.")
        .no_shell();

    let body = doc_layout::render_doc_page_after(
        Some(
            "The TestResults component displays test suite results including summary statistics, progress, individual tests, and error details.",
        ),
        vec![
            usage_section,
            features_section,
            status_colors_section,
            basic_section,
            suites_section,
            errors_section,
            composable_section,
            props_section,
            notes_section,
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
