use super::super::super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(in crate::ui) fn preview_accordion(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let _ = value;

    let max_w_xl = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(640.0)))
        .min_w_0();
    let max_w_lg = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(512.0)))
        .min_w_0();
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let demo = {
        let simple = shadcn::Accordion::single_uncontrolled(Some("item-1"))
            .collapsible(true)
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .items([
                shadcn::AccordionItem::new(
                    "item-1",
                    shadcn::AccordionTrigger::new(vec![cx.text("Is it accessible?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Yes. It adheres to the WAI-ARIA design pattern.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-2",
                    shadcn::AccordionTrigger::new(vec![cx.text("Is it styled?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Yes. It comes with default styles that matches the other components' aesthetic.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-3",
                    shadcn::AccordionTrigger::new(vec![cx.text("Is it animated?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Yes. It's animated by default, but you can disable it if you prefer.",
                    )]),
                ),
            ])
            .into_element(cx);

        let long = shadcn::Accordion::single_uncontrolled(Some("item-1"))
            .collapsible(true)
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .items([
                shadcn::AccordionItem::new(
                    "item-1",
                    shadcn::AccordionTrigger::new(vec![cx.text(
                        "What are the key considerations when implementing a comprehensive enterprise-level authentication system?",
                    )]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Implementing a robust enterprise authentication system requires careful consideration of multiple factors. This includes secure password hashing and storage, multi-factor authentication (MFA) implementation, session management, OAuth2 and SSO integration, regular security audits, rate limiting to prevent brute force attacks, and maintaining detailed audit logs. Additionally, you'll need to consider scalability, performance impact, and compliance with relevant data protection regulations such as GDPR or HIPAA.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-2",
                    shadcn::AccordionTrigger::new(vec![cx.text(
                        "How does modern distributed system architecture handle eventual consistency and data synchronization across multiple regions?",
                    )]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Modern distributed systems employ various strategies to maintain data consistency across regions. This often involves using techniques like CRDT (Conflict-Free Replicated Data Types), vector clocks, and gossip protocols. Systems might implement event sourcing patterns, utilize message queues for asynchronous updates, and employ sophisticated conflict resolution strategies. Popular solutions like Amazon's DynamoDB and Google's Spanner demonstrate different approaches to solving these challenges, balancing between consistency, availability, and partition tolerance as described in the CAP theorem.",
                    )]),
                ),
            ])
            .into_element(cx);

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(max_w_xl.clone()),
            move |_cx| vec![simple, long],
        )
        .test_id("ui-gallery-accordion-demo")
    };

    let extras = {
        let muted = shadcn::typography::muted(
            cx,
            "Extras are Fret-specific recipes and regression gates (not part of upstream shadcn AccordionDemo).",
        );

        let legacy_demo = shadcn::Accordion::single_uncontrolled(Some("shipping"))
            .collapsible(true)
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "shipping",
                    shadcn::AccordionTrigger::new(vec![cx.text("What are your shipping options?")])
                        .test_id("ui-gallery-accordion-demo-shipping-trigger"),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "We offer standard (5-7 days), express (2-3 days), and overnight shipping. Free shipping on international orders.",
                    )])
                    .test_id("ui-gallery-accordion-demo-shipping-content"),
                )
                .test_id("ui-gallery-accordion-demo-shipping-item"),
                shadcn::AccordionItem::new(
                    "returns",
                    shadcn::AccordionTrigger::new(vec![cx.text("What is your return policy?")])
                        .test_id("ui-gallery-accordion-demo-returns-trigger"),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Returns accepted within 30 days. Items must be unused and in original packaging. Refunds processed within 5-7 business days.",
                    )])
                    .test_id("ui-gallery-accordion-demo-returns-content"),
                )
                .test_id("ui-gallery-accordion-demo-returns-item"),
                shadcn::AccordionItem::new(
                    "support",
                    shadcn::AccordionTrigger::new(vec![cx.text(
                        "How can I contact customer support?",
                    )]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Reach us via email, live chat, or phone. We respond within 24 hours during business days.",
                    )]),
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-accordion-legacy-demo");

        let multiple = shadcn::Accordion::multiple_uncontrolled(["notifications"])
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "notifications",
                    shadcn::AccordionTrigger::new(vec![cx.text("Notification Settings")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Manage how you receive notifications. You can enable email alerts for updates or push notifications for mobile devices.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "privacy",
                    shadcn::AccordionTrigger::new(vec![cx.text("Privacy & Security")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Control your privacy settings and security preferences. Enable two-factor authentication, manage connected devices, review active sessions, and configure data sharing preferences. You can also download your data or delete your account.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "billing",
                    shadcn::AccordionTrigger::new(vec![cx.text("Billing & Subscription")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "View your current plan, payment history, and upcoming invoices. Update your payment method, change your subscription tier, or cancel your subscription.",
                    )]),
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-accordion-multiple");

        let disabled = shadcn::Accordion::single_uncontrolled(None::<Arc<str>>)
            .collapsible(true)
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "item-1",
                    shadcn::AccordionTrigger::new(vec![cx.text("Can I access my account history?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Yes, you can view your complete account history including all transactions, plan changes, and support tickets in the Account History section of your dashboard.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-2",
                    shadcn::AccordionTrigger::new(vec![cx.text("Premium feature information")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "This section contains information about premium features. Upgrade your plan to access this content.",
                    )]),
                )
                .disabled(true),
                shadcn::AccordionItem::new(
                    "item-3",
                    shadcn::AccordionTrigger::new(vec![cx.text("How do I update my email address?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "You can update your email address in your account settings. You'll receive a verification email at your new address to confirm the change.",
                    )]),
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-accordion-disabled");

        let borders = {
            let accordion = shadcn::Accordion::single_uncontrolled(Some("billing"))
                .collapsible(true)
                .refine_layout(LayoutRefinement::default().w_full())
                .items([
                    shadcn::AccordionItem::new(
                        "billing",
                        shadcn::AccordionTrigger::new(vec![cx.text("How does billing work?")]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "We offer monthly and annual subscription plans. Billing is charged at the beginning of each cycle, and you can cancel anytime. All plans include automatic backups, 24/7 support, and unlimited team members.",
                        )]),
                    )
                    .refine_style(ChromeRefinement::default().px(Space::N4)),
                    shadcn::AccordionItem::new(
                        "security",
                        shadcn::AccordionTrigger::new(vec![cx.text("Is my data secure?")]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "Yes. We use end-to-end encryption, SOC 2 Type II compliance, and regular third-party security audits. All data is encrypted at rest and in transit using industry-standard protocols.",
                        )]),
                    )
                    .refine_style(ChromeRefinement::default().px(Space::N4)),
                    shadcn::AccordionItem::new(
                        "integration",
                        shadcn::AccordionTrigger::new(vec![cx.text("What integrations do you support?")]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "We integrate with 500+ popular tools including Slack, Zapier, Salesforce, HubSpot, and more. You can also build custom integrations using our REST API and webhooks.",
                        )]),
                    )
                    .refine_style(ChromeRefinement::default().px(Space::N4)),
                ])
                .into_element(cx);

            let wrapper_props = cx.with_theme(|theme| {
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default().border_1().rounded(Radius::Lg),
                    max_w_lg.clone(),
                )
            });
            cx.container(wrapper_props, move |_cx| vec![accordion])
        }
        .test_id("ui-gallery-accordion-borders");

        let card = {
            let accordion = shadcn::Accordion::single_uncontrolled(Some("plans"))
                .collapsible(true)
                .refine_layout(LayoutRefinement::default().w_full())
                .items([
                    shadcn::AccordionItem::new(
                        "plans",
                        shadcn::AccordionTrigger::new(vec![cx.text("What subscription plans do you offer?")]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "We offer three subscription tiers: Starter ($9/month), Professional ($29/month), and Enterprise ($99/month). Each plan includes increasing storage limits, API access, priority support, and team collaboration features.",
                        )]),
                    ),
                    shadcn::AccordionItem::new(
                        "billing",
                        shadcn::AccordionTrigger::new(vec![cx.text("How does billing work?")]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "Billing occurs automatically at the start of each billing cycle. We accept all major credit cards, PayPal, and ACH transfers for enterprise customers. You'll receive an invoice via email after each payment.",
                        )]),
                    ),
                    shadcn::AccordionItem::new(
                        "cancel",
                        shadcn::AccordionTrigger::new(vec![cx.text("How do I cancel my subscription?")]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "You can cancel your subscription anytime from your account settings. There are no cancellation fees or penalties. Your access will continue until the end of your current billing period.",
                        )]),
                    ),
                ])
                .into_element(cx);

            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![
                    shadcn::CardTitle::new("Subscription & Billing").into_element(cx),
                    shadcn::CardDescription::new(
                        "Common questions about your account, plans, payments and cancellations.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardContent::new(vec![accordion]).into_element(cx),
            ])
            .refine_layout(max_w_sm.clone())
            .into_element(cx)
        }
        .test_id("ui-gallery-accordion-card");

        let rtl = doc_layout::rtl(cx, |cx| {
            shadcn::Accordion::single_uncontrolled(Some("item-1"))
                .collapsible(true)
                .dir(Some(fret_ui_kit::primitives::direction::LayoutDirection::Rtl))
                .refine_layout(max_w_lg.clone())
                .items([
                    shadcn::AccordionItem::new(
                        "item-1",
                        shadcn::AccordionTrigger::new(vec![cx.text(
                            "كيف يمكنني إعادة تعيين كلمة المرور؟",
                        )]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "انقر على 'نسيت كلمة المرور' في صفحة تسجيل الدخول، أدخل عنوان بريدك الإلكتروني، وسنرسل لك رابطًا لإعادة تعيين كلمة المرور. سينتهي صلاحية الرابط خلال 24 ساعة.",
                        )]),
                    ),
                    shadcn::AccordionItem::new(
                        "item-2",
                        shadcn::AccordionTrigger::new(vec![cx.text(
                            "هل يمكنني تغيير خطة الاشتراك الخاصة بي؟",
                        )]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "نعم، يمكنك ترقية أو تخفيض خطتك في أي وقت من إعدادات حسابك. ستظهر التغييرات في دورة الفوترة التالية.",
                        )]),
                    ),
                    shadcn::AccordionItem::new(
                        "item-3",
                        shadcn::AccordionTrigger::new(vec![cx.text(
                            "ما هي طرق الدفع التي تقبلونها؟",
                        )]),
                        shadcn::AccordionContent::new(vec![cx.text(
                            "نقبل جميع بطاقات الائتمان الرئيسية و PayPal والتحويلات المصرفية. تتم معالجة جميع المدفوعات بأمان من خلال شركاء الدفع لدينا.",
                        )]),
                    ),
                ])
                .into_element(cx)
        })
        .test_id("ui-gallery-accordion-rtl");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| {
                vec![
                    muted,
                    shadcn::typography::h4(cx, "Legacy Demo"),
                    legacy_demo,
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
        .test_id("ui-gallery-accordion-extras")
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows `accordion-demo.tsx` (new-york-v4): a simple accordion + a long-content accordion.",
            "Extras keep Fret-specific variants and test ids used by local regression harnesses.",
            "API reference: `ecosystem/fret-ui-shadcn/src/accordion.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "A vertically stacked set of interactive headings that each reveal a section of content.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Upstream shadcn AccordionDemo structure.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Accordion::single_uncontrolled(Some("item-1"))
    .collapsible(true)
    .items([
        shadcn::AccordionItem::new("item-1", trigger, content),
        shadcn::AccordionItem::new("item-2", trigger, content),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Extras", extras)
                .description("Fret-specific variants + RTL coverage.")
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"shadcn::Accordion::multiple_uncontrolled(["notifications"])
    .items([/* ... */])
    .into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).description("Parity notes and references."),
        ],
    );

    vec![body.test_id("ui-gallery-accordion")]
}
