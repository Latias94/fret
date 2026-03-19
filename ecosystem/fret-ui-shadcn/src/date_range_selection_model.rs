use fret_runtime::Model;
use fret_ui_headless::calendar::DateRangeSelection;

/// Narrow interop bridge for range-selection widgets that store their value in a
/// `Model<DateRangeSelection>`.
pub trait IntoDateRangeSelectionModel {
    fn into_date_range_selection_model(self) -> Model<DateRangeSelection>;
}

impl IntoDateRangeSelectionModel for Model<DateRangeSelection> {
    fn into_date_range_selection_model(self) -> Model<DateRangeSelection> {
        self
    }
}

impl IntoDateRangeSelectionModel for &Model<DateRangeSelection> {
    fn into_date_range_selection_model(self) -> Model<DateRangeSelection> {
        self.clone()
    }
}
