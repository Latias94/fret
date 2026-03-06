pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let trigger = shadcn::TooltipTrigger::new(
        shadcn::Button::new("Hover")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx),
    )
    .into_element(cx);

    let content = shadcn::TooltipContent::new([shadcn::TooltipContent::text(cx, "Add to library")])
        .into_element(cx);

    shadcn::Tooltip::new(trigger, content).into_element(cx)
}
// endregion: example
