use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

fn status_colors_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let header = shadcn::TableHeader::new([shadcn::TableRow::new(
        3,
        [
            shadcn::TableHead::new("Status").into_element(cx),
            shadcn::TableHead::new("Color").into_element(cx),
            shadcn::TableHead::new("Use case").into_element(cx),
        ],
    )
    .into_element(cx)])
    .into_element(cx);

    let body = shadcn::TableBody::new([
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("passed")).into_element(cx),
                shadcn::TableCell::new(cx.text("Green")).into_element(cx),
                shadcn::TableCell::new(cx.text("Test succeeded")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("failed")).into_element(cx),
                shadcn::TableCell::new(cx.text("Red")).into_element(cx),
                shadcn::TableCell::new(cx.text("Test failed")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("skipped")).into_element(cx),
                shadcn::TableCell::new(cx.text("Yellow")).into_element(cx),
                shadcn::TableCell::new(cx.text("Test skipped")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("running")).into_element(cx),
                shadcn::TableCell::new(cx.text("Blue")).into_element(cx),
                shadcn::TableCell::new(cx.text("Test in progress")).into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Table::new([header, body]).into_element(cx)
}

fn parts_props_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let header = shadcn::TableHeader::new([shadcn::TableRow::new(
        3,
        [
            shadcn::TableHead::new("Part").into_element(cx),
            shadcn::TableHead::new("Key inputs").into_element(cx),
            shadcn::TableHead::new("Notes").into_element(cx),
        ],
    )
    .into_element(cx)])
    .into_element(cx);

    let body = shadcn::TableBody::new([
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestResults")).into_element(cx),
                shadcn::TableCell::new(cx.text("summary")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Root surface; summary-driven parts can read from the root provider."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestResultsHeader")).into_element(cx),
                shadcn::TableCell::new(cx.text("children")).into_element(cx),
                shadcn::TableCell::new(cx.text("Header row with border-bottom + padding."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestResultsSummary")).into_element(cx),
                shadcn::TableCell::new(cx.text("summary | from_context()")).into_element(cx),
                shadcn::TableCell::new(cx.text("Renders pass/fail/skip badges.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestResultsDuration")).into_element(cx),
                shadcn::TableCell::new(cx.text("summary | from_context()")).into_element(cx),
                shadcn::TableCell::new(cx.text("Renders formatted duration when present."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestResultsProgress")).into_element(cx),
                shadcn::TableCell::new(cx.text("summary | from_context()")).into_element(cx),
                shadcn::TableCell::new(cx.text("Progress bar + pass ratio labels."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestResultsContent")).into_element(cx),
                shadcn::TableCell::new(cx.text("children")).into_element(cx),
                shadcn::TableCell::new(cx.text("Padded wrapper for suites and rows."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestSuite")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("new(trigger, content) | named(name, status)"),
                )
                .into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Collapsible suite shell; root can now provide context for trigger/content parts."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestSuiteName")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("new(name, status) | from_context() | children(...)"),
                )
                .into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Trigger row; can read suite context from the root and accept custom label children."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestSuiteStats")).into_element(cx),
                shadcn::TableCell::new(cx.text("passed, failed, skipped")).into_element(cx),
                shadcn::TableCell::new(cx.text("Optional trailing stats helper for custom suite rows."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("Test")).into_element(cx),
                shadcn::TableCell::new(cx.text("name, status, duration_ms, details, on_activate"))
                    .into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Row surface with optional error details and activation seam."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("TestStatus / TestName / TestDuration"))
                    .into_element(cx),
                shadcn::TableCell::new(
                    cx.text("new(...) | from_context()"),
                )
                .into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Composable row parts for custom `Test::children(...)` layouts."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Table::new([header, body]).into_element(cx)
}

pub(super) fn preview_ai_test_results_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let overview = snippets::test_results_demo::render(cx);
    let features = crate::ui::doc_layout::notes(
        cx,
        [
            "Summary statistics (passed/failed/skipped)",
            "Progress bar visualization",
            "Collapsible test suites",
            "Individual test status and duration",
            "Error messages with stack traces",
            "Color-coded status indicators",
        ],
    );
    let status_colors = status_colors_table(cx);
    let basic = snippets::test_results_basic::render(cx);
    let suites = snippets::test_results_suites::render(cx);
    let errors = snippets::test_results_errors::render(cx);
    let props = parts_props_table(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "The TestResults component displays test suite results including summary statistics, progress, individual tests, and error details.",
        ),
        vec![
            DocSection::new("Overview", overview)
                .description("Rust/Fret analogue of the official AI Elements all-in-one preview.")
                .test_id_prefix("ui-gallery-ai-test-results-overview")
                .code_rust_from_file_region(snippets::test_results_demo::SOURCE, "example"),
            DocSection::new("Features", features).no_shell(),
            DocSection::new("Status Colors", status_colors).no_shell(),
            DocSection::new("Basic Usage", basic)
                .description(
                    "Summary badges and duration only, matching the official basic example.",
                )
                .test_id_prefix("ui-gallery-ai-test-results-basic")
                .code_rust_from_file_region(snippets::test_results_basic::SOURCE, "example"),
            DocSection::new("With Test Suites", suites)
                .description(
                    "Expandable suites with individual test rows, matching the docs sample.",
                )
                .test_id_prefix("ui-gallery-ai-test-results-suites")
                .code_rust_from_file_region(snippets::test_results_suites::SOURCE, "example"),
            DocSection::new("With Error Details", errors)
                .description(
                    "Failed tests render inline error panels and stack traces for copyable repros.",
                )
                .test_id_prefix("ui-gallery-ai-test-results-errors")
                .code_rust_from_file_region(snippets::test_results_errors::SOURCE, "example"),
            DocSection::new("Parts & Props", props).no_shell(),
        ],
    );

    vec![body]
}
