pub const SOURCE: &str = include_str!("responsive_mixed_semantics.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let range_from = time::Date::from_calendar_date(2022, time::Month::January, 20)
        .expect("valid range start date");
    let range_to = range_from + time::Duration::days(20);
    let popover_open = cx.local_model_keyed("popover_open", || false);
    let range_month = cx.local_model_keyed("range_month", || CalendarMonth::from_date(range_from));
    let range_selected = cx.local_model_keyed("range_selected", || DateRangeSelection {
        from: Some(range_from),
        to: Some(range_to),
    });

    let panel_calendar = shadcn::CalendarRange::new(range_month.clone(), range_selected.clone())
        .fixed_weeks(true)
        .number_of_months(2)
        .test_id_prefix("ui-gallery.calendar.responsive.panel")
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);

    let panel = ui::v_stack(move |cx| {
        vec![
            shadcn::Badge::new("Panel: container queries").into_element(cx),
            panel_calendar,
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_px(Px(420.0)).min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-calendar-responsive-panel");

    let popover = shadcn::Popover::from_open(popover_open.clone())
        .side(shadcn::PopoverSide::Bottom)
        .align(shadcn::PopoverAlign::Start)
        .into_element_with(
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
                        .fixed_weeks(true)
                        .number_of_months(2)
                        .test_id_prefix("ui-gallery.calendar.responsive.popover")
                        .into_element(cx);

                shadcn::PopoverContent::build(cx, |_cx| [calendar])
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

    ui::h_flex(move |_cx| vec![panel, popover])
        .gap(Space::N6)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
// endregion: example
