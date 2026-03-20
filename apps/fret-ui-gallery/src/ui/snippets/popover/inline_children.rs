pub const SOURCE: &str = include_str!("inline_children.rs");

// region: example
use fret::{UiChild, UiCx};
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let content = shadcn::PopoverContent::build(cx, |cx| {
        [
            shadcn::PopoverHeader::new([
                shadcn::PopoverTitle::new("Inline children").into_element(cx),
                shadcn::PopoverDescription::new(
                    "Inline-sized children should keep their intrinsic width by default.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Button::new("Inline action")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-popover-inline-children-button")
                .into_element(cx),
        ]
    })
    .test_id("ui-gallery-popover-inline-children-panel");

    let popover = shadcn::Popover::new(
        cx,
        shadcn::PopoverTrigger::build(
            shadcn::Button::new("Open Popover")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-popover-inline-children-trigger"),
        ),
        content,
    )
    .align(shadcn::PopoverAlign::Start)
    .into_element(cx)
    .test_id("ui-gallery-popover-inline-children-popover");

    centered(popover)
        .into_element(cx)
        .test_id("ui-gallery-popover-inline-children-demo")
}
// endregion: example
