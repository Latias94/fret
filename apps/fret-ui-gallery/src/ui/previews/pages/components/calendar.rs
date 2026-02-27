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

    let demo = sections::demo(cx, &models);
    let basic = sections::basic(cx, &models);
    let locale = sections::locale(cx, &models);
    let range = sections::range(cx, &models);
    let responsive_mixed_semantics = sections::responsive_mixed_semantics(cx, &models);
    let month_year_selector = sections::month_year_selector(cx, &models);
    let presets = sections::presets(cx, &models, today);
    let date_and_time_picker = sections::date_and_time_picker(cx, &theme, &models);
    let booked_dates = sections::booked_dates(cx, &models, today);
    let custom_cell_size = sections::custom_cell_size(cx, &models);
    let week_numbers = sections::week_numbers(cx, &models);
    let rtl = sections::rtl(cx, &models);
    let hijri = sections::hijri(cx, &models);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn/ui v4 Calendar examples.",
            "Fret uses `time::Date` for selections, so timezone offset issues from JS `Date` do not apply.",
            "Not all upstream docs sections are shown yet (timezone/SSR guidance, custom day renderers/modifiers).",
            "Set `FRET_UI_GALLERY_FIXED_TODAY=YYYY-MM-DD` to make presets deterministic in screenshots/tests.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("A calendar component that allows users to select a date or a range of dates."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-demo")
                .code(
                    "rust",
                    r#"// Mirrors the shadcn/ui v4 `calendar-demo` preview.
// `month`: Model<CalendarMonth>, `selected`: Model<Option<Date>>
shadcn::Calendar::new(month, selected)
    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
    .refine_style(
        ChromeRefinement::default()
            .border_1()
            .rounded(Radius::Md)
            .shadow_sm(),
    )
    .into_element(cx);"#,
                ),
            DocSection::new("Basic", basic)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-single")
                .code(
                    "rust",
                    r#"use fret_ui_headless::calendar::CalendarMonth;
use time::OffsetDateTime;

let today = OffsetDateTime::now_utc().date();
let month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
let selected = cx.app.models_mut().insert(Some(today));

shadcn::Calendar::new(month, selected)
    .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
    .into_element(cx);"#,
                ),
            DocSection::new("Range Calendar", range)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-range")
                .code(
                    "rust",
                    r#"use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use time::OffsetDateTime;

let today = OffsetDateTime::now_utc().date();
let from = time::Date::from_calendar_date(today.year(), time::Month::January, 12).unwrap();
let to = from + time::Duration::days(30);

let month = cx.app.models_mut().insert(CalendarMonth::from_date(from));
let selected = cx.app.models_mut().insert(DateRangeSelection {
    from: Some(from),
    to: Some(to),
});

shadcn::CalendarRange::new(month, selected)
    .number_of_months(2)
    .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
    .into_element(cx);"#,
                ),
            DocSection::new("Month and Year Selector", month_year_selector)
                .test_id_prefix("ui-gallery-calendar-caption")
                .code(
                    "rust",
                    r#"// `month`: Model<CalendarMonth>, `selected`: Model<Option<Date>>
shadcn::Calendar::new(month, selected)
    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
    .into_element(cx);"#,
                ),
            DocSection::new("Presets", presets)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-presets")
                .code(
                    "rust",
                    r#"// Use a shared `Model<Option<Date>>` for selection + a separate `Model<CalendarMonth>`
// so preset buttons can "jump" the calendar viewport to the newly selected month.
let calendar = shadcn::Calendar::new(presets_month.clone(), presets_selected.clone())
    .fixed_weeks(true)
    .into_element(cx);

let buttons = vec![
    shadcn::Button::new("Today").variant(shadcn::ButtonVariant::Outline).into_element(cx),
    shadcn::Button::new("Tomorrow").variant(shadcn::ButtonVariant::Outline).into_element(cx),
];

shadcn::Card::new(vec![
    shadcn::CardContent::new(vec![calendar]).into_element(cx),
    shadcn::CardFooter::new(buttons).into_element(cx),
])"#,
                ),
            DocSection::new("Date and Time Picker", date_and_time_picker)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-time")
                .code(
                    "rust",
                    r#"// Compose `<Calendar />` + time inputs with `CardContent` and `CardFooter`.
shadcn::InputGroup::new(time_model)
    .a11y_label("Start Time")
    .trailing([clock_icon])
    .into_element(cx);"#,
                ),
            DocSection::new("Booked dates", booked_dates)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-booked")
                .code(
                    "rust",
                    r#"use fret_ui_headless::calendar::DayMatcher;
use std::sync::Arc;
use time::OffsetDateTime;

let today = OffsetDateTime::now_utc().date();
let year = today.year();
let start = time::Date::from_calendar_date(year, time::Month::January, 12).unwrap();
let booked = Arc::<[Date]>::from(
    (0..15)
        .map(|i| start + time::Duration::days(i))
        .collect::<Vec<_>>(),
);

shadcn::Calendar::new(month, selected)
    .disabled(DayMatcher::dates(booked))
    .into_element(cx);"#,
                ),
            DocSection::new("Custom Cell Size", custom_cell_size)
                .test_id_prefix("ui-gallery-calendar-custom-cell")
                .code(
                    "rust",
                    r#"// `month`: Model<CalendarMonth>, `selected`: Model<Option<Date>>
shadcn::Calendar::new(month, selected)
    .cell_size(Px(44.0))
    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
    .into_element(cx);"#,
                ),
            DocSection::new("Week Numbers", week_numbers)
                .test_id_prefix("ui-gallery-calendar-week-numbers")
                .code(
                    "rust",
                    r#"// `month`: Model<CalendarMonth>, `selected`: Model<Option<Date>>
shadcn::Calendar::new(month, selected)
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
            DocSection::new("Persian / Hijri / Jalali Calendar", hijri)
                .test_id_prefix("ui-gallery-calendar-hijri")
                .code(
                    "rust",
                    r#"use fret_ui_headless::calendar_solar_hijri::SolarHijriMonth;
use time::OffsetDateTime;

let today = OffsetDateTime::now_utc().date();
let month = cx.app.models_mut().insert(SolarHijriMonth::from_gregorian(today));
let selected = cx.app.models_mut().insert(Some(today));

shadcn::CalendarHijri::new(month, selected).into_element(cx);"#,
                ),
            DocSection::new("Locale (WIP)", locale)
                .test_id_prefix("ui-gallery-calendar-locale")
                .code(
                    "rust",
                    r#"// `month`: Model<CalendarMonth>, `selected`: Model<Option<Date>>
shadcn::Calendar::new(month, selected)
    .locale(shadcn::calendar::CalendarLocale::Es)
    .week_start(time::Weekday::Monday)
    .into_element(cx);"#,
                ),
            DocSection::new("Responsive semantics (Fret)", responsive_mixed_semantics)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-calendar-responsive")
                .code(
                    "rust",
                    r#"// Calendar is intentionally mixed:
// - in panels: container queries (editor-grade)
// - in popovers: viewport breakpoints (avoid circular sizing)
"#,
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-calendar-notes"),
        ],
    );

    vec![body]
}
