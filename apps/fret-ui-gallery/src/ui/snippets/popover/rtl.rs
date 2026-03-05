pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let popover = |cx: &mut ElementContext<'_, H>, label: &'static str, side| {
            shadcn::Popover::new_controllable(cx, None, false)
                .side(side)
                .into_element(
                    cx,
                    |cx| {
                        let trigger = shadcn::Button::new(label)
                            .variant(shadcn::ButtonVariant::Outline)
                            .into_element(cx);
                        shadcn::PopoverTrigger::new(trigger).into_element(cx)
                    },
                    |cx| {
                        shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                            shadcn::PopoverTitle::new("الأبعاد").into_element(cx),
                            shadcn::PopoverDescription::new("تعيين الأبعاد للطبقة.")
                                .into_element(cx),
                        ])
                        .into_element(cx)])
                        .into_element(cx)
                    },
                )
        };

        fret_ui_kit::ui::h_flex(move |cx| {
            [
                ("left", "يسار", shadcn::PopoverSide::Left),
                ("top", "أعلى", shadcn::PopoverSide::Top),
                ("bottom", "أسفل", shadcn::PopoverSide::Bottom),
                ("right", "يمين", shadcn::PopoverSide::Right),
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
