pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let max_w_xl = LayoutRefinement::default()
        .w_full()
        .max_w(Px(640.0))
        .min_w_0();
    let max_w_lg = LayoutRefinement::default()
        .w_full()
        .max_w(Px(512.0))
        .min_w_0();
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(Px(384.0))
        .min_w_0();

    let multiple = shadcn::Accordion::multiple_uncontrolled(["notifications"])
        .refine_layout(max_w_lg.clone())
        .items([
            shadcn::AccordionItem::new(
                "notifications",
                shadcn::AccordionTrigger::new(vec![cx.text("Notifications")]),
                shadcn::AccordionContent::new(vec![shadcn::typography::p(
                    cx,
                    "Configure email, push, and in-app notifications.",
                )]),
            ),
            shadcn::AccordionItem::new(
                "security",
                shadcn::AccordionTrigger::new(vec![cx.text("Security")]),
                shadcn::AccordionContent::new(vec![shadcn::typography::p(
                    cx,
                    "Manage passwords, 2FA, and active sessions.",
                )]),
            ),
        ])
        .into_element(cx);

    let disabled = shadcn::Accordion::single_uncontrolled(Some("item-1"))
        .collapsible(true)
        .refine_layout(max_w_sm.clone())
        .items([shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![cx.text("Disabled")]).disabled(true),
            shadcn::AccordionContent::new(vec![shadcn::typography::p(
                cx,
                "This item is disabled and should not be interactive.",
            )]),
        )])
        .into_element(cx);

    let borders = {
        let accordion = shadcn::Accordion::single_uncontrolled(Some("item-1"))
            .collapsible(true)
            .refine_layout(max_w_sm.clone())
            .items([shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Borders")]),
                shadcn::AccordionContent::new(vec![shadcn::typography::p(
                    cx,
                    "Use an outer chrome wrapper when you want a bordered surface.",
                )]),
            )])
            .into_element(cx);

        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Lg),
                LayoutRefinement::default().w_full().min_w_0(),
            )
        });

        cx.container(props, move |_cx| [accordion])
    };

    let card = {
        let accordion = shadcn::Accordion::single_uncontrolled(Some("item-1"))
            .collapsible(true)
            .refine_layout(max_w_sm.clone())
            .items([shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Card")]),
                shadcn::AccordionContent::new(vec![shadcn::typography::p(
                    cx,
                    "Card chrome is composed via `ChromeRefinement` on a wrapper container.",
                )]),
            )])
            .into_element(cx);

        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Lg)
                    .shadow_sm(),
                LayoutRefinement::default().w_full().min_w_0(),
            )
        });

        cx.container(props, move |_cx| [accordion])
    };

    let rtl = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::Accordion::single_uncontrolled(Some("item-1"))
                .collapsible(true)
                .refine_layout(max_w_xl.clone())
                .items([shadcn::AccordionItem::new(
                    "item-1",
                    shadcn::AccordionTrigger::new(vec![cx.text("RTL")]),
                    shadcn::AccordionContent::new(vec![shadcn::typography::p(
                        cx,
                        "Ensure icons and spacing mirror correctly under RTL.",
                    )]),
                )])
                .into_element(cx)
        },
    );

    let multiple = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3).items_start(),
        move |cx| {
            vec![
                shadcn::typography::h4(cx, "Multiple"),
                multiple,
                shadcn::typography::h4(cx, "Disabled"),
                disabled,
                shadcn::typography::h4(cx, "Borders"),
                borders,
                shadcn::typography::h4(cx, "Card"),
                card,
                shadcn::typography::h4(cx, "RTL"),
                rtl,
            ]
        },
    )
    .test_id("ui-gallery-accordion-extras");

    multiple
}
// endregion: example
