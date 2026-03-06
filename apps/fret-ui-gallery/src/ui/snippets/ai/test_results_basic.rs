pub const SOURCE: &str = include_str!("test_results_basic.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let summary = ui_ai::TestResultsSummaryData::new(10, 2, 1, 13).duration_ms(3500);

    ui_ai::TestResults::new()
        .summary(summary.clone())
        .children([ui_ai::TestResultsHeader::new([
            ui_ai::TestResultsSummary::new(summary.clone()).into_element(cx),
            ui_ai::TestResultsDuration::new(summary).into_element(cx),
        ])
        .into_element(cx)])
        .into_element(cx)
}
// endregion: example
