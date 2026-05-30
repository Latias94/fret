//! Material-owned string lookup helpers.

use std::sync::Arc;

use fret_runtime::fret_i18n::{I18nService, MessageArgs, MessageKey};
use fret_ui::UiHost;
use time::{Date, Month, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchBarStringKey {
    Search,
    SuggestionsAvailable,
}

impl SearchBarStringKey {
    fn as_str(self) -> &'static str {
        match self {
            Self::Search => "material3-search-bar-search",
            Self::SuggestionsAvailable => "material3-search-bar-suggestions-available",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimePickerStringKey {
    Title,
    Dismiss,
    Cancel,
    Confirm,
    ToggleInput,
    ToggleDial,
    HourSelection,
    MinuteSelection,
    HourTextField,
    MinuteTextField,
    PeriodToggle,
    PeriodAm,
    PeriodPm,
    Hour,
    Minute,
    HourError12h,
    HourError24h,
    MinuteError,
    HourValue12h,
    HourValue24h,
    MinuteValue,
}

impl TimePickerStringKey {
    fn as_str(self) -> &'static str {
        match self {
            Self::Title => "material3-time-picker-title",
            Self::Dismiss => "material3-time-picker-dismiss",
            Self::Cancel => "material3-time-picker-cancel",
            Self::Confirm => "material3-time-picker-confirm",
            Self::ToggleInput => "material3-time-picker-toggle-input",
            Self::ToggleDial => "material3-time-picker-toggle-dial",
            Self::HourSelection => "material3-time-picker-hour-selection",
            Self::MinuteSelection => "material3-time-picker-minute-selection",
            Self::HourTextField => "material3-time-picker-hour-text-field",
            Self::MinuteTextField => "material3-time-picker-minute-text-field",
            Self::PeriodToggle => "material3-time-picker-period-toggle",
            Self::PeriodAm => "material3-time-picker-period-am",
            Self::PeriodPm => "material3-time-picker-period-pm",
            Self::Hour => "material3-time-picker-hour",
            Self::Minute => "material3-time-picker-minute",
            Self::HourError12h => "material3-time-picker-hour-error-12h",
            Self::HourError24h => "material3-time-picker-hour-error-24h",
            Self::MinuteError => "material3-time-picker-minute-error",
            Self::HourValue12h => "material3-time-picker-hour-value-12h",
            Self::HourValue24h => "material3-time-picker-hour-value-24h",
            Self::MinuteValue => "material3-time-picker-minute-value",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatePickerStringKey {
    Title,
    Dismiss,
    Cancel,
    Confirm,
    PreviousMonth,
    NextMonth,
    PreviousMonthShort,
    NextMonthShort,
    MonthYear,
    DayDescription,
    TodayDescription,
    TodayDateDescription,
    MonthJanuary,
    MonthFebruary,
    MonthMarch,
    MonthApril,
    MonthMay,
    MonthJune,
    MonthJuly,
    MonthAugust,
    MonthSeptember,
    MonthOctober,
    MonthNovember,
    MonthDecember,
    WeekdayShortMonday,
    WeekdayShortTuesday,
    WeekdayShortWednesday,
    WeekdayShortThursday,
    WeekdayShortFriday,
    WeekdayShortSaturday,
    WeekdayShortSunday,
    WeekdayLongMonday,
    WeekdayLongTuesday,
    WeekdayLongWednesday,
    WeekdayLongThursday,
    WeekdayLongFriday,
    WeekdayLongSaturday,
    WeekdayLongSunday,
}

impl DatePickerStringKey {
    fn as_str(self) -> &'static str {
        match self {
            Self::Title => "material3-date-picker-title",
            Self::Dismiss => "material3-date-picker-dismiss",
            Self::Cancel => "material3-date-picker-cancel",
            Self::Confirm => "material3-date-picker-confirm",
            Self::PreviousMonth => "material3-date-picker-previous-month",
            Self::NextMonth => "material3-date-picker-next-month",
            Self::PreviousMonthShort => "material3-date-picker-previous-month-short",
            Self::NextMonthShort => "material3-date-picker-next-month-short",
            Self::MonthYear => "material3-date-picker-month-year",
            Self::DayDescription => "material3-date-picker-day-description",
            Self::TodayDescription => "material3-date-picker-today-description",
            Self::TodayDateDescription => "material3-date-picker-today-date-description",
            Self::MonthJanuary => "material3-date-picker-month-january",
            Self::MonthFebruary => "material3-date-picker-month-february",
            Self::MonthMarch => "material3-date-picker-month-march",
            Self::MonthApril => "material3-date-picker-month-april",
            Self::MonthMay => "material3-date-picker-month-may",
            Self::MonthJune => "material3-date-picker-month-june",
            Self::MonthJuly => "material3-date-picker-month-july",
            Self::MonthAugust => "material3-date-picker-month-august",
            Self::MonthSeptember => "material3-date-picker-month-september",
            Self::MonthOctober => "material3-date-picker-month-october",
            Self::MonthNovember => "material3-date-picker-month-november",
            Self::MonthDecember => "material3-date-picker-month-december",
            Self::WeekdayShortMonday => "material3-date-picker-weekday-short-monday",
            Self::WeekdayShortTuesday => "material3-date-picker-weekday-short-tuesday",
            Self::WeekdayShortWednesday => "material3-date-picker-weekday-short-wednesday",
            Self::WeekdayShortThursday => "material3-date-picker-weekday-short-thursday",
            Self::WeekdayShortFriday => "material3-date-picker-weekday-short-friday",
            Self::WeekdayShortSaturday => "material3-date-picker-weekday-short-saturday",
            Self::WeekdayShortSunday => "material3-date-picker-weekday-short-sunday",
            Self::WeekdayLongMonday => "material3-date-picker-weekday-long-monday",
            Self::WeekdayLongTuesday => "material3-date-picker-weekday-long-tuesday",
            Self::WeekdayLongWednesday => "material3-date-picker-weekday-long-wednesday",
            Self::WeekdayLongThursday => "material3-date-picker-weekday-long-thursday",
            Self::WeekdayLongFriday => "material3-date-picker-weekday-long-friday",
            Self::WeekdayLongSaturday => "material3-date-picker-weekday-long-saturday",
            Self::WeekdayLongSunday => "material3-date-picker-weekday-long-sunday",
        }
    }
}

fn material_string<H: UiHost>(
    app: &H,
    key: &'static str,
    args: Option<&MessageArgs>,
    fallback: impl FnOnce() -> String,
) -> Arc<str> {
    if let Some(service) = app.global::<I18nService>() {
        let message_key = MessageKey::from(key);
        if let Ok(message) = service.format(&message_key, args) {
            return Arc::<str>::from(message.text);
        }
    }

    Arc::<str>::from(fallback())
}

pub(crate) fn material_search_bar_search_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, SearchBarStringKey::Search.as_str(), None, || {
        "Search".to_string()
    })
}

pub(crate) fn material_search_bar_suggestions_available_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        SearchBarStringKey::SuggestionsAvailable.as_str(),
        None,
        || "Suggestions below".to_string(),
    )
}

pub(crate) fn material_time_picker_title<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, TimePickerStringKey::Title.as_str(), None, || {
        "Select time".to_string()
    })
}

pub(crate) fn material_time_picker_dismiss_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, TimePickerStringKey::Dismiss.as_str(), None, || {
        "Dismiss".to_string()
    })
}

pub(crate) fn material_time_picker_cancel_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, TimePickerStringKey::Cancel.as_str(), None, || {
        "Cancel".to_string()
    })
}

pub(crate) fn material_time_picker_confirm_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, TimePickerStringKey::Confirm.as_str(), None, || {
        "OK".to_string()
    })
}

pub(crate) fn material_time_picker_toggle_input_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, TimePickerStringKey::ToggleInput.as_str(), None, || {
        "Switch to text input mode".to_string()
    })
}

pub(crate) fn material_time_picker_toggle_dial_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, TimePickerStringKey::ToggleDial.as_str(), None, || {
        "Switch to clock mode".to_string()
    })
}

pub(crate) fn material_time_picker_hour_selection_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        TimePickerStringKey::HourSelection.as_str(),
        None,
        || "Select hour".to_string(),
    )
}

pub(crate) fn material_time_picker_minute_selection_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        TimePickerStringKey::MinuteSelection.as_str(),
        None,
        || "Select minutes".to_string(),
    )
}

pub(crate) fn material_time_picker_hour_text_field_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        TimePickerStringKey::HourTextField.as_str(),
        None,
        || "for hour".to_string(),
    )
}

pub(crate) fn material_time_picker_minute_text_field_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        TimePickerStringKey::MinuteTextField.as_str(),
        None,
        || "for minutes".to_string(),
    )
}

pub(crate) fn material_time_picker_period_toggle_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        TimePickerStringKey::PeriodToggle.as_str(),
        None,
        || "Select AM or PM".to_string(),
    )
}

pub(crate) fn material_time_picker_period_am_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, TimePickerStringKey::PeriodAm.as_str(), None, || {
        "AM".to_string()
    })
}

pub(crate) fn material_time_picker_period_pm_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, TimePickerStringKey::PeriodPm.as_str(), None, || {
        "PM".to_string()
    })
}

pub(crate) fn material_time_picker_hour_supporting_text<H: UiHost>(
    app: &H,
    is_24h: bool,
    is_valid: bool,
) -> Arc<str> {
    let key = match (is_24h, is_valid) {
        (_, true) => TimePickerStringKey::Hour,
        (true, false) => TimePickerStringKey::HourError24h,
        (false, false) => TimePickerStringKey::HourError12h,
    };
    material_string(app, key.as_str(), None, || match (is_24h, is_valid) {
        (_, true) => "Hour".to_string(),
        (true, false) => "Hour must be 0-23".to_string(),
        (false, false) => "Hour must be 1-12".to_string(),
    })
}

pub(crate) fn material_time_picker_minute_supporting_text<H: UiHost>(
    app: &H,
    is_valid: bool,
) -> Arc<str> {
    let key = if is_valid {
        TimePickerStringKey::Minute
    } else {
        TimePickerStringKey::MinuteError
    };
    material_string(app, key.as_str(), None, || {
        if is_valid {
            "Minute".to_string()
        } else {
            "Minute must be 0-59".to_string()
        }
    })
}

pub(crate) fn material_time_picker_hour_value_description<H: UiHost>(
    app: &H,
    hour: u32,
    is_24h: bool,
) -> Arc<str> {
    let key = if is_24h {
        TimePickerStringKey::HourValue24h
    } else {
        TimePickerStringKey::HourValue12h
    };
    let args = MessageArgs::new().with("hour", hour as u64);
    material_string(app, key.as_str(), Some(&args), || {
        if is_24h {
            format!("{hour} hours")
        } else {
            format!("{hour} o'clock")
        }
    })
}

pub(crate) fn material_time_picker_minute_value_description<H: UiHost>(
    app: &H,
    minute: u32,
) -> Arc<str> {
    let args = MessageArgs::new().with("minute", minute as u64);
    material_string(
        app,
        TimePickerStringKey::MinuteValue.as_str(),
        Some(&args),
        || format!("{minute} minutes"),
    )
}

pub(crate) fn material_date_picker_title<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, DatePickerStringKey::Title.as_str(), None, || {
        "Select date".to_string()
    })
}

pub(crate) fn material_date_picker_dismiss_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, DatePickerStringKey::Dismiss.as_str(), None, || {
        "Dismiss".to_string()
    })
}

pub(crate) fn material_date_picker_cancel_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, DatePickerStringKey::Cancel.as_str(), None, || {
        "Cancel".to_string()
    })
}

pub(crate) fn material_date_picker_confirm_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, DatePickerStringKey::Confirm.as_str(), None, || {
        "OK".to_string()
    })
}

pub(crate) fn material_date_picker_previous_month_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        DatePickerStringKey::PreviousMonth.as_str(),
        None,
        || "Switch to previous month".to_string(),
    )
}

pub(crate) fn material_date_picker_next_month_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, DatePickerStringKey::NextMonth.as_str(), None, || {
        "Switch to next month".to_string()
    })
}

pub(crate) fn material_date_picker_previous_month_short_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        DatePickerStringKey::PreviousMonthShort.as_str(),
        None,
        || "Prev".to_string(),
    )
}

pub(crate) fn material_date_picker_next_month_short_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        DatePickerStringKey::NextMonthShort.as_str(),
        None,
        || "Next".to_string(),
    )
}

pub(crate) fn material_date_picker_month_year<H: UiHost>(
    app: &H,
    month: Month,
    year: i32,
) -> Arc<str> {
    let month_name = material_date_picker_month_name(app, month);
    let args = MessageArgs::new()
        .with("month", month_name.as_ref())
        .with("month_number", u64::from(u8::from(month)))
        .with("year", i64::from(year));
    material_string(
        app,
        DatePickerStringKey::MonthYear.as_str(),
        Some(&args),
        || format!("{month_name} {year}"),
    )
}

pub(crate) fn material_date_picker_day_description<H: UiHost>(
    app: &H,
    date: Date,
    is_today: bool,
) -> Arc<str> {
    let month_name = material_date_picker_month_name(app, date.month());
    let args = MessageArgs::new()
        .with("month", month_name.as_ref())
        .with("month_number", u64::from(u8::from(date.month())))
        .with("day", u64::from(date.day()))
        .with("year", i64::from(date.year()));
    let date_description = material_string(
        app,
        DatePickerStringKey::DayDescription.as_str(),
        Some(&args),
        || format!("{month_name} {}, {}", date.day(), date.year()),
    );

    if !is_today {
        return date_description;
    }

    let today = material_string(
        app,
        DatePickerStringKey::TodayDescription.as_str(),
        None,
        || "Today".to_string(),
    );
    let args = MessageArgs::new()
        .with("today", today.as_ref())
        .with("date", date_description.as_ref());
    material_string(
        app,
        DatePickerStringKey::TodayDateDescription.as_str(),
        Some(&args),
        || format!("{today}, {date_description}"),
    )
}

pub(crate) fn material_date_picker_weekday_short_label<H: UiHost>(
    app: &H,
    weekday: Weekday,
) -> Arc<str> {
    let key = match weekday {
        Weekday::Monday => DatePickerStringKey::WeekdayShortMonday,
        Weekday::Tuesday => DatePickerStringKey::WeekdayShortTuesday,
        Weekday::Wednesday => DatePickerStringKey::WeekdayShortWednesday,
        Weekday::Thursday => DatePickerStringKey::WeekdayShortThursday,
        Weekday::Friday => DatePickerStringKey::WeekdayShortFriday,
        Weekday::Saturday => DatePickerStringKey::WeekdayShortSaturday,
        Weekday::Sunday => DatePickerStringKey::WeekdayShortSunday,
    };
    material_string(app, key.as_str(), None, || {
        weekday_short_en(weekday).to_string()
    })
}

pub(crate) fn material_date_picker_weekday_long_label<H: UiHost>(
    app: &H,
    weekday: Weekday,
) -> Arc<str> {
    let key = match weekday {
        Weekday::Monday => DatePickerStringKey::WeekdayLongMonday,
        Weekday::Tuesday => DatePickerStringKey::WeekdayLongTuesday,
        Weekday::Wednesday => DatePickerStringKey::WeekdayLongWednesday,
        Weekday::Thursday => DatePickerStringKey::WeekdayLongThursday,
        Weekday::Friday => DatePickerStringKey::WeekdayLongFriday,
        Weekday::Saturday => DatePickerStringKey::WeekdayLongSaturday,
        Weekday::Sunday => DatePickerStringKey::WeekdayLongSunday,
    };
    material_string(app, key.as_str(), None, || {
        weekday_long_en(weekday).to_string()
    })
}

fn material_date_picker_month_name<H: UiHost>(app: &H, month: Month) -> Arc<str> {
    let key = match month {
        Month::January => DatePickerStringKey::MonthJanuary,
        Month::February => DatePickerStringKey::MonthFebruary,
        Month::March => DatePickerStringKey::MonthMarch,
        Month::April => DatePickerStringKey::MonthApril,
        Month::May => DatePickerStringKey::MonthMay,
        Month::June => DatePickerStringKey::MonthJune,
        Month::July => DatePickerStringKey::MonthJuly,
        Month::August => DatePickerStringKey::MonthAugust,
        Month::September => DatePickerStringKey::MonthSeptember,
        Month::October => DatePickerStringKey::MonthOctober,
        Month::November => DatePickerStringKey::MonthNovember,
        Month::December => DatePickerStringKey::MonthDecember,
    };
    material_string(app, key.as_str(), None, || month_name_en(month).to_string())
}

fn month_name_en(month: Month) -> &'static str {
    match month {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    }
}

fn weekday_short_en(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "Mo",
        Weekday::Tuesday => "Tu",
        Weekday::Wednesday => "We",
        Weekday::Thursday => "Th",
        Weekday::Friday => "Fr",
        Weekday::Saturday => "Sa",
        Weekday::Sunday => "Su",
    }
}

fn weekday_long_en(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    }
}
