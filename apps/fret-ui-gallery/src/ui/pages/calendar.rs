use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::calendar as snippets;

pub(super) fn preview_calendar(
    cx: &mut UiCx<'_>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    let _ = (month, selected);

    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let range = snippets::range::render(cx);
    let month_year_selector = snippets::month_year_selector::render(cx);
    let date_of_birth_picker = snippets::date_of_birth_picker::render(cx);
    let presets = snippets::presets::render(cx);
    let date_and_time_picker = snippets::date_and_time_picker::render(cx);
    let natural_language_picker = snippets::natural_language_picker::render(cx);
    let booked_dates = snippets::booked_dates::render(cx);
    let custom_cell_size = snippets::custom_cell_size::render(cx);
    let week_numbers = snippets::week_numbers::render(cx);
    let rtl = snippets::rtl::render(cx);
    let hijri = snippets::hijri::render(cx);
    let locale = snippets::locale::render(cx);
    let responsive_mixed_semantics = snippets::responsive_mixed_semantics::render(cx);

    let about = doc_layout::notes_block([
        "Fret's `Calendar` plays the same role that React DayPicker does upstream, but the month grid and selection logic live in `fret_ui_headless::calendar` instead of a DOM package.",
        "The shadcn recipe layer in `ecosystem/fret-ui-shadcn/src/calendar.rs` owns the visual parity surface: caption layout, navigation buttons, day-cell chrome, and slot-aware backgrounds.",
    ]);

    let date_picker = doc_layout::notes_block([
        "Use the dedicated `Date Picker` page when the calendar should be composed with a trigger, field, and popover; that composition remains a separate recipe rather than being folded into `Calendar` itself.",
        "This mirrors the upstream docs split between the base `Calendar` component and higher-level date-picker recipes.",
    ]);

    let selected_date_timezone = doc_layout::notes_block([
        "Upstream documents a `timeZone` prop because JS `Date` can shift the highlighted day across offsets when a calendar works with date-times.",
        "Fret `Calendar` selections are `time::Date`, so date-only selection does not need a calendar-level timezone prop to keep the chosen day stable.",
        "If an app needs time-of-day or instant semantics, keep timezone conversion at the surrounding form / date-picker boundary rather than inside the base calendar recipe.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/calendar.rs` (Calendar).",
        "Calendar exposes both `new(...)` for externally owned month state and `new_controllable(...)` for copyable docs/gallery-style authoring.",
        "For the closest source-aligned equivalent to upstream `selected/onSelect`, pass your selected model into `Calendar::new_controllable(cx, Some(selected), default_selected)` and let the calendar own only the visible month state.",
        "Gallery sections now mirror the upstream docs path first: Demo, Usage, About, Date Picker, Persian / Hijri / Jalali Calendar, Selected Date (With TimeZone), core examples, RTL. Fret-only extensions stay after that path.",
        "Default-style ownership follows shadcn upstream: recipe defaults own the inner calendar chrome (`bg-background`, padding, day-cell styling), while example-level `rounded-lg border`, `p-0`, and custom `--cell-size` tweaks stay caller-owned.",
        "Fret uses `time::Date` for selections, so timezone offset issues from JS `Date` do not apply.",
        "Set `FRET_UI_GALLERY_FIXED_TODAY=YYYY-MM-DD` to make presets deterministic in screenshots/tests.",
        "Diagnostics use inner `ui-gallery.calendar.*` test_id prefixes from snippets, while page sections keep `ui-gallery-calendar-*` doc IDs.",
        "Upstream uses a DayPicker-style `components.DayButton` escape hatch; in Fret the equivalent customization surface is `CalendarDayButton` plus refinements, so a generic children API is not currently warranted.",
    ]);

    let about = DocSection::build(cx, "About", about)
        .no_shell()
        .test_id_prefix("ui-gallery-calendar-about");
    let date_picker = DocSection::build(cx, "Date Picker", date_picker)
        .no_shell()
        .test_id_prefix("ui-gallery-calendar-date-picker");
    let selected_date_timezone =
        DocSection::build(cx, "Selected Date (With TimeZone)", selected_date_timezone)
            .no_shell()
            .test_id_prefix("ui-gallery-calendar-selected-date-timezone");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-calendar-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let hijri = DocSection::build(cx, "Persian / Hijri / Jalali Calendar", hijri)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-hijri")
        .code_rust_from_file_region(snippets::hijri::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let range = DocSection::build(cx, "Range Calendar", range)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-range")
        .code_rust_from_file_region(snippets::range::SOURCE, "example");
    let month_year_selector = DocSection::build(cx, "Month and Year Selector", month_year_selector)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-caption")
        .code_rust_from_file_region(snippets::month_year_selector::SOURCE, "example");
    let presets = DocSection::build(cx, "Presets", presets)
        .no_shell()
        .test_id_prefix("ui-gallery-calendar-presets")
        .code_rust_from_file_region(snippets::presets::SOURCE, "example");
    let date_and_time_picker = DocSection::build(cx, "Date and Time Picker", date_and_time_picker)
        .no_shell()
        .test_id_prefix("ui-gallery-calendar-time")
        .code_rust_from_file_region(snippets::date_and_time_picker::SOURCE, "example");
    let booked_dates = DocSection::build(cx, "Booked dates", booked_dates)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-booked")
        .code_rust_from_file_region(snippets::booked_dates::SOURCE, "example");
    let custom_cell_size = DocSection::build(cx, "Custom Cell Size", custom_cell_size)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-custom-cell")
        .code_rust_from_file_region(snippets::custom_cell_size::SOURCE, "example");
    let week_numbers = DocSection::build(cx, "Week Numbers", week_numbers)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-week-numbers")
        .code_rust_from_file_region(snippets::week_numbers::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let date_of_birth_picker = DocSection::build(cx, "Date of Birth Picker", date_of_birth_picker)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-dob")
        .code_rust_from_file_region(snippets::date_of_birth_picker::SOURCE, "example");
    let natural_language_picker =
        DocSection::build(cx, "Natural Language Picker", natural_language_picker)
            .no_shell()
            .test_id_prefix("ui-gallery-calendar-natural-language")
            .code_rust_from_file_region(snippets::natural_language_picker::SOURCE, "example");
    let locale = DocSection::build(cx, "Locale (WIP)", locale)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-calendar-locale")
        .code_rust_from_file_region(snippets::locale::SOURCE, "example");
    let responsive_mixed_semantics = DocSection::build(
        cx,
        "Responsive semantics (Fret)",
        responsive_mixed_semantics,
    )
    .max_w(Px(980.0))
    .test_id_prefix("ui-gallery-calendar-responsive")
    .code_rust_from_file_region(snippets::responsive_mixed_semantics::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn docs order first (`Demo`, `Usage`, `About`, `Date Picker`, `Persian / Hijri / Jalali Calendar`, `Selected Date (With TimeZone)`, examples, `RTL`), then appends Fret-only regression surfaces.",
        ),
        vec![
            demo,
            usage,
            about,
            date_picker,
            hijri,
            selected_date_timezone,
            basic,
            range,
            month_year_selector,
            presets,
            date_and_time_picker,
            booked_dates,
            custom_cell_size,
            week_numbers,
            rtl,
            date_of_birth_picker,
            natural_language_picker,
            locale,
            responsive_mixed_semantics,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-calendar").into_element(cx)]
}
