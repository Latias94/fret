// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn centered<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_center(),
        move |_cx| [body],
    )
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let popover = shadcn::Popover::new_controllable(cx, None, false)
        .align(shadcn::PopoverAlign::Start)
        .into_element(
            cx,
            |cx| {
                let trigger = shadcn::Button::new("Open Popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
                    .test_id("ui-gallery-popover-basic-trigger");
                shadcn::PopoverTrigger::new(trigger).into_element(cx)
            },
            |cx| {
                shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                    shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                    shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                        .into_element(cx),
                ])
                .into_element(cx)])
                .into_element(cx)
                .test_id("ui-gallery-popover-basic-panel")
            },
        )
        .test_id("ui-gallery-popover-basic-popover");

    centered(cx, popover).test_id("ui-gallery-popover-basic")
}
// endregion: example
