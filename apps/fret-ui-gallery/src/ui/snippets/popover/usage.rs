pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Popover::new_controllable(cx, None, false).into_element(
        cx,
        |cx| {
            shadcn::PopoverTrigger::new(
                shadcn::Button::new("Open Popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            )
            .into_element(cx)
        },
        |cx| {
            shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                shadcn::PopoverTitle::new("Title").into_element(cx),
                shadcn::PopoverDescription::new("Description text here.").into_element(cx),
            ])
            .into_element(cx)])
            .into_element(cx)
        },
    )
}
// endregion: example
