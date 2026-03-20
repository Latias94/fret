pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let popover = |cx: &mut UiCx<'_>, label: &'static str, side| {
            let content = shadcn::PopoverContent::build(cx, |cx| {
                [shadcn::PopoverHeader::new([
                    shadcn::PopoverTitle::new("???????").into_element(cx),
                    shadcn::PopoverDescription::new("????? ??????? ??????.").into_element(cx),
                ])]
            });

            shadcn::Popover::new(
                cx,
                shadcn::PopoverTrigger::build(
                    shadcn::Button::new(label).variant(shadcn::ButtonVariant::Outline),
                ),
                content,
            )
            .side(side)
            .into_element(cx)
        };

        fret_ui_kit::ui::h_flex(move |cx| {
            [
                ("left", "????", shadcn::PopoverSide::Left),
                ("top", "????", shadcn::PopoverSide::Top),
                ("bottom", "????", shadcn::PopoverSide::Bottom),
                ("right", "????", shadcn::PopoverSide::Right),
            ]
            .into_iter()
            .map(|(id, label, side)| cx.keyed(id, |cx| popover(cx, label, side)))
            .collect::<Vec<_>>()
        })
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .justify_center()
        .into_element(cx)
    })
    .test_id("ui-gallery-popover-rtl")
}
// endregion: example
