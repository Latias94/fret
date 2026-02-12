use super::super::super::super::super::*;

use fret_ui_headless::calendar::DateRangeSelection;

#[derive(Default, Clone)]
struct CalendarModels {
    caption_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
    caption_selected: Option<Model<Option<Date>>>,
    range_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
    range_selected: Option<Model<DateRangeSelection>>,
    presets_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
    presets_selected: Option<Model<Option<Date>>>,
    time_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
    time_selected: Option<Model<Option<Date>>>,
    time_from: Option<Model<String>>,
    time_to: Option<Model<String>>,
    booked_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
    booked_selected: Option<Model<Option<Date>>>,
    custom_cell_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
    custom_cell_selected: Option<Model<Option<Date>>>,
    week_number_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
    week_number_selected: Option<Model<Option<Date>>>,
    rtl_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
    rtl_selected: Option<Model<Option<Date>>>,
}

#[derive(Clone)]
pub(super) struct CalendarHandles {
    pub(super) month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) selected: Model<Option<Date>>,
    pub(super) caption_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) caption_selected: Model<Option<Date>>,
    pub(super) range_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) range_selected: Model<DateRangeSelection>,
    pub(super) presets_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) presets_selected: Model<Option<Date>>,
    pub(super) time_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) time_selected: Model<Option<Date>>,
    pub(super) time_from: Model<String>,
    pub(super) time_to: Model<String>,
    pub(super) booked_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) booked_selected: Model<Option<Date>>,
    pub(super) custom_cell_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) custom_cell_selected: Model<Option<Date>>,
    pub(super) week_number_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) week_number_selected: Model<Option<Date>>,
    pub(super) rtl_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    pub(super) rtl_selected: Model<Option<Date>>,
}

pub(super) fn get_or_init(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
    today: Date,
) -> CalendarHandles {
    let initial_month = cx
        .get_model_copied(&month, Invalidation::Layout)
        .unwrap_or_else(|| fret_ui_headless::calendar::CalendarMonth::from_date(today));

    let state = cx.with_state(CalendarModels::default, |st| st.clone());

    let caption_month = match state.caption_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.caption_month = Some(model.clone())
            });
            model
        }
    };
    let caption_selected = match state.caption_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.caption_selected = Some(model.clone())
            });
            model
        }
    };

    let range_month = match state.range_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.range_month = Some(model.clone())
            });
            model
        }
    };
    let range_selected = match state.range_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(DateRangeSelection::default());
            cx.with_state(CalendarModels::default, |st| {
                st.range_selected = Some(model.clone())
            });
            model
        }
    };

    let preset_date = time::Date::from_calendar_date(today.year(), time::Month::February, 12)
        .expect("valid preset date");
    let presets_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(preset_date);
    let presets_month = match state.presets_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(presets_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.presets_month = Some(model.clone())
            });
            model
        }
    };
    let presets_selected = match state.presets_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(preset_date));
            cx.with_state(CalendarModels::default, |st| {
                st.presets_selected = Some(model.clone())
            });
            model
        }
    };

    let time_date = time::Date::from_calendar_date(today.year(), today.month(), 12)
        .expect("valid time picker date");
    let time_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(time_date);
    let time_month = match state.time_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(time_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.time_month = Some(model.clone())
            });
            model
        }
    };
    let time_selected = match state.time_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(time_date));
            cx.with_state(CalendarModels::default, |st| {
                st.time_selected = Some(model.clone())
            });
            model
        }
    };
    let time_from = match state.time_from {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("10:30:00"));
            cx.with_state(CalendarModels::default, |st| {
                st.time_from = Some(model.clone())
            });
            model
        }
    };
    let time_to = match state.time_to {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("12:30:00"));
            cx.with_state(CalendarModels::default, |st| {
                st.time_to = Some(model.clone())
            });
            model
        }
    };

    let booked_month = match state.booked_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.booked_month = Some(model.clone())
            });
            model
        }
    };
    let booked_selected = match state.booked_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.booked_selected = Some(model.clone())
            });
            model
        }
    };

    let custom_cell_month = match state.custom_cell_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.custom_cell_month = Some(model.clone())
            });
            model
        }
    };
    let custom_cell_selected = match state.custom_cell_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.custom_cell_selected = Some(model.clone())
            });
            model
        }
    };

    let week_number_month = match state.week_number_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.week_number_month = Some(model.clone())
            });
            model
        }
    };
    let week_number_selected = match state.week_number_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.week_number_selected = Some(model.clone())
            });
            model
        }
    };

    let rtl_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(today);
    let rtl_month = match state.rtl_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(rtl_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.rtl_month = Some(model.clone())
            });
            model
        }
    };
    let rtl_selected = match state.rtl_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(today));
            cx.with_state(CalendarModels::default, |st| {
                st.rtl_selected = Some(model.clone())
            });
            model
        }
    };

    CalendarHandles {
        month,
        selected,
        caption_month,
        caption_selected,
        range_month,
        range_selected,
        presets_month,
        presets_selected,
        time_month,
        time_selected,
        time_from,
        time_to,
        booked_month,
        booked_selected,
        custom_cell_month,
        custom_cell_selected,
        week_number_month,
        week_number_selected,
        rtl_month,
        rtl_selected,
    }
}
