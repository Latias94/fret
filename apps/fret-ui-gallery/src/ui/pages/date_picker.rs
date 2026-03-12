use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::date_picker as snippets;
use fret::UiCx;

use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

pub(super) fn preview_date_picker(
    cx: &mut UiCx<'_>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct DatePickerModels {
        basic_open: Option<Model<bool>>,
        basic_month: Option<Model<CalendarMonth>>,
        basic_selected: Option<Model<Option<Date>>>,
        range_open: Option<Model<bool>>,
        range_month: Option<Model<CalendarMonth>>,
        range_selected: Option<Model<DateRangeSelection>>,
        dob_open: Option<Model<bool>>,
        dob_month: Option<Model<CalendarMonth>>,
        dob_selected: Option<Model<Option<Date>>>,
        rtl_open: Option<Model<bool>>,
        rtl_month: Option<Model<CalendarMonth>>,
        rtl_selected: Option<Model<Option<Date>>>,
        presets_open: Option<Model<bool>>,
        presets_month: Option<Model<CalendarMonth>>,
        presets_selected: Option<Model<Option<Date>>>,
        input_open: Option<Model<bool>>,
        input_month: Option<Model<CalendarMonth>>,
        input_selected: Option<Model<Option<Date>>>,
        input_value: Option<Model<String>>,
        input_last_selected: Option<Date>,
        natural_open: Option<Model<bool>>,
        natural_month: Option<Model<CalendarMonth>>,
        natural_selected: Option<Model<Option<Date>>>,
        natural_value: Option<Model<String>>,
        natural_last_selected: Option<Date>,
        time_open: Option<Model<bool>>,
        time_month: Option<Model<CalendarMonth>>,
        time_selected: Option<Model<Option<Date>>>,
        time_value: Option<Model<String>>,
    }

    fn parse_iso_date_ymd(raw: &str) -> Option<Date> {
        let raw = raw.trim();
        let (year, rest) = raw.split_once('-')?;
        let (month, day) = rest.split_once('-')?;

        let year: i32 = year.parse().ok()?;
        let month: u8 = month.parse().ok()?;
        let day: u8 = day.parse().ok()?;

        let month = time::Month::try_from(month).ok()?;
        Date::from_calendar_date(year, month, day).ok()
    }

    let diag_calendar_roving =
        std::env::var_os("FRET_UI_GALLERY_DIAG_CALENDAR_ROVING").is_some_and(|v| !v.is_empty());

    let today = std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());

    let (
        basic_open,
        basic_month,
        basic_selected,
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
        presets_open,
        presets_month,
        presets_selected,
        input_open,
        input_month,
        input_selected,
        input_value,
        natural_open,
        natural_month,
        natural_selected,
        natural_value,
        time_open,
        time_month,
        time_selected,
        time_value,
    ) = cx.with_state(DatePickerModels::default, |st| {
        (
            st.basic_open.clone(),
            st.basic_month.clone(),
            st.basic_selected.clone(),
            st.range_open.clone(),
            st.range_month.clone(),
            st.range_selected.clone(),
            st.dob_open.clone(),
            st.dob_month.clone(),
            st.dob_selected.clone(),
            st.rtl_open.clone(),
            st.rtl_month.clone(),
            st.rtl_selected.clone(),
            st.presets_open.clone(),
            st.presets_month.clone(),
            st.presets_selected.clone(),
            st.input_open.clone(),
            st.input_month.clone(),
            st.input_selected.clone(),
            st.input_value.clone(),
            st.natural_open.clone(),
            st.natural_month.clone(),
            st.natural_selected.clone(),
            st.natural_value.clone(),
            st.time_open.clone(),
            st.time_month.clone(),
            st.time_selected.clone(),
            st.time_value.clone(),
        )
    });

    let (
        basic_open,
        basic_month,
        basic_selected,
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
        presets_open,
        presets_month,
        presets_selected,
        input_open,
        input_month,
        input_selected,
        input_value,
        natural_open,
        natural_month,
        natural_selected,
        natural_value,
        time_open,
        time_month,
        time_selected,
        time_value,
    ) = match (
        basic_open,
        basic_month,
        basic_selected,
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
        presets_open,
        presets_month,
        presets_selected,
        input_open,
        input_month,
        input_selected,
        input_value,
        natural_open,
        natural_month,
        natural_selected,
        natural_value,
        time_open,
        time_month,
        time_selected,
        time_value,
    ) {
        (
            Some(basic_open),
            Some(basic_month),
            Some(basic_selected),
            Some(range_open),
            Some(range_month),
            Some(range_selected),
            Some(dob_open),
            Some(dob_month),
            Some(dob_selected),
            Some(rtl_open),
            Some(rtl_month),
            Some(rtl_selected),
            Some(presets_open),
            Some(presets_month),
            Some(presets_selected),
            Some(input_open),
            Some(input_month),
            Some(input_selected),
            Some(input_value),
            Some(natural_open),
            Some(natural_month),
            Some(natural_selected),
            Some(natural_value),
            Some(time_open),
            Some(time_month),
            Some(time_selected),
            Some(time_value),
        ) => (
            basic_open,
            basic_month,
            basic_selected,
            range_open,
            range_month,
            range_selected,
            dob_open,
            dob_month,
            dob_selected,
            rtl_open,
            rtl_month,
            rtl_selected,
            presets_open,
            presets_month,
            presets_selected,
            input_open,
            input_month,
            input_selected,
            input_value,
            natural_open,
            natural_month,
            natural_selected,
            natural_value,
            time_open,
            time_month,
            time_selected,
            time_value,
        ),
        _ => {
            let basic_open = cx.app.models_mut().insert(false);
            let range_open = cx.app.models_mut().insert(false);
            let diag_month = CalendarMonth::from_date(
                Date::from_calendar_date(2024, time::Month::February, 1).expect("valid date"),
            );
            let basic_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let basic_selected = cx.app.models_mut().insert(None::<Date>);
            let range_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let range_selected = cx.app.models_mut().insert(if diag_calendar_roving {
                DateRangeSelection::default()
            } else {
                DateRangeSelection::default()
            });
            let dob_open = cx.app.models_mut().insert(false);
            let dob_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let dob_selected = cx.app.models_mut().insert(None::<Date>);
            let rtl_open = cx.app.models_mut().insert(false);
            let rtl_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let rtl_selected = cx.app.models_mut().insert(Some(today));

            let presets_open = cx.app.models_mut().insert(false);
            let presets_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            let presets_selected = cx.app.models_mut().insert(None::<Date>);

            let input_open = cx.app.models_mut().insert(false);
            let input_seed = Date::from_calendar_date(2025, time::Month::June, 1).expect("date");
            let input_month = cx
                .app
                .models_mut()
                .insert(CalendarMonth::from_date(input_seed));
            let input_selected = cx.app.models_mut().insert(Some(input_seed));
            let input_value = cx.app.models_mut().insert(String::from("June 01, 2025"));

            let input_last_selected = Some(input_seed);

            let natural_open = cx.app.models_mut().insert(false);
            let natural_seed_value = String::from("In 2 days");
            let natural_selected = cx.app.models_mut().insert(None::<Date>);
            let natural_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            let natural_value = cx.app.models_mut().insert(natural_seed_value);
            let natural_last_selected = None;

            let time_open = cx.app.models_mut().insert(false);
            let time_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            let time_selected = cx.app.models_mut().insert(None::<Date>);
            let time_value = cx.app.models_mut().insert(String::from("10:30:00"));

            cx.with_state(DatePickerModels::default, |st| {
                st.basic_open = Some(basic_open.clone());
                st.basic_month = Some(basic_month.clone());
                st.basic_selected = Some(basic_selected.clone());
                st.range_open = Some(range_open.clone());
                st.range_month = Some(range_month.clone());
                st.range_selected = Some(range_selected.clone());
                st.dob_open = Some(dob_open.clone());
                st.dob_month = Some(dob_month.clone());
                st.dob_selected = Some(dob_selected.clone());
                st.rtl_open = Some(rtl_open.clone());
                st.rtl_month = Some(rtl_month.clone());
                st.rtl_selected = Some(rtl_selected.clone());
                st.presets_open = Some(presets_open.clone());
                st.presets_month = Some(presets_month.clone());
                st.presets_selected = Some(presets_selected.clone());
                st.input_open = Some(input_open.clone());
                st.input_month = Some(input_month.clone());
                st.input_selected = Some(input_selected.clone());
                st.input_value = Some(input_value.clone());
                st.input_last_selected = input_last_selected;
                st.natural_open = Some(natural_open.clone());
                st.natural_month = Some(natural_month.clone());
                st.natural_selected = Some(natural_selected.clone());
                st.natural_value = Some(natural_value.clone());
                st.natural_last_selected = natural_last_selected;
                st.time_open = Some(time_open.clone());
                st.time_month = Some(time_month.clone());
                st.time_selected = Some(time_selected.clone());
                st.time_value = Some(time_value.clone());
            });

            (
                basic_open,
                basic_month,
                basic_selected,
                range_open,
                range_month,
                range_selected,
                dob_open,
                dob_month,
                dob_selected,
                rtl_open,
                rtl_month,
                rtl_selected,
                presets_open,
                presets_month,
                presets_selected,
                input_open,
                input_month,
                input_selected,
                input_value,
                natural_open,
                natural_month,
                natural_selected,
                natural_value,
                time_open,
                time_month,
                time_selected,
                time_value,
            )
        }
    };

    let demo = snippets::demo::render(cx, open.clone(), month.clone(), selected.clone());
    let basic = snippets::basic::render(
        cx,
        basic_open.clone(),
        basic_month.clone(),
        basic_selected.clone(),
    );
    let usage = snippets::usage::render(cx);

    let label = snippets::label::render(cx);
    let range = snippets::range::render(
        cx,
        range_open.clone(),
        range_month.clone(),
        range_selected.clone(),
    );
    let dob = snippets::dob::render(
        cx,
        dob_open.clone(),
        dob_month.clone(),
        dob_selected.clone(),
    );
    let dropdowns = snippets::dropdowns::render(cx);
    let presets = snippets::presets::render(
        cx,
        presets_open.clone(),
        presets_month.clone(),
        presets_selected.clone(),
        today,
    );
    let natural_language = snippets::natural_language::render(
        cx,
        natural_open.clone(),
        natural_month.clone(),
        natural_selected.clone(),
        natural_value.clone(),
        today,
    );
    let input = snippets::input::render(
        cx,
        input_open.clone(),
        input_month.clone(),
        input_selected.clone(),
        input_value.clone(),
    );
    let time_picker = snippets::time_picker::render(
        cx,
        time_open.clone(),
        time_month.clone(),
        time_selected.clone(),
        time_value.clone(),
    );
    let rtl = snippets::rtl::render(
        cx,
        rtl_open.clone(),
        rtl_month.clone(),
        rtl_selected.clone(),
    );
    let notes_stack = snippets::notes::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Date Picker docs flow: Demo -> Usage -> Basic -> Range -> Date of birth -> Input -> Time -> Natural language -> RTL. Presets and dropdowns remain gallery extensions.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A compact date picker trigger (docs: Date Picker demo).")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for the compact `DatePicker` builder surface.")
                .test_id_prefix("ui-gallery-date-picker-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Basic", basic)
                .description("A basic date picker component (docs: Date Picker Basic).")
                .code_rust_from_file_region(
                    snippets::basic::SOURCE,
                    "example",
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Range Picker", range)
                .description("A date picker component for selecting a range of dates.")
                .code_rust_from_file_region(
                    snippets::range::SOURCE,
                    "example",
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Date of Birth", dob)
                .description(
                    "A date picker component with a dropdown caption layout for month/year selection.",
                )
                .code_rust_from_file_region(snippets::dob::SOURCE, "example")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Input", input)
                .description(
                    "InputGroup + calendar button + popover calendar (docs: Date Picker Input).",
                )
                .code_rust_from_file_region(
                    snippets::input::SOURCE,
                    "example",
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Time Picker", time_picker)
                .description("Date + time fields side-by-side (docs: Date Picker Time).")
                .code_rust_from_file_region(
                    snippets::time_picker::SOURCE,
                    "example",
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Natural Language Picker", natural_language)
                .description("This example parses natural language into a date (subset).")
                .code_rust_from_file_region(
                    snippets::natural_language::SOURCE,
                    "example",
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Label Association", label)
                .description("Use `FieldLabel::for_control`, `DatePicker::control_id`, and `DatePicker::test_id_prefix` to focus the trigger and keep derived automation anchors stable.")
                .test_id_prefix("ui-gallery-date-picker-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Extras: With Presets", presets)
                .description("shadcn `date-picker-with-presets` (Select + Calendar in a popover).")
                .code_rust_from_file_region(
                    snippets::presets::SOURCE,
                    "example",
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Extras: With Dropdowns", dropdowns)
                .description(
                    "Gallery-only: desktop uses a Popover; mobile uses a Drawer. Calendar caption uses dropdown month/year selection.",
                )
                .code_rust_from_file_region(
                    snippets::dropdowns::SOURCE,
                    "example",
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Notes", notes_stack)
                .description("Guidelines and parity notes for date picker recipes.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::notes::SOURCE, "example"),
        ],
    );

    vec![body]
}
