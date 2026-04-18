pub const SOURCE: &str = include_str!("dropdowns.rs");

// region: example
use super::fixed_today;
use fret::adaptive::{DeviceShellSwitchPolicy, device_shell_switch};
use fret::children::UiElementSinkExt;
use fret::component::prelude::Model;
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui::element::AnyElement;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

fn render_calendar_dropdown_content(
    cx: &mut AppComponentCx<'_>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
) -> AnyElement {
    shadcn::Calendar::new(month, selected)
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .test_id_prefix("ui-gallery-date-picker-dropdowns-calendar")
        .into_element(cx)
}

fn render_dropdown_trigger(cx: &mut AppComponentCx<'_>, open: Model<bool>) -> AnyElement {
    shadcn::Button::new("Pick a date")
        .variant(shadcn::ButtonVariant::Outline)
        .toggle_model(open)
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-dropdowns-trigger")
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();

    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(today));
    let selected = cx.local_model_keyed("selected", || None::<Date>);
    let desktop_open = open.clone();
    let desktop_month = month.clone();
    let desktop_selected = selected.clone();
    let mobile_open = open.clone();
    let mobile_month = month.clone();
    let mobile_selected = selected.clone();

    let overlay = device_shell_switch(
        cx,
        Invalidation::Layout,
        DeviceShellSwitchPolicy::default(),
        |cx| {
            shadcn::Popover::from_open(desktop_open.clone())
                .side(shadcn::PopoverSide::Bottom)
                .align(shadcn::PopoverAlign::Start)
                .into_element_with(
                    cx,
                    move |cx| render_dropdown_trigger(cx, desktop_open.clone()),
                    move |cx| {
                        shadcn::PopoverContent::build(cx, |cx| {
                            [render_calendar_dropdown_content(
                                cx,
                                desktop_month.clone(),
                                desktop_selected.clone(),
                            )]
                        })
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            LayoutRefinement::default()
                                .w(fret_ui_kit::LengthRefinement::Auto)
                                .min_w_0()
                                .min_h_0(),
                        )
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dropdowns-popover-content")
                    },
                )
        },
        |cx| {
            let done_open = mobile_open.clone();
            shadcn::Drawer::new(mobile_open.clone()).into_element(
                cx,
                move |cx| render_dropdown_trigger(cx, mobile_open.clone()),
                move |cx| {
                    let calendar = render_calendar_dropdown_content(
                        cx,
                        mobile_month.clone(),
                        mobile_selected.clone(),
                    );
                    shadcn::DrawerContent::build(|cx, out| {
                        out.push(calendar);
                        out.push_ui(
                            cx,
                            shadcn::DrawerFooter::build(|cx, out| {
                                out.push_ui(
                                    cx,
                                    shadcn::Button::new("Done")
                                        .variant(shadcn::ButtonVariant::Outline)
                                        .toggle_model(done_open.clone())
                                        .test_id("ui-gallery-date-picker-dropdowns-done"),
                                );
                            }),
                        );
                    })
                    .into_element(cx)
                    .test_id("ui-gallery-date-picker-dropdowns-drawer-content")
                },
            )
        },
    );

    shadcn::Field::new([
        shadcn::FieldLabel::new("With dropdowns").into_element(cx),
        overlay,
    ])
    .into_element(cx)
    .test_id("ui-gallery-date-picker-dropdowns")
}
// endregion: example
