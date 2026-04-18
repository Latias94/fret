pub const SOURCE: &str = include_str!("presets.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
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

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = today_from_env_or_now();
    let preset_date = time::Date::from_calendar_date(today.year(), time::Month::February, 12)
        .expect("valid preset date");
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(preset_date));
    let selected = cx.local_model_keyed("selected", || Some(preset_date));
    let preset_button = |label: &'static str, test_id: &'static str, days: i64| {
        let month = month.clone();
        let selected = selected.clone();
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .refine_layout(LayoutRefinement::default().flex_1().min_w(Px(72.0)))
            .test_id(test_id)
            .on_activate(Arc::new(move |host, _acx, _reason| {
                let new_date = today + time::Duration::days(days);
                let _ = host.models_mut().update(&selected, |v| *v = Some(new_date));
                let _ = host.models_mut().update(&month, |m| {
                    *m = CalendarMonth::from_date(new_date);
                });
            }))
    };

    let calendar = shadcn::Calendar::new(month.clone(), selected.clone())
        .test_id_prefix("ui-gallery.calendar.presets")
        .fixed_weeks(true)
        .cell_size(Px(38.0))
        .refine_style(ChromeRefinement::default().p(Space::N0))
        .into_element(cx);

    let card = shadcn::card(move |cx| {
        ui::children![
            cx;
            shadcn::card_content(move |cx| ui::children![cx; calendar]).size(shadcn::CardSize::Sm),
            shadcn::card_footer(move |cx| {
                ui::children![
                    cx;
                    preset_button("Today", "ui-gallery-calendar-presets-button-today", 0),
                    preset_button(
                        "Tomorrow",
                        "ui-gallery-calendar-presets-button-tomorrow",
                        1,
                    ),
                    preset_button(
                        "In 3 days",
                        "ui-gallery-calendar-presets-button-in-3-days",
                        3,
                    ),
                    preset_button(
                        "In a week",
                        "ui-gallery-calendar-presets-button-in-a-week",
                        7,
                    ),
                    preset_button(
                        "In 2 weeks",
                        "ui-gallery-calendar-presets-button-in-2-weeks",
                        14,
                    ),
                ]
            })
                .size(shadcn::CardSize::Sm)
                .border_top(true)
                .wrap(true)
                .gap(Space::N2),
        ]
    })
    .size(shadcn::CardSize::Sm)
    .refine_layout(
        LayoutRefinement::default()
            .w(fret_ui_kit::LengthRefinement::Auto)
            .max_w(MetricRef::Px(Px(300.0)))
            .min_w_0(),
    )
    .into_element(cx);

    ui::h_flex(move |_cx| vec![card])
        .justify_center()
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-calendar-presets-card")
}
// endregion: example
