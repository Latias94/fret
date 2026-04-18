pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use super::{default_month, fixed_today, format_date_ppp_en};
use fret::{AppComponentCx, UiChild};
use fret_core::FontWeight;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();
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
        .map(format_date_ppp_en)
        .unwrap_or_else(|| String::from("Pick a date"));

    let mut trigger = shadcn::Button::new(button_text)
        .variant(shadcn::ButtonVariant::Outline)
        .content_justify(fret_ui_kit::Justify::Between)
        .text_weight(FontWeight::NORMAL)
        .trailing_icon(fret_icons::IconId::new_static("lucide.chevron-down"))
        .refine_layout(LayoutRefinement::default().w_px(Px(212.0)));

    if selected_now.is_none() {
        trigger = trigger.style(shadcn::raw::button::ButtonStyle::default().foreground(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Token {
                key: "muted-foreground",
                fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
            })),
        ));
    }

    let content = shadcn::PopoverContent::build(cx, |cx| {
        [shadcn::Calendar::new(month.clone(), selected.clone()).into_element(cx)]
    })
    .refine_style(ChromeRefinement::default().p(Space::N0))
    .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto));

    shadcn::Popover::new(cx, shadcn::PopoverTrigger::build(trigger), content)
        .side(shadcn::PopoverSide::Bottom)
        .align(shadcn::PopoverAlign::Start)
        .into_element(cx)
}
// endregion: example
