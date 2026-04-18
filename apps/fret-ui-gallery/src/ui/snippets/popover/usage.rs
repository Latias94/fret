pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let content = shadcn::PopoverContent::build(cx, |cx| {
        [shadcn::PopoverHeader::new([
            shadcn::PopoverTitle::new("Title").into_element(cx),
            shadcn::PopoverDescription::new("Description text here.").into_element(cx),
        ])]
    });

    shadcn::Popover::new(
        cx,
        shadcn::PopoverTrigger::build(
            shadcn::Button::new("Open Popover").variant(shadcn::ButtonVariant::Outline),
        ),
        content,
    )
    .into_element(cx)
}
// endregion: example
