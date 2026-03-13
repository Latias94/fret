pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::accordion_single_uncontrolled(cx, Some("item-1"), |cx| {
        [
            shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Is it accessible?")]),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p("Yes. It adheres to the WAI-ARIA design pattern.")
                ]),
            ),
            shadcn::AccordionItem::new(
                "item-2",
                shadcn::AccordionTrigger::new(vec![cx.text("Is it styled?")]),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p(
                        "Yes. It ships with shadcn-style trigger, chevron, spacing, and motion defaults.",
                    )
                ]),
            ),
        ]
    })
        .collapsible(true)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0(),
        )
        .into_element(cx)
        .test_id("ui-gallery-accordion-basic")
}
// endregion: example
