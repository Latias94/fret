pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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

    let multiple = shadcn::accordion_multiple_uncontrolled(cx, ["notifications"], |cx| {
        [
            shadcn::AccordionItem::new(
                "notifications",
                shadcn::AccordionTrigger::new(vec![cx.text("Notifications")])
                    .test_id("ui-gallery-accordion-extras-multiple-trigger-notifications"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p("Configure email, push, and in-app notifications.")
                ])
                .test_id("ui-gallery-accordion-extras-multiple-content-notifications"),
            ),
            shadcn::AccordionItem::new(
                "security",
                shadcn::AccordionTrigger::new(vec![cx.text("Security")])
                    .test_id("ui-gallery-accordion-extras-multiple-trigger-security"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p("Manage passwords, 2FA, and active sessions.")
                ]),
            ),
        ]
    })
    .refine_layout(max_w_lg.clone())
    .into_element(cx)
    .test_id("ui-gallery-accordion-extras-multiple");

    let disabled = shadcn::accordion_single_uncontrolled(cx, Some("item-1"), |cx| {
        [shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![cx.text("Disabled")]).disabled(true),
            shadcn::AccordionContent::new(ui::children![
                cx;
                shadcn::raw::typography::p("This item is disabled and should not be interactive.")
            ]),
        )]
    })
    .collapsible(true)
    .refine_layout(max_w_sm.clone())
    .into_element(cx);

    let borders = {
        let accordion = shadcn::accordion_single_uncontrolled(cx, Some("item-1"), |cx| {
            [shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Borders")]),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p(
                        "Use an outer chrome wrapper when you want a bordered surface.",
                    )
                ]),
            )]
        })
        .collapsible(true)
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                max_w_sm
                    .clone()
                    .merge(LayoutRefinement::default().overflow_visible()),
            )
        });

        cx.container(props, move |_cx| [accordion])
            .test_id("ui-gallery-accordion-extras-borders")
    };

    let card = {
        let accordion = shadcn::accordion_multiple_uncontrolled(cx, ["plans"], |cx| {
            [
                shadcn::AccordionItem::new(
                    "plans",
                    shadcn::AccordionTrigger::new(vec![cx.text(
                        "What subscription plans do you offer?",
                    )]),
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
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-accordion-extras-card")
    };

    let rtl = with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::accordion_single_uncontrolled(cx, Some("item-1"), |cx| {
            [shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("RTL")])
                    .test_id("ui-gallery-accordion-extras-rtl-trigger"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p(
                        "Ensure icons and spacing mirror correctly under RTL.",
                    )
                ]),
            )]
        })
        .collapsible(true)
        .refine_layout(max_w_xl.clone())
        .into_element(cx)
    });

    let multiple_section = ui::v_flex(move |cx| {
        vec![
            shadcn::raw::typography::h4("Multiple").into_element(cx),
            multiple,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let disabled_section = ui::v_flex(move |cx| {
        vec![
            shadcn::raw::typography::h4("Disabled").into_element(cx),
            disabled,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let borders_section = ui::v_flex(move |cx| {
        vec![
            shadcn::raw::typography::h4("Borders").into_element(cx),
            borders,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let card_section =
        ui::v_flex(move |cx| vec![shadcn::raw::typography::h4("Card").into_element(cx), card])
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

    let rtl_section =
        ui::v_flex(move |cx| vec![shadcn::raw::typography::h4("RTL").into_element(cx), rtl])
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
            .test_id("ui-gallery-accordion-extras-rtl-section");

    let extras = ui::v_flex(move |_cx| {
        vec![
            multiple_section,
            disabled_section,
            borders_section,
            card_section,
            rtl_section,
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-accordion-extras");

    extras
}
// endregion: example
