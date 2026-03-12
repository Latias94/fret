pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Accordion::single_uncontrolled(Some("item-1"))
        .collapsible(true)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(640.0)),
        )
        .items([
            shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Product Information")]),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p(
                        "Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability.",
                    ),
                    shadcn::raw::typography::p(
                        "Key features include advanced processing capabilities, and an intuitive user interface designed for both beginners and experts.",
                    )
                ])
                .gap(Space::N4),
            ),
            shadcn::AccordionItem::new(
                "item-2",
                shadcn::AccordionTrigger::new(vec![cx.text("Shipping Details")])
                    .test_id("ui-gallery-accordion-demo-shipping-trigger"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p(
                        "We offer worldwide shipping through trusted courier partners. Standard delivery takes 3-5 business days, while express shipping ensures delivery within 1-2 business days.",
                    ),
                    shadcn::raw::typography::p(
                        "All orders are carefully packaged and fully insured. Track your shipment in real-time through our dedicated tracking portal.",
                    )
                ])
                .gap(Space::N4)
                .test_id("ui-gallery-accordion-demo-shipping-content"),
            )
            .test_id("ui-gallery-accordion-demo-shipping-item"),
            shadcn::AccordionItem::new(
                "item-3",
                shadcn::AccordionTrigger::new(vec![cx.text("Return Policy")])
                    .test_id("ui-gallery-accordion-demo-returns-trigger"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p(
                        "We stand behind our products with a comprehensive 30-day return policy. If you're not completely satisfied, simply return the item in its original condition.",
                    ),
                    shadcn::raw::typography::p(
                        "Our hassle-free return process includes free return shipping and full refunds processed within 48 hours of receiving the returned item.",
                    )
                ])
                .gap(Space::N4)
                .test_id("ui-gallery-accordion-demo-returns-content"),
            )
            .test_id("ui-gallery-accordion-demo-returns-item"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-accordion-demo")
}
// endregion: example
