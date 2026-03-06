pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn centered<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement {
    ui::h_flex(move |_cx| [body])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let content = shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
        shadcn::PopoverDescription::new("Set the dimensions for the layer.").into_element(cx),
    ])
    .into_element(cx)])
    .test_id("ui-gallery-popover-basic-panel");

    let popover = shadcn::Popover::new_controllable(cx, None, false)
        .align(shadcn::PopoverAlign::Start)
        .build(
            cx,
            shadcn::PopoverTrigger::build(
                shadcn::Button::new("Open Popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-popover-basic-trigger"),
            ),
            content,
        )
        .test_id("ui-gallery-popover-basic-popover");

    centered(cx, popover).test_id("ui-gallery-popover-basic")
}
// endregion: example
