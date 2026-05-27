//! Material-owned string lookup helpers.

use std::sync::Arc;

use fret_runtime::fret_i18n::{I18nService, MessageArgs, MessageKey};
use fret_ui::UiHost;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MaterialStringKey {
    TimePickerTitle,
    TimePickerDismiss,
    TimePickerCancel,
    TimePickerConfirm,
    TimePickerToggleInput,
    TimePickerToggleDial,
    TimePickerHourSelection,
    TimePickerMinuteSelection,
    TimePickerHourTextField,
    TimePickerMinuteTextField,
    TimePickerPeriodToggle,
    TimePickerPeriodAm,
    TimePickerPeriodPm,
    TimePickerHour,
    TimePickerMinute,
    TimePickerHourError12h,
    TimePickerHourError24h,
    TimePickerMinuteError,
    TimePickerHourValue12h,
    TimePickerHourValue24h,
    TimePickerMinuteValue,
}

impl MaterialStringKey {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::TimePickerTitle => "material3-time-picker-title",
            Self::TimePickerDismiss => "material3-time-picker-dismiss",
            Self::TimePickerCancel => "material3-time-picker-cancel",
            Self::TimePickerConfirm => "material3-time-picker-confirm",
            Self::TimePickerToggleInput => "material3-time-picker-toggle-input",
            Self::TimePickerToggleDial => "material3-time-picker-toggle-dial",
            Self::TimePickerHourSelection => "material3-time-picker-hour-selection",
            Self::TimePickerMinuteSelection => "material3-time-picker-minute-selection",
            Self::TimePickerHourTextField => "material3-time-picker-hour-text-field",
            Self::TimePickerMinuteTextField => "material3-time-picker-minute-text-field",
            Self::TimePickerPeriodToggle => "material3-time-picker-period-toggle",
            Self::TimePickerPeriodAm => "material3-time-picker-period-am",
            Self::TimePickerPeriodPm => "material3-time-picker-period-pm",
            Self::TimePickerHour => "material3-time-picker-hour",
            Self::TimePickerMinute => "material3-time-picker-minute",
            Self::TimePickerHourError12h => "material3-time-picker-hour-error-12h",
            Self::TimePickerHourError24h => "material3-time-picker-hour-error-24h",
            Self::TimePickerMinuteError => "material3-time-picker-minute-error",
            Self::TimePickerHourValue12h => "material3-time-picker-hour-value-12h",
            Self::TimePickerHourValue24h => "material3-time-picker-hour-value-24h",
            Self::TimePickerMinuteValue => "material3-time-picker-minute-value",
        }
    }
}

pub(crate) fn material_string<H: UiHost>(
    app: &H,
    key: MaterialStringKey,
    args: Option<&MessageArgs>,
    fallback: impl FnOnce() -> String,
) -> Arc<str> {
    if let Some(service) = app.global::<I18nService>() {
        let message_key = MessageKey::from(key.as_str());
        if let Ok(message) = service.format(&message_key, args) {
            return Arc::<str>::from(message.text);
        }
    }

    Arc::<str>::from(fallback())
}

pub(crate) fn material_time_picker_title<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerTitle, None, || {
        "Select time".to_string()
    })
}

pub(crate) fn material_time_picker_dismiss_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerDismiss, None, || {
        "Dismiss".to_string()
    })
}

pub(crate) fn material_time_picker_cancel_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerCancel, None, || {
        "Cancel".to_string()
    })
}

pub(crate) fn material_time_picker_confirm_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerConfirm, None, || {
        "OK".to_string()
    })
}

pub(crate) fn material_time_picker_toggle_input_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerToggleInput, None, || {
        "Switch to text input mode".to_string()
    })
}

pub(crate) fn material_time_picker_toggle_dial_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerToggleDial, None, || {
        "Switch to clock mode".to_string()
    })
}

pub(crate) fn material_time_picker_hour_selection_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        MaterialStringKey::TimePickerHourSelection,
        None,
        || "Select hour".to_string(),
    )
}

pub(crate) fn material_time_picker_minute_selection_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        MaterialStringKey::TimePickerMinuteSelection,
        None,
        || "Select minutes".to_string(),
    )
}

pub(crate) fn material_time_picker_hour_text_field_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        MaterialStringKey::TimePickerHourTextField,
        None,
        || "for hour".to_string(),
    )
}

pub(crate) fn material_time_picker_minute_text_field_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(
        app,
        MaterialStringKey::TimePickerMinuteTextField,
        None,
        || "for minutes".to_string(),
    )
}

pub(crate) fn material_time_picker_period_toggle_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerPeriodToggle, None, || {
        "Select AM or PM".to_string()
    })
}

pub(crate) fn material_time_picker_period_am_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerPeriodAm, None, || {
        "AM".to_string()
    })
}

pub(crate) fn material_time_picker_period_pm_label<H: UiHost>(app: &H) -> Arc<str> {
    material_string(app, MaterialStringKey::TimePickerPeriodPm, None, || {
        "PM".to_string()
    })
}

pub(crate) fn material_time_picker_hour_supporting_text<H: UiHost>(
    app: &H,
    is_24h: bool,
    is_valid: bool,
) -> Arc<str> {
    let key = match (is_24h, is_valid) {
        (_, true) => MaterialStringKey::TimePickerHour,
        (true, false) => MaterialStringKey::TimePickerHourError24h,
        (false, false) => MaterialStringKey::TimePickerHourError12h,
    };
    material_string(app, key, None, || match (is_24h, is_valid) {
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
        MaterialStringKey::TimePickerMinute
    } else {
        MaterialStringKey::TimePickerMinuteError
    };
    material_string(app, key, None, || {
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
        MaterialStringKey::TimePickerHourValue24h
    } else {
        MaterialStringKey::TimePickerHourValue12h
    };
    let args = MessageArgs::new().with("hour", hour as u64);
    material_string(app, key, Some(&args), || {
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
        MaterialStringKey::TimePickerMinuteValue,
        Some(&args),
        || format!("{minute} minutes"),
    )
}
