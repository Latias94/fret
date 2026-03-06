pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let trigger = shadcn::HoverCardTrigger::new(
        shadcn::Button::new("Hover")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
    )
    .into_element(cx);

    let content = shadcn::HoverCardContent::new([ui::text_block(
        "The React Framework - created and maintained by @vercel.",
    )
    .text_sm()
    .wrap(TextWrap::WordBreak)
    .into_element(cx)]);

    shadcn::HoverCard::new(trigger, content).into_element(cx)
}
// endregion: example
