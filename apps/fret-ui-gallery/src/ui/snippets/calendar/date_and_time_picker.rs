pub const SOURCE: &str = include_str!("date_and_time_picker.rs");

// region: example
use fret_core::Px;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

const FIELD_W_PX: Px = Px(128.0);

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

fn today_from_env_or_now() -> Date {
    std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date())
}

fn format_date_locale_short_en_us(date: Date) -> String {
    let month = u8::from(date.month());
    format!("{month}/{}/{}", date.day(), date.year())
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let today = today_from_env_or_now();
    let time_date =
        time::Date::from_calendar_date(today.year(), today.month(), 12).expect("valid time date");
    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(time_date));
    let selected = cx.local_model_keyed("selected", || Some(time_date));
    let time_value = cx.local_model_keyed("time", || String::from("10:30:00"));

    let open_for_calendar = open.clone();
    let selected_for_calendar = selected.clone();

    let button_text = cx
        .app
        .models()
        .read(&selected, |v| *v)
        .ok()
        .flatten()
        .map(format_date_locale_short_en_us)
        .unwrap_or_else(|| String::from("Select date"));

    let calendar = shadcn::Calendar::new(month, selected_for_calendar)
        .test_id_prefix("ui-gallery.calendar.time")
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .close_on_select(open_for_calendar.clone())
        .into_element(cx)
        .test_id("ui-gallery.calendar.time.calendar");

    let popover = shadcn::Popover::new(open.clone())
        .align(shadcn::PopoverAlign::Start)
        .into_element(
            cx,
            move |cx| {
                let trigger = shadcn::Button::new(button_text.clone())
                    .variant(shadcn::ButtonVariant::Outline)
                    .content_justify(fret_ui_kit::Justify::Between)
                    .text_weight(fret_core::FontWeight::NORMAL)
                    .trailing_icon(fret_icons::IconId::new_static("lucide.chevron-down"))
                    .refine_layout(LayoutRefinement::default().w_px(FIELD_W_PX))
                    .into_element(cx)
                    .test_id("ui-gallery.calendar.time.date-trigger");

                shadcn::PopoverTrigger::new(trigger).into_element(cx)
            },
            move |cx| {
                shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(
                        LayoutRefinement::default()
                            .w(fret_ui_kit::LengthRefinement::Auto)
                            .overflow_hidden(),
                    )
                    .into_element(cx)
                    .test_id("ui-gallery.calendar.time.popover-content")
            },
        );

    let date_column =
        ui::v_stack(|cx| vec![shadcn::FieldLabel::new("Date").into_element(cx), popover])
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_px(FIELD_W_PX))
            .into_element(cx);

    let time_column = ui::v_stack(|cx| {
        vec![
            shadcn::FieldLabel::new("Time").into_element(cx),
            shadcn::InputGroup::new(time_value)
                .a11y_label("Time")
                .into_element(cx)
                .test_id("ui-gallery.calendar.time.time-input"),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_px(FIELD_W_PX))
    .into_element(cx);

    ui::h_flex(|_cx| vec![date_column, time_column])
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery.calendar.time.picker")
}
// endregion: example
