pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn centered<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>,
{
    ui::h_flex(move |cx| [body.into_element(cx)])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let content = shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
        shadcn::PopoverDescription::new("Set the dimensions for the layer.").into_element(cx),
    ])
    .into_element(cx)])
    .test_id("ui-gallery-popover-basic-panel");

    let popover = shadcn::Popover::new(
        cx,
        shadcn::PopoverTrigger::build(
            shadcn::Button::new("Open Popover")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-popover-basic-trigger"),
        ),
        content,
    )
    .align(shadcn::PopoverAlign::Start)
    .into_element(cx)
    .test_id("ui-gallery-popover-basic-popover");

    centered(popover)
        .into_element(cx)
        .test_id("ui-gallery-popover-basic")
}
// endregion: example
