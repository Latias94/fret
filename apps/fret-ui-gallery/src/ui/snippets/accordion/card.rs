pub const SOURCE: &str = include_str!("card.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let accordion = shadcn::accordion_multiple_uncontrolled(cx, ["plans"], |cx| {
        [
            shadcn::AccordionItem::new(
                "plans",
                shadcn::AccordionTrigger::new(vec![decl_text::text_button_label(
                    cx,
                    "What subscription plans do you offer?",
                )]),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::typography::p(
                        "We offer multiple tiers with increasing storage limits, API access, and priority support.",
                    )
                ]),
            ),
            shadcn::AccordionItem::new(
                "billing",
                shadcn::AccordionTrigger::new(vec![decl_text::text_button_label(
                    cx,
                    "How does billing work?",
                )]),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::typography::p(
                        "Billing occurs automatically at the start of each billing cycle. You can update your payment method anytime.",
                    )
                ]),
            ),
        ]
    })
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Subscription & Billing"),
                    shadcn::card_description(
                        "Common questions about your account, plans, and payments.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; accordion]),
        ]
    })
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
