use super::super::super::super::*;

mod models;
mod sections;

pub(in crate::ui) fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    let theme = Theme::global(&*cx.app).snapshot();

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

    let today = std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());

    let models = models::get_or_init(cx, month, selected, today);

    let basic = sections::basic(cx, &theme, &models);
    let range = sections::range(cx, &theme, &models);
    let responsive_mixed_semantics = sections::responsive_mixed_semantics(cx, &models);
    let month_year_selector = sections::month_year_selector(cx, &models);
    let presets = sections::presets(cx, &models, today);
    let date_and_time_picker = sections::date_and_time_picker(cx, &theme, &models);
    let booked_dates = sections::booked_dates(cx, &theme, &models);
    let custom_cell_size = sections::custom_cell_size(cx, &models);
    let week_numbers = sections::week_numbers(cx, &models);
    let rtl = sections::rtl(cx, &models);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Calendar demo (new-york-v4).",
            "Not all upstream variants are implemented yet (multiple selection, locale demo, custom day renderers/modifiers).",
            "Set `FRET_UI_GALLERY_FIXED_TODAY=YYYY-MM-DD` to make presets deterministic in screenshots/tests.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Calendar demos for single selection, ranges, presets, and layout variants."),
        vec![
            DocSection::new("Single selection", basic)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-single")
                .code(
                    "rust",
                    r#"shadcn::Calendar::new(month, selected)
    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
    .into_element(cx);"#,
                ),
            DocSection::new("Range selection", range)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-range")
                .code(
                    "rust",
                    r#"shadcn::CalendarRange::new(month, selected)
    .number_of_months(2)
    .into_element(cx);"#,
                ),
            DocSection::new("Responsive semantics", responsive_mixed_semantics)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-responsive")
                .code(
                    "rust",
                    r#"// Calendar is intentionally mixed:
// - in panels: container queries (editor-grade)
// - in popovers: viewport breakpoints (avoid circular sizing)
"#,
                ),
            DocSection::new("Month and year selector", month_year_selector)
                .test_id_prefix("ui-gallery-calendar-caption")
                .code(
                    "rust",
                    r#"shadcn::Calendar::new(month, selected)
    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
    .into_element(cx);"#,
                ),
            DocSection::new("Presets", presets)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-presets")
                .code(
                    "rust",
                    r#"shadcn::Card::new(vec![
    shadcn::CardContent::new(vec![calendar]).into_element(cx),
    shadcn::CardFooter::new(buttons).into_element(cx),
]);"#,
                ),
            DocSection::new("With time input", date_and_time_picker)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-time")
                .code(
                    "rust",
                    r#"shadcn::InputGroup::new(time_model)
    .a11y_label("Start Time")
    .trailing([clock_icon])
    .into_element(cx);"#,
                ),
            DocSection::new("Disabled dates", booked_dates)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-booked")
                .code(
                    "rust",
                    r#"shadcn::Calendar::new(month, selected)
    .disabled_by(|d| matches!(d.weekday(), time::Weekday::Saturday | time::Weekday::Sunday))
    .into_element(cx);"#,
                ),
            DocSection::new("Custom cell size", custom_cell_size)
                .test_id_prefix("ui-gallery-calendar-custom-cell")
                .code(
                    "rust",
                    r#"shadcn::Calendar::new(month, selected)
    .cell_size(Px(44.0))
    .into_element(cx);"#,
                ),
            DocSection::new("Week numbers", week_numbers)
                .test_id_prefix("ui-gallery-calendar-week-numbers")
                .code(
                    "rust",
                    r#"shadcn::Calendar::new(month, selected)
    .show_week_number(true)
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-calendar-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Calendar::new(month, selected).into_element(cx)
});"#,
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-notes"),
        ],
    );

    vec![body]
}
