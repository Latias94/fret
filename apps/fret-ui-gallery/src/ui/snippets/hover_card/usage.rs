pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let content = shadcn::HoverCardContent::build(cx, |cx| {
        [decl_text::text_paragraph_break_words(
            cx,
            "The React Framework – created and maintained by @vercel.",
        )]
    });
    let trigger_label = decl_text::text_button_label(cx, "Hover");

    shadcn::HoverCard::new(cx, shadcn::HoverCardTrigger::build(trigger_label), content)
        .into_element(cx)
}
// endregion: example
