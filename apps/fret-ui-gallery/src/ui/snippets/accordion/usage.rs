pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Accordion::single_uncontrolled(Some("item-1"))
        .collapsible(true)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(640.0)),
        )
        .items([shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![cx.text("Is it accessible?")]),
            shadcn::AccordionContent::new(vec![shadcn::typography::p(
                cx,
                "Yes. It adheres to the WAI-ARIA design pattern.",
            )]),
        )])
        .into_element(cx)
        .test_id("ui-gallery-accordion-usage")
}
// endregion: example
