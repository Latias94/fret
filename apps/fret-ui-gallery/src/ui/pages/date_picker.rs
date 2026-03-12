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
    let diag_month = CalendarMonth::from_date(
        Date::from_calendar_date(2024, time::Month::February, 1).expect("valid date"),
    );

    let basic_open = cx.local_model_keyed("basic_open", || false);
    let basic_month = cx.local_model_keyed("basic_month", || {
        if diag_calendar_roving {
            diag_month
        } else {
            CalendarMonth::from_date(today)
        }
    });
    let basic_selected = cx.local_model_keyed("basic_selected", || None::<Date>);

    let range_open = cx.local_model_keyed("range_open", || false);
    let range_month = cx.local_model_keyed("range_month", || {
        if diag_calendar_roving {
            diag_month
        } else {
            CalendarMonth::from_date(today)
        }
    });
    let range_selected = cx.local_model_keyed("range_selected", DateRangeSelection::default);

    let dob_open = cx.local_model_keyed("dob_open", || false);
    let dob_month = cx.local_model_keyed("dob_month", || {
        if diag_calendar_roving {
            diag_month
        } else {
            CalendarMonth::from_date(today)
        }
    });
    let dob_selected = cx.local_model_keyed("dob_selected", || None::<Date>);

    let rtl_open = cx.local_model_keyed("rtl_open", || false);
    let rtl_month = cx.local_model_keyed("rtl_month", || {
        if diag_calendar_roving {
            diag_month
        } else {
            CalendarMonth::from_date(today)
        }
    });
    let rtl_selected = cx.local_model_keyed("rtl_selected", || Some(today));

    let presets_open = cx.local_model_keyed("presets_open", || false);
    let presets_month = cx.local_model_keyed("presets_month", || CalendarMonth::from_date(today));
    let presets_selected = cx.local_model_keyed("presets_selected", || None::<Date>);

    let input_seed = Date::from_calendar_date(2025, time::Month::June, 1).expect("date");
    let input_open = cx.local_model_keyed("input_open", || false);
    let input_month = cx.local_model_keyed("input_month", || CalendarMonth::from_date(input_seed));
    let input_selected = cx.local_model_keyed("input_selected", || Some(input_seed));
    let input_value = cx.local_model_keyed("input_value", || String::from("June 01, 2025"));

    let natural_open = cx.local_model_keyed("natural_open", || false);
    let natural_month = cx.local_model_keyed("natural_month", || CalendarMonth::from_date(today));
    let natural_selected = cx.local_model_keyed("natural_selected", || None::<Date>);
    let natural_value = cx.local_model_keyed("natural_value", || String::from("In 2 days"));

    let time_open = cx.local_model_keyed("time_open", || false);
    let time_month = cx.local_model_keyed("time_month", || CalendarMonth::from_date(today));
    let time_selected = cx.local_model_keyed("time_selected", || None::<Date>);
    let time_value = cx.local_model_keyed("time_value", || String::from("10:30:00"));

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
