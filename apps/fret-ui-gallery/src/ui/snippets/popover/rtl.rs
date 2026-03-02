pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
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

            let physical = stack::hstack_build(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N2)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full())
                    .justify_center(),
                |cx, out| {
                    for (id, label, side) in [
                        ("left", "يسار", shadcn::PopoverSide::Left),
                        ("top", "أعلى", shadcn::PopoverSide::Top),
                        ("bottom", "أسفل", shadcn::PopoverSide::Bottom),
                        ("right", "يمين", shadcn::PopoverSide::Right),
                    ] {
                        out.push(cx.keyed(id, |cx| popover(cx, label, side)));
                    }
                },
            );

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N4)
                    .layout(LayoutRefinement::default().w_full()),
                move |_cx| [physical],
            )
        },
    )
    .test_id("ui-gallery-popover-rtl")
}
// endregion: example
