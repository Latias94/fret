use fret_runtime::Model;
use fret_ui_headless::calendar::CalendarMonth;

/// Narrow interop bridge for calendar-like widgets that store their visible month in a
/// `Model<CalendarMonth>`.
///
/// This keeps the public authoring surface focused on the current high-frequency date/calendar
/// path without introducing a generic `IntoModel<T>` abstraction across the whole crate.
pub trait IntoCalendarMonthModel {
    fn into_calendar_month_model(self) -> Model<CalendarMonth>;
}

impl IntoCalendarMonthModel for Model<CalendarMonth> {
    fn into_calendar_month_model(self) -> Model<CalendarMonth> {
        self
    }
}

impl IntoCalendarMonthModel for &Model<CalendarMonth> {
    fn into_calendar_month_model(self) -> Model<CalendarMonth> {
        self.clone()
    }
}
