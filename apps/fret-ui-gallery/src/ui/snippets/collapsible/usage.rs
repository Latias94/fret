pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::CollapsibleRoot::new().into_element(cx, |cx| {
        vec![
            shadcn::CollapsibleTriggerPart::new([decl_text::text_button_label(
                cx,
                "Can I use this in my project?",
            )])
            .into_element(cx),
            shadcn::CollapsibleContentPart::new([decl_text::text_paragraph_break_words(
                cx,
                "Yes. Free to use for personal and commercial projects. No attribution required.",
            )])
            .into_element(cx),
        ]
    })
}
// endregion: example
