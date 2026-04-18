pub const SOURCE: &str = include_str!("test_results_basic.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let summary = ui_ai::TestResultsSummaryData::new(10, 2, 1, 13).duration_ms(3500);

    ui_ai::TestResults::new()
        .summary(summary)
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::TestResultsHeader::new([
                    ui_ai::TestResultsSummary::from_context().into_element(cx),
                    ui_ai::TestResultsDuration::from_context().into_element(cx),
                ])
                .into_element(cx),
            ]
        })
}
// endregion: example
