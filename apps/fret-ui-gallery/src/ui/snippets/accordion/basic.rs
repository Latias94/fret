pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Accordion::single_uncontrolled(Some("item-1"))
        .collapsible(true)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0(),
        )
        .items([
            shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Is it accessible?")]),
                shadcn::AccordionContent::new(vec![shadcn::raw::typography::p(
                    "Yes. It adheres to the WAI-ARIA design pattern.",
                ).into_element(cx)]),
            ),
            shadcn::AccordionItem::new(
                "item-2",
                shadcn::AccordionTrigger::new(vec![cx.text("Is it styled?")]),
                shadcn::AccordionContent::new(vec![shadcn::raw::typography::p(
                    "Yes. It ships with shadcn-style trigger, chevron, spacing, and motion defaults.",
                ).into_element(cx)]),
            ),
        ])
        .into_element(cx)
        .test_id("ui-gallery-accordion-basic")
}
// endregion: example
