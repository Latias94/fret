pub const SOURCE: &str = include_str!("disabled.rs");

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
        .items([shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![cx.text("Disabled")]),
            shadcn::AccordionContent::new(vec![shadcn::raw::typography::p(
                "This item is disabled and should not be interactive.",
            ).into_element(cx)]),
        )
        .disabled(true)])
        .into_element(cx)
        .test_id("ui-gallery-accordion-disabled")
}
// endregion: example
