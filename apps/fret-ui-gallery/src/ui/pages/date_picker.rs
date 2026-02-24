use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

use std::sync::Arc;

use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use fret_ui_kit::declarative::style as decl_style;

pub(super) fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct DatePickerModels {
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

    fn format_date_month_dd_yyyy_en(date: Date) -> String {
        use time::Month;

        let month = match date.month() {
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
        };

        format!("{month} {:02}, {}", date.day(), date.year())
    }

    fn parse_date_month_dd_yyyy_en(raw: &str) -> Option<Date> {
        use time::Month;

        let raw = raw.trim();
        let (month, rest) = raw.split_once(' ')?;
        let (day, year) = rest.split_once(", ")?;

        let month = match month {
            "January" => Month::January,
            "February" => Month::February,
            "March" => Month::March,
            "April" => Month::April,
            "May" => Month::May,
            "June" => Month::June,
            "July" => Month::July,
            "August" => Month::August,
            "September" => Month::September,
            "October" => Month::October,
            "November" => Month::November,
            "December" => Month::December,
            _ => return None,
        };

        let day: u8 = day.parse().ok()?;
        let year: i32 = year.parse().ok()?;
        Date::from_calendar_date(year, month, day).ok()
    }

    let diag_calendar_roving =
        std::env::var_os("FRET_UI_GALLERY_DIAG_CALENDAR_ROVING").is_some_and(|v| !v.is_empty());

    let theme = Theme::global(&*cx.app).snapshot();

    let today = std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());

    let (
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
        time_open,
        time_month,
        time_selected,
        time_value,
    ) = cx.with_state(DatePickerModels::default, |st| {
        (
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
            st.time_open.clone(),
            st.time_month.clone(),
            st.time_selected.clone(),
            st.time_value.clone(),
        )
    });

    let (
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
        time_open,
        time_month,
        time_selected,
        time_value,
    ) = match (
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
        time_open,
        time_month,
        time_selected,
        time_value,
    ) {
        (
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
            Some(time_open),
            Some(time_month),
            Some(time_selected),
            Some(time_value),
        ) => (
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
            time_open,
            time_month,
            time_selected,
            time_value,
        ),
        _ => {
            let range_open = cx.app.models_mut().insert(false);
            let diag_month = CalendarMonth::from_date(
                Date::from_calendar_date(2024, time::Month::February, 1).expect("valid date"),
            );
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
            let input_value = cx
                .app
                .models_mut()
                .insert(format_date_month_dd_yyyy_en(input_seed));

            let input_last_selected = Some(input_seed);

            let time_open = cx.app.models_mut().insert(false);
            let time_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            let time_selected = cx.app.models_mut().insert(None::<Date>);
            let time_value = cx.app.models_mut().insert(String::from("10:30:00"));

            cx.with_state(DatePickerModels::default, |st| {
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
                st.time_open = Some(time_open.clone());
                st.time_month = Some(time_month.clone());
                st.time_selected = Some(time_selected.clone());
                st.time_value = Some(time_value.clone());
            });

            (
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
                time_open,
                time_month,
                time_selected,
                time_value,
            )
        }
    };

    let simple = shadcn::DatePicker::new(open, month, selected.clone())
        .placeholder("Pick a date")
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-simple");

    let mut range_picker = shadcn::DateRangePicker::new(
        range_open.clone(),
        range_month.clone(),
        range_selected.clone(),
    )
    .placeholder("Pick a date");

    if diag_calendar_roving {
        let d14 = Date::from_calendar_date(2024, time::Month::February, 14).expect("valid date");
        let d15 = Date::from_calendar_date(2024, time::Month::February, 15).expect("valid date");
        range_picker = range_picker.disabled_by(move |d| d == d14 || d == d15);
    }

    let range_picker = range_picker
        .refine_layout(LayoutRefinement::default().w_px(Px(300.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-range");
    let range = range_picker;

    let dropdown_text = cx
        .app
        .models()
        .read(&dob_selected, |v| v.map(|d| d.to_string()))
        .ok()
        .flatten()
        .unwrap_or_else(|| "Pick a date".to_string());

    let dropdowns = {
        let theme = theme.clone();
        let open = dob_open.clone();

        shadcn::Popover::new(open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new(dropdown_text)
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open.clone())
                        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                        .into_element(cx)
                },
                |cx| {
                    let calendar = shadcn::Calendar::new(dob_month.clone(), dob_selected.clone())
                        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dropdowns-calendar");

                    let done = shadcn::Button::new("Done")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .toggle_model(open.clone())
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dropdowns-done");

                    let footer_props = decl_style::container_props(
                        &theme,
                        ChromeRefinement::default().p(Space::N2),
                        LayoutRefinement::default().w_full().min_w_0(),
                    );
                    let footer = cx.container(footer_props, move |_cx| vec![done]);

                    let separator = shadcn::Separator::new()
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dropdowns-separator");

                    let body = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N0)
                            .items_stretch()
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        move |_cx| vec![calendar, separator, footer],
                    );

                    shadcn::PopoverContent::new([body])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .into_element(cx)
                },
            )
            .test_id("ui-gallery-date-picker-dropdowns")
    };

    let presets = shadcn::DatePickerWithPresets::new(
        presets_open.clone(),
        presets_month.clone(),
        presets_selected.clone(),
    )
    .today(today)
    .placeholder("Pick a date")
    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
    .into_element(cx)
    .test_id("ui-gallery-date-picker-with-presets");

    let input = {
        let current_value = cx
            .app
            .models()
            .read(&input_value, |v| v.clone())
            .unwrap_or_default();

        if let Some(parsed) = parse_date_month_dd_yyyy_en(&current_value) {
            let selected_now = cx.app.models().read(&input_selected, |v| *v).ok().flatten();
            if selected_now != Some(parsed) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&input_selected, |v| *v = Some(parsed));
                let _ = cx
                    .app
                    .models_mut()
                    .update(&input_month, |v| *v = CalendarMonth::from_date(parsed));
            }
        }

        let selected_now = cx.app.models().read(&input_selected, |v| *v).ok().flatten();
        let next_value = cx.with_state(DatePickerModels::default, |st| {
            if selected_now != st.input_last_selected {
                st.input_last_selected = selected_now;
                Some(
                    selected_now
                        .map(format_date_month_dd_yyyy_en)
                        .unwrap_or_default(),
                )
            } else {
                None
            }
        });
        if let Some(next) = next_value {
            let _ = cx.app.models_mut().update(&input_value, |v| *v = next);
        }

        let open = input_open.clone();
        let open_for_calendar = open.clone();
        let open_for_key = open.clone();
        let month = input_month.clone();
        let selected = input_selected.clone();
        let calendar = move |cx: &mut ElementContext<'_, App>| {
            shadcn::Calendar::new(month.clone(), selected.clone())
                .close_on_select(open_for_calendar.clone())
                .into_element(cx)
                .test_id("ui-gallery-date-picker-input-calendar")
        };

        let popover = shadcn::Popover::new(open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::End)
            .align_offset(Px(-8.0))
            .side_offset(Px(10.0))
            .into_element(
                cx,
                move |cx| {
                    let trigger = shadcn::InputGroupButton::new("")
                        .a11y_label("Select date")
                        .size(shadcn::InputGroupButtonSize::IconSm)
                        .icon(fret_icons::IconId::new_static("lucide.calendar"))
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-input-trigger");

                    shadcn::PopoverTrigger::new(trigger).into_element(cx)
                },
                move |cx| {
                    shadcn::PopoverContent::new([calendar(cx)])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto),
                        )
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-input-content")
                },
            );

        shadcn::Field::new([
            shadcn::FieldLabel::new("Subscription Date").into_element(cx),
            shadcn::InputGroup::new(input_value.clone())
                .a11y_label("Subscription Date")
                .control_test_id("ui-gallery-date-picker-input-control")
                .control_on_key_down(Arc::new(move |host, _acx, kcx| {
                    if kcx.key == fret_core::KeyCode::ArrowDown {
                        let _ = host.models_mut().update(&open_for_key, |v| *v = true);
                        return true;
                    }
                    false
                }))
                .trailing([popover])
                .trailing_has_button(true)
                .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-date-picker-input")
    };

    let time_picker = {
        let date = shadcn::DatePicker::new(time_open.clone(), time_month.clone(), time_selected)
            .placeholder("Select date")
            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
            .into_element(cx)
            .test_id("ui-gallery-date-picker-time-date");

        let time = shadcn::InputGroup::new(time_value.clone())
            .a11y_label("Time")
            .control_test_id("ui-gallery-date-picker-time-control")
            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
            .into_element(cx)
            .test_id("ui-gallery-date-picker-time-input");

        shadcn::FieldGroup::new([
            shadcn::Field::new([shadcn::FieldLabel::new("Date").into_element(cx), date])
                .into_element(cx),
            shadcn::Field::new([shadcn::FieldLabel::new("Time").into_element(cx), time])
                .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-date-picker-time")
    };

    let demo = doc_layout::wrap_row_snapshot(
        cx,
        &theme,
        Space::N4,
        fret_ui::element::CrossAlign::Start,
        |_cx| vec![simple, dropdowns, range],
    )
    .test_id("ui-gallery-date-picker-demo");

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::DatePicker::new(rtl_open.clone(), rtl_month.clone(), rtl_selected.clone())
            .placeholder("Pick a date")
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
    })
    .test_id("ui-gallery-date-picker-rtl");

    let notes_stack = doc_layout::notes(
        cx,
        [
            "This page mirrors shadcn Date Picker docs (new-york-v4) and keeps the diag suite stable.",
            "The upstream dropdowns demo uses a Drawer on mobile; this gallery currently renders the Popover-only desktop recipe.",
            "Calendar dropdown caption improves large-jump navigation compared with arrow-only controls.",
            "For diag runs, some dates are intentionally disabled (via env flag) to validate skip behavior.",
            "Natural language picker (chrono-like parsing) is not implemented in this gallery yet.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn DatePickerDemo flow (Simple + With Dropdowns + With Range). Extras: RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Simple + With dropdown caption + With range (2 months).")
                .code(
                    "rust",
                    r#"// Simple
shadcn::DatePicker::new(open, month, selected)
    .placeholder("Pick a date")
    .into_element(cx);

// With dropdown caption + Done footer (Popover-only approximation)
shadcn::Popover::new(open).into_element(cx, |cx| trigger, |cx| {
    shadcn::PopoverContent::new([
        shadcn::Calendar::new(month, selected)
            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
            .into_element(cx),
    ])
    .into_element(cx)
});

// With range (2 months)
shadcn::DateRangePicker::new(open, month, range_selected).into_element(cx);"#,
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("With presets", presets)
                .description("shadcn `date-picker-with-presets` (Select + Calendar in a popover).")
                .code(
                    "rust",
                    r#"shadcn::DatePickerWithPresets::new(open, month, selected)
    .today(today) // optional: make presets deterministic
    .into_element(cx);"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Input", input)
                .description(
                    "InputGroup + calendar button + popover calendar (docs: Date Picker Input).",
                )
                .code(
                    "rust",
                    r#"let open = cx.app.models_mut().insert(false);
let date = cx.app.models_mut().insert(None::<Date>);
let month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
let value = cx.app.models_mut().insert(String::new());

let calendar = move |cx: &mut ElementContext<'_, App>| {
    shadcn::Calendar::new(month.clone(), date.clone())
        .close_on_select(open.clone())
        .into_element(cx)
};

let popover = shadcn::Popover::new(open.clone()).into_element(
    cx,
    move |cx| {
        let trigger = shadcn::InputGroupButton::new("")
            .a11y_label("Select date")
            .size(shadcn::InputGroupButtonSize::IconSm)
            .icon(fret_icons::IconId::new_static("lucide.calendar"))
            .into_element(cx);
        shadcn::PopoverTrigger::new(trigger).into_element(cx)
    },
    move |cx| {
        shadcn::PopoverContent::new([calendar(cx)])
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
            .into_element(cx)
    },
);

shadcn::InputGroup::new(value)
    .a11y_label("Subscription Date")
    .trailing([popover])
    .trailing_has_button(true)
    .into_element(cx);"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Time picker", time_picker)
                .description("Date + time fields side-by-side (docs: Date Picker Time).")
                .code(
                    "rust",
                    r#"let date_open = cx.app.models_mut().insert(false);
let date_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
let date = cx.app.models_mut().insert(None::<Date>);
let time = cx.app.models_mut().insert(String::from("10:30:00"));

let date_picker = shadcn::DatePicker::new(date_open, date_month, date)
    .placeholder("Select date")
    .into_element(cx);

let time_input = shadcn::InputGroup::new(time)
    .a11y_label("Time")
    .into_element(cx);

shadcn::FieldGroup::new([
    shadcn::Field::new([shadcn::FieldLabel::new("Date").into_element(cx), date_picker])
        .into_element(cx),
    shadcn::Field::new([shadcn::FieldLabel::new("Time").into_element(cx), time_input])
        .into_element(cx),
])
.into_element(cx);"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Extras: RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code(
                    "rust",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::DatePicker::new(open, month, selected).into_element(cx)
});"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Notes", notes_stack)
                .description("Guidelines and parity notes for date picker recipes.")
                .max_w(Px(780.0)),
        ],
    );

    vec![body]
}
