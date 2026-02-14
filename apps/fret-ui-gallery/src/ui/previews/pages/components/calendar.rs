use super::super::super::super::*;

mod models;
mod sections;

pub(in crate::ui) fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
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
    let month_year_selector = sections::month_year_selector(cx, &models);
    let presets = sections::presets(cx, &models, today);
    let date_and_time_picker = sections::date_and_time_picker(cx, &theme, &models);
    let booked_dates = sections::booked_dates(cx, &theme, &models);
    let custom_cell_size = sections::custom_cell_size(cx, &models);
    let week_numbers = sections::week_numbers(cx, &models);
    let rtl = sections::rtl(cx, &models);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                basic,
                range,
                month_year_selector,
                presets,
                date_and_time_picker,
                booked_dates,
                custom_cell_size,
                week_numbers,
                rtl,
            ]
        },
    )]
}
