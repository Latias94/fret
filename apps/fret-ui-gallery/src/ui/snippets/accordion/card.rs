pub const SOURCE: &str = include_str!("card.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let accordion = shadcn::Accordion::multiple_uncontrolled(["plans"])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .items([
            shadcn::AccordionItem::new(
                "plans",
                shadcn::AccordionTrigger::new(vec![cx.text("What subscription plans do you offer?")]),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p(
                        "We offer multiple tiers with increasing storage limits, API access, and priority support.",
                    )
                ]),
            ),
            shadcn::AccordionItem::new(
                "billing",
                shadcn::AccordionTrigger::new(vec![cx.text("How does billing work?")]),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p(
                        "Billing occurs automatically at the start of each billing cycle. You can update your payment method anytime.",
                    )
                ]),
            ),
        ])
        .into_element(cx);

    let header = shadcn::CardHeader::new([
        shadcn::CardTitle::new("Subscription & Billing").into_element(cx),
        shadcn::CardDescription::new("Common questions about your account, plans, and payments.")
            .into_element(cx),
    ])
    .into_element(cx);

    let content = shadcn::CardContent::new([accordion]).into_element(cx);

    shadcn::Card::new([header, content])
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0(),
        )
        .into_element(cx)
        .test_id("ui-gallery-accordion-card")
}
// endregion: example
