pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let content = shadcn::HoverCardContent::build(cx, |cx| {
        [
            ui::raw_text("The React Framework – created and maintained by @vercel.")
                .into_element(cx),
        ]
    });

    shadcn::HoverCard::new(
        cx,
        shadcn::HoverCardTrigger::build(ui::raw_text("Hover")),
        content,
    )
    .into_element(cx)
}
// endregion: example
