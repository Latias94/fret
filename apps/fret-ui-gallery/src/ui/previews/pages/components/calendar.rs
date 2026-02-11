use super::super::super::super::*;

mod models;
mod sections;

pub(in crate::ui) fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();
    let today = time::OffsetDateTime::now_utc().date();

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
