use super::*;

#[path = "calendar/fixtures.rs"]
mod fixtures;

fn build_shadcn_calendar_22_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(192.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Date of birth").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected)
                    .week_start(time::Weekday::Sunday)
                    .disable_outside_days(false)
                    .into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_shadcn_calendar_23_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(224.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Select your stay").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected)
                    .week_start(time::Weekday::Sunday)
                    .disable_outside_days(false)
                    .into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}
