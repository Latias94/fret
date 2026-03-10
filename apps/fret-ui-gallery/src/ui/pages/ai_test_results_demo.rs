use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

fn status_colors_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let row =
        |status: &'static str, color: &'static str, use_case: &'static str| {
            shadcn::TableRow::build(3, move |cx, out| {
                out.push_ui(cx, shadcn::TableCell::build(ui::text(status)));
                out.push_ui(cx, shadcn::TableCell::build(ui::text(color)));
                out.push_ui(cx, shadcn::TableCell::build(ui::text(use_case)));
            })
        };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(3, |cx, out| {
                        out.push(shadcn::TableHead::new("Status").into_element(cx));
                        out.push(shadcn::TableHead::new("Color").into_element(cx));
                        out.push(shadcn::TableHead::new("Use case").into_element(cx));
                    })
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("passed", "Green", "Test succeeded"));
                out.push_ui(cx, row("failed", "Red", "Test failed"));
                out.push_ui(cx, row("skipped", "Yellow", "Test skipped"));
                out.push_ui(cx, row("running", "Blue", "Test in progress"));
            }),
        );
    })
    .into_element(cx)
}

fn parts_props_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let row = |part: &'static str, inputs: &'static str, notes: &'static str| {
        shadcn::TableRow::build(3, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(part)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(inputs)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(notes)));
        })
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(3, |cx, out| {
                        out.push(shadcn::TableHead::new("Part").into_element(cx));
                        out.push(shadcn::TableHead::new("Key inputs").into_element(cx));
                        out.push(shadcn::TableHead::new("Notes").into_element(cx));
                    })
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("TestResults", "summary", "Root surface; summary-driven parts can read from the root provider."));
                out.push_ui(cx, row("TestResultsHeader", "children", "Header row with border-bottom + padding."));
                out.push_ui(cx, row("TestResultsSummary", "summary | from_context()", "Renders pass/fail/skip badges."));
                out.push_ui(cx, row("TestResultsDuration", "summary | from_context()", "Renders formatted duration when present."));
                out.push_ui(cx, row("TestResultsProgress", "summary | from_context()", "Progress bar + pass ratio labels."));
                out.push_ui(cx, row("TestResultsContent", "children", "Padded wrapper for suites and rows."));
                out.push_ui(cx, row("TestSuite", "new(trigger, content) | named(name, status)", "Collapsible suite shell; root can now provide context for trigger/content parts."));
                out.push_ui(cx, row("TestSuiteName", "new(name, status) | from_context() | children(...)", "Trigger row; can read suite context from the root and accept custom label children."));
                out.push_ui(cx, row("TestSuiteStats", "passed, failed, skipped", "Optional trailing stats helper for custom suite rows."));
                out.push_ui(cx, row("Test", "name, status, duration_ms, details, on_activate", "Row surface with optional error details and activation seam."));
                out.push_ui(cx, row("TestStatus / TestName / TestDuration", "new(...) | from_context()", "Composable row parts for custom `Test::children(...)` layouts."));
            }),
        );
    })
    .into_element(cx)
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
