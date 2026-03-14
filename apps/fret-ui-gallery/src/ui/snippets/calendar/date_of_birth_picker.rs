pub const SOURCE: &str = include_str!("date_of_birth_picker.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let today = today_from_env_or_now();
    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(today));
    let selected = cx.local_model_keyed("selected", || None::<Date>);

    let selected_now = cx.app.models().read(&selected, |v| *v).ok().flatten();
    if let Some(selected_now) = selected_now {
        let _ = cx
            .app
            .models_mut()
            .update(&month, |m| *m = CalendarMonth::from_date(selected_now));
    }

    let button_text = selected_now
        .map(format_date_locale_short_en_us)
        .unwrap_or_else(|| String::from("Select date"));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Date of birth").into_element(cx),
        shadcn::Popover::from_open(open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::Start)
            .into_element_with(
                cx,
                |cx| {
                    shadcn::Button::new(button_text)
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open.clone())
                        .content_justify(fret_ui_kit::Justify::Between)
                        .text_weight(fret_core::FontWeight::NORMAL)
                        .trailing_icon(fret_icons::IconId::new_static("lucide.chevron-down"))
                        .refine_layout(LayoutRefinement::default().w_px(Px(192.0)))
                        .into_element(cx)
                        .test_id("ui-gallery.calendar.dob.trigger")
                },
                |cx| {
                    let calendar = shadcn::Calendar::new(month.clone(), selected.clone())
                        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                        .test_id_prefix("ui-gallery.calendar.dob.calendar")
                        .close_on_select(open.clone())
                        .into_element(cx)
                        .test_id("ui-gallery.calendar.dob.calendar");

                    shadcn::PopoverContent::new([calendar])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            LayoutRefinement::default()
                                .w(fret_ui_kit::LengthRefinement::Auto)
                                .overflow_hidden(),
                        )
                        .into_element(cx)
                        .test_id("ui-gallery.calendar.dob.content")
                },
            ),
    ])
    .into_element(cx)
    .test_id("ui-gallery.calendar.dob")
}
// endregion: example
