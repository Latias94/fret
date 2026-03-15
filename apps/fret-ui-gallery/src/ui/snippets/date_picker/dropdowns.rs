pub const SOURCE: &str = include_str!("dropdowns.rs");

// region: example
use crate::ui::snippets::date_picker::fixed_today;
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();

    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(today));
    let selected = cx.local_model_keyed("selected", || None::<Date>);

    let is_desktop = fret_ui_kit::declarative::viewport_queries::viewport_width_at_least(
        cx,
        Invalidation::Layout,
        fret_ui_kit::declarative::viewport_queries::tailwind::MD,
        fret_ui_kit::declarative::viewport_queries::ViewportQueryHysteresis::default(),
    );

    let content_month = month.clone();
    let content_selected = selected.clone();
    let content = move |cx: &mut UiCx<'_>| {
        shadcn::Calendar::new(content_month.clone(), content_selected.clone())
            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
            .test_id_prefix("ui-gallery-date-picker-dropdowns-calendar")
            .into_element(cx)
    };

    let trigger_open = open.clone();
    let trigger = move |cx: &mut UiCx<'_>| {
        shadcn::Button::new("Pick a date")
            .variant(shadcn::ButtonVariant::Outline)
            .toggle_model(trigger_open.clone())
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
            .test_id("ui-gallery-date-picker-dropdowns-trigger")
    };

    let overlay = if is_desktop {
        shadcn::Popover::from_open(open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::Start)
            .into_element_with(
                cx,
                move |cx| trigger(cx),
                move |cx| {
                    shadcn::PopoverContent::new([content(cx)])
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
    } else {
        let done_open = open.clone();
        shadcn::Drawer::new(open.clone()).into_element(
            cx,
            move |cx| trigger(cx),
            move |cx| {
                shadcn::DrawerContent::new([
                    content(cx),
                    shadcn::DrawerFooter::new([shadcn::Button::new("Done")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(done_open.clone())
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dropdowns-done")])
                    .into_element(cx),
                ])
                .into_element(cx)
                .test_id("ui-gallery-date-picker-dropdowns-drawer-content")
            },
        )
    };

    shadcn::Field::new([
        shadcn::FieldLabel::new("With dropdowns").into_element(cx),
        overlay,
    ])
    .into_element(cx)
    .test_id("ui-gallery-date-picker-dropdowns")
}
// endregion: example
