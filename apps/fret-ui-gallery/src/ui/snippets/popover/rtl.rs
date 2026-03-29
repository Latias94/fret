pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let title = "الأبعاد";
        let description = "تعيين الأبعاد للطبقة.";
        let popover = |cx: &mut UiCx<'_>,
                       popover_test_id: &'static str,
                       trigger_test_id: &'static str,
                       label: &'static str,
                       side| {
            let content = shadcn::PopoverContent::build(cx, |cx| {
                [shadcn::PopoverHeader::new([
                    shadcn::PopoverTitle::new(title).into_element(cx),
                    shadcn::PopoverDescription::new(description).into_element(cx),
                ])]
            });

            shadcn::Popover::new(
                cx,
                shadcn::PopoverTrigger::build(
                    shadcn::Button::new(label)
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id(trigger_test_id),
                ),
                content,
            )
            .side(side)
            .into_element(cx)
            .test_id(popover_test_id)
        };

        ui::v_flex(move |cx| {
            let physical = ui::h_flex(move |cx| {
                [
                    (
                        "ui-gallery-popover-rtl-left-popover",
                        "ui-gallery-popover-rtl-left-trigger",
                        "يسار",
                        shadcn::PopoverSide::Left,
                    ),
                    (
                        "ui-gallery-popover-rtl-top-popover",
                        "ui-gallery-popover-rtl-top-trigger",
                        "أعلى",
                        shadcn::PopoverSide::Top,
                    ),
                    (
                        "ui-gallery-popover-rtl-bottom-popover",
                        "ui-gallery-popover-rtl-bottom-trigger",
                        "أسفل",
                        shadcn::PopoverSide::Bottom,
                    ),
                    (
                        "ui-gallery-popover-rtl-right-popover",
                        "ui-gallery-popover-rtl-right-trigger",
                        "يمين",
                        shadcn::PopoverSide::Right,
                    ),
                ]
                .into_iter()
                .map(|(popover_test_id, trigger_test_id, label, side)| {
                    cx.keyed(trigger_test_id, |cx| {
                        popover(cx, popover_test_id, trigger_test_id, label, side)
                    })
                })
                .collect::<Vec<_>>()
            })
            .gap(Space::N2)
            .wrap()
            .w_full()
            .items_center()
            .justify_center()
            .into_element(cx);

            let logical = ui::h_flex(move |cx| {
                [
                    (
                        "ui-gallery-popover-rtl-inline-start-popover",
                        "ui-gallery-popover-rtl-inline-start-trigger",
                        "بداية السطر",
                        shadcn::PopoverSide::InlineStart,
                    ),
                    (
                        "ui-gallery-popover-rtl-inline-end-popover",
                        "ui-gallery-popover-rtl-inline-end-trigger",
                        "نهاية السطر",
                        shadcn::PopoverSide::InlineEnd,
                    ),
                ]
                .into_iter()
                .map(|(popover_test_id, trigger_test_id, label, side)| {
                    cx.keyed(trigger_test_id, |cx| {
                        popover(cx, popover_test_id, trigger_test_id, label, side)
                    })
                })
                .collect::<Vec<_>>()
            })
            .gap(Space::N2)
            .wrap()
            .w_full()
            .items_center()
            .justify_center()
            .into_element(cx);

            vec![physical, logical]
        })
        .gap(Space::N4)
        .w_full()
        .into_element(cx)
    })
    .test_id("ui-gallery-popover-rtl")
}
// endregion: example
