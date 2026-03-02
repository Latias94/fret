use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::calendar as snippets;

pub(super) fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    let _ = (month, selected);

    let demo = snippets::demo::render(cx);
    let basic = snippets::basic::render(cx);
    let range = snippets::range::render(cx);
    let month_year_selector = snippets::month_year_selector::render(cx);
    let presets = snippets::presets::render(cx);
    let date_and_time_picker = snippets::date_and_time_picker::render(cx);
    let booked_dates = snippets::booked_dates::render(cx);
    let custom_cell_size = snippets::custom_cell_size::render(cx);
    let week_numbers = snippets::week_numbers::render(cx);
    let rtl = snippets::rtl::render(cx);
    let hijri = snippets::hijri::render(cx);
    let locale = snippets::locale::render(cx);
    let responsive_mixed_semantics = snippets::responsive_mixed_semantics::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn/ui v4 Calendar examples.",
            "Fret uses `time::Date` for selections, so timezone offset issues from JS `Date` do not apply.",
            "Set `FRET_UI_GALLERY_FIXED_TODAY=YYYY-MM-DD` to make presets deterministic in screenshots/tests.",
            "Diagnostics scripts depend on `ui-gallery.calendar.*` test_id prefixes and calendar section doc IDs.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("A calendar component that allows users to select a date or a range of dates."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-single")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Range Calendar", range)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-range")
                .code_rust_from_file_region(snippets::range::SOURCE, "example"),
            DocSection::new("Month and Year Selector", month_year_selector)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-caption")
                .code_rust_from_file_region(snippets::month_year_selector::SOURCE, "example"),
            DocSection::new("Presets", presets)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-presets")
                .code_rust_from_file_region(snippets::presets::SOURCE, "example"),
            DocSection::new("Date and Time Picker", date_and_time_picker)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-time")
                .code_rust_from_file_region(snippets::date_and_time_picker::SOURCE, "example"),
            DocSection::new("Booked dates", booked_dates)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-booked")
                .code_rust_from_file_region(snippets::booked_dates::SOURCE, "example"),
            DocSection::new("Custom Cell Size", custom_cell_size)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-custom-cell")
                .code_rust_from_file_region(snippets::custom_cell_size::SOURCE, "example"),
            DocSection::new("Week Numbers", week_numbers)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-week-numbers")
                .code_rust_from_file_region(snippets::week_numbers::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Persian / Hijri / Jalali Calendar", hijri)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-hijri")
                .code_rust_from_file_region(snippets::hijri::SOURCE, "example"),
            DocSection::new("Locale (WIP)", locale)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-locale")
                .code_rust_from_file_region(snippets::locale::SOURCE, "example"),
            DocSection::new("Responsive semantics (Fret)", responsive_mixed_semantics)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-responsive")
                .code_rust_from_file_region(
                    snippets::responsive_mixed_semantics::SOURCE,
                    "example",
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-page-calendar")]
}
