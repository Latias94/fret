pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::FontWeight;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

fn format_date_lll_dd_y_en(date: Date) -> String {
    use time::Month;

    let month = match date.month() {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    };

    let day = format!("{:02}", date.day());
    format!("{month} {day}, {}", date.year())
}

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
        .map(format_date_lll_dd_y_en)
        .unwrap_or_else(|| String::from("Pick a date"));

    shadcn::Popover::new(open.clone())
        .side(shadcn::PopoverSide::Bottom)
        .align(shadcn::PopoverAlign::Start)
        .into_element(
            cx,
            |cx| {
                let mut button = shadcn::Button::new(button_text)
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(open.clone())
                    .content_justify(fret_ui_kit::Justify::Between)
                    .text_weight(FontWeight::NORMAL)
                    .trailing_icon(fret_icons::IconId::new_static("lucide.chevron-down"))
                    .refine_layout(LayoutRefinement::default().w_px(Px(212.0)))
                    .test_id("ui-gallery-date-picker-demo-trigger");

                if selected_now.is_none() {
                    button =
                        button.style(fret_ui_shadcn::button::ButtonStyle::default().foreground(
                            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Token {
                                key: "muted-foreground",
                                fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                            })),
                        ));
                }

                button.into_element(cx)
            },
            |cx| {
                let calendar = shadcn::Calendar::new(month.clone(), selected.clone())
                    .into_element(cx)
                    .test_id("ui-gallery-date-picker-demo-calendar");

                shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(
                        LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto),
                    )
                    .into_element(cx)
                    .test_id("ui-gallery-date-picker-demo-content")
            },
        )
        .test_id("ui-gallery-date-picker-demo")
}
// endregion: example
