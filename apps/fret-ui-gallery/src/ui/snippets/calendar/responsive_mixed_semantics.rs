pub const SOURCE: &str = include_str!("responsive_mixed_semantics.rs");

// region: example
use fret_core::Px;
use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use time::Date;

#[derive(Default)]
struct Models {
    popover_open: Option<Model<bool>>,
    range_month: Option<Model<CalendarMonth>>,
    range_selected: Option<Model<DateRangeSelection>>,
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

fn today_from_env_or_now() -> Date {
    std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date())
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (popover_open, range_month, range_selected) = cx.with_state(Models::default, |st| {
        (
            st.popover_open.clone(),
            st.range_month.clone(),
            st.range_selected.clone(),
        )
    });

    let today = today_from_env_or_now();
    let range_from = time::Date::from_calendar_date(today.year(), time::Month::January, 12)
        .expect("valid range start date");
    let range_to = range_from + time::Duration::days(30);

    let popover_open = match popover_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.popover_open = Some(model.clone()));
            model
        }
    };
    let range_month = match range_month {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(CalendarMonth::from_date(range_from));
            cx.with_state(Models::default, |st| st.range_month = Some(model.clone()));
            model
        }
    };
    let range_selected = match range_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(DateRangeSelection {
                from: Some(range_from),
                to: Some(range_to),
            });
            cx.with_state(Models::default, |st| {
                st.range_selected = Some(model.clone())
            });
            model
        }
    };

    let panel_calendar = shadcn::CalendarRange::new(range_month.clone(), range_selected.clone())
        .number_of_months(2)
        .test_id_prefix("ui-gallery.calendar.responsive.panel")
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);

    let panel = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_px(Px(420.0)).min_w_0()),
        move |cx| {
            vec![
                shadcn::Badge::new("Panel: container queries").into_element(cx),
                panel_calendar,
            ]
        },
    )
    .test_id("ui-gallery-calendar-responsive-panel");

    let popover = shadcn::Popover::new(popover_open.clone())
        .side(shadcn::PopoverSide::Bottom)
        .align(shadcn::PopoverAlign::Start)
        .into_element(
            cx,
            move |cx| {
                shadcn::Button::new("Open calendar popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(popover_open.clone())
                    .test_id("ui-gallery-calendar-responsive-popover-trigger")
                    .into_element(cx)
            },
            move |cx| {
                let calendar =
                    shadcn::CalendarRange::new(range_month.clone(), range_selected.clone())
                        .number_of_months(2)
                        .test_id_prefix("ui-gallery.calendar.responsive.popover")
                        .into_element(cx);

                shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(
                        LayoutRefinement::default()
                            .w(fret_ui_kit::LengthRefinement::Auto)
                            .min_w_0()
                            .overflow_hidden(),
                    )
                    .into_element(cx)
                    .test_id("ui-gallery-calendar-responsive-popover-content")
            },
        );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |_cx| vec![panel, popover],
    )
}
// endregion: example
