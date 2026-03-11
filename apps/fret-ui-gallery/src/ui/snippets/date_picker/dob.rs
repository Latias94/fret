pub const SOURCE: &str = include_str!("dob.rs");

// region: example
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
) -> AnyElement {
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
        shadcn::Popover::new(open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
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

                    shadcn::PopoverContent::new([calendar])
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
