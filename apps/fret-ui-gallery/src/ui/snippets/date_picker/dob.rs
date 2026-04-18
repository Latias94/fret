pub const SOURCE: &str = include_str!("dob.rs");

// region: example
use super::{default_month, fixed_today};
use fret::{AppComponentCx, UiChild};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();
    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || default_month(today));
    let selected = cx.local_model_keyed("selected", || None::<Date>);

    let selected_now = cx.app.models().read(&selected, |v| *v).ok().flatten();
    if let Some(selected_now) = selected_now {
        let _ = cx
            .app
            .models_mut()
            .update(&month, |m| *m = CalendarMonth::from_date(selected_now));
    }

    let button_text = selected_now
        .map(|d| d.to_string())
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
                        .content_justify_start()
                        .text_weight(fret_core::FontWeight::NORMAL)
                        .refine_layout(LayoutRefinement::default().w_px(Px(176.0)))
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dob-trigger")
                },
                |cx| {
                    let calendar = shadcn::Calendar::new(month.clone(), selected.clone())
                        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                        .test_id_prefix("ui-gallery-date-picker-dob-calendar")
                        .close_on_select(open.clone())
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dob-calendar");

                    shadcn::PopoverContent::build(cx, |_cx| [calendar])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            LayoutRefinement::default()
                                .w(fret_ui_kit::LengthRefinement::Auto)
                                .overflow_hidden(),
                        )
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dob-content")
                },
            ),
    ])
    .into_element(cx)
    .test_id("ui-gallery-date-picker-dob")
}
// endregion: example
