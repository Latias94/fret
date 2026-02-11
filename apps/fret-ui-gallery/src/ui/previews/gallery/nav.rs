use super::super::super::*;

pub(in crate::ui) fn preview_tabs(
    cx: &mut ElementContext<'_, App>,
    _value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let primary = cx.with_theme(|theme| theme.color_required("primary"));
    let line_style = shadcn::tabs::TabsStyle::default()
        .trigger_background(fret_ui_kit::WidgetStateProperty::new(Some(
            ColorRef::Color(CoreColor::TRANSPARENT),
        )))
        .trigger_border_color(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Color(CoreColor::TRANSPARENT)))
                .when(
                    fret_ui_kit::WidgetStates::SELECTED,
                    Some(ColorRef::Color(primary)),
                ),
        );

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let card_panel = |cx: &mut ElementContext<'_, App>,
                      title: &'static str,
                      description: &'static str,
                      content: &'static str| {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new(title).into_element(cx),
                shadcn::CardDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![shadcn::typography::muted(cx, content)]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .into_element(cx)
    };

    let demo = {
        let tabs = shadcn::Tabs::uncontrolled(Some("overview"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "overview",
                    "Overview",
                    [card_panel(
                        cx,
                        "Overview",
                        "View your key metrics and recent project activity.",
                        "You have 12 active projects and 3 pending tasks.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "analytics",
                    "Analytics",
                    [card_panel(
                        cx,
                        "Analytics",
                        "Track performance and user engagement metrics.",
                        "Page views are up 25% compared to last month.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "reports",
                    "Reports",
                    [card_panel(
                        cx,
                        "Reports",
                        "Generate and download your detailed reports.",
                        "You have 5 reports ready and available to export.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "settings",
                    "Settings",
                    [card_panel(
                        cx,
                        "Settings",
                        "Manage your account preferences and options.",
                        "Configure notifications, security, and themes.",
                    )],
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-demo");

        let demo_shell = shell(cx, tabs);
        let body = centered(cx, demo_shell);
        section(cx, "Demo", body)
    };

    let line = {
        let tabs = shadcn::Tabs::uncontrolled(Some("overview"))
            .style(line_style.clone())
            .refine_style(ChromeRefinement::default().bg(ColorRef::Color(CoreColor::TRANSPARENT)))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new("overview", "Overview", Vec::<AnyElement>::new()),
                shadcn::TabsItem::new("analytics", "Analytics", Vec::<AnyElement>::new()),
                shadcn::TabsItem::new("reports", "Reports", Vec::<AnyElement>::new()),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-line");

        let group = stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |cx| {
            vec![
                tabs,
                shadcn::typography::muted(
                    cx,
                    "Line variant is approximated with trigger style overrides in current API.",
                ),
            ]
        });
        let body = centered(cx, group);
        section(cx, "Line", body)
    };

    let vertical = {
        let tabs = shadcn::Tabs::uncontrolled(Some("account"))
            .orientation(shadcn::tabs::TabsOrientation::Vertical)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
            .items([
                shadcn::TabsItem::new(
                    "account",
                    "Account",
                    [card_panel(
                        cx,
                        "Account",
                        "Update your account details and profile settings.",
                        "Display name and avatar were updated 2 days ago.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "password",
                    "Password",
                    [card_panel(
                        cx,
                        "Password",
                        "Change your password and keep your account secure.",
                        "Last password update was 28 days ago.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "notifications",
                    "Notifications",
                    [card_panel(
                        cx,
                        "Notifications",
                        "Choose how and when you receive updates.",
                        "Email alerts are enabled for build failures.",
                    )],
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-vertical");

        let vertical_shell = shell(cx, tabs);
        let body = centered(cx, vertical_shell);
        section(cx, "Vertical", body)
    };

    let disabled = {
        let tabs = shadcn::Tabs::uncontrolled(Some("home"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "home",
                    "Home",
                    [card_panel(
                        cx,
                        "Home",
                        "This panel remains interactive.",
                        "The disabled tab cannot be focused or activated.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "settings",
                    "Disabled",
                    [card_panel(
                        cx,
                        "Disabled",
                        "This panel should not become active.",
                        "",
                    )],
                )
                .disabled(true),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-disabled");

        let disabled_shell = shell(cx, tabs);
        let body = centered(cx, disabled_shell);
        section(cx, "Disabled", body)
    };

    let icons = {
        let preview_trigger = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.app-window")),
                    cx.text("Preview"),
                ]
            },
        );
        let code_trigger = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.code")),
                    cx.text("Code"),
                ]
            },
        );

        let tabs = shadcn::Tabs::uncontrolled(Some("preview"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "preview",
                    "Preview",
                    [card_panel(
                        cx,
                        "Preview",
                        "Visual output for the current component.",
                        "Switch between preview and code using icon tabs.",
                    )],
                )
                .trigger_child(preview_trigger),
                shadcn::TabsItem::new(
                    "code",
                    "Code",
                    [card_panel(
                        cx,
                        "Code",
                        "Implementation details and source view.",
                        "This panel can host syntax-highlighted snippets.",
                    )],
                )
                .trigger_child(code_trigger),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-icons");

        let icons_shell = shell(cx, tabs);
        let body = centered(cx, icons_shell);
        section(cx, "Icons", body)
    };

    let rtl = {
        let tabs = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Tabs::uncontrolled(Some("overview"))
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
                    .items([
                        shadcn::TabsItem::new(
                            "overview",
                            "Overview",
                            [card_panel(
                                cx,
                                "Overview",
                                "RTL layout should keep keyboard and focus behavior intact.",
                                "Direction-sensitive navigation is provided by direction context.",
                            )],
                        ),
                        shadcn::TabsItem::new(
                            "analytics",
                            "Analytics",
                            [card_panel(
                                cx,
                                "Analytics",
                                "Arrow-key movement follows RTL expectations.",
                                "Verify trigger order and selected styling in RTL mode.",
                            )],
                        ),
                        shadcn::TabsItem::new(
                            "reports",
                            "Reports",
                            [card_panel(
                                cx,
                                "Reports",
                                "Panel composition remains identical under RTL.",
                                "Only directional behavior should change.",
                            )],
                        ),
                    ])
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-tabs-rtl");

        let rtl_shell = shell(cx, tabs);
        let body = centered(cx, rtl_shell);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("A set of layered sections of content that are displayed one at a time."),
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![demo, line, vertical, disabled, icons, rtl],
        ),
    ]
}

pub(in crate::ui) fn preview_accordion(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let _ = value;

    let max_w_lg = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(512.0)))
        .min_w_0();
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    // Mirrors the top-level `accordion-demo` preview slot.
    let demo = {
        let accordion = shadcn::Accordion::single_uncontrolled(Some("shipping"))
            .collapsible(true)
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "shipping",
                    shadcn::AccordionTrigger::new(vec![cx.text("What are your shipping options?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "We offer standard (5-7 days), express (2-3 days), and overnight shipping. Free shipping on international orders.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "returns",
                    shadcn::AccordionTrigger::new(vec![cx.text("What is your return policy?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Returns accepted within 30 days. Items must be unused and in original packaging. Refunds processed within 5-7 business days.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "support",
                    shadcn::AccordionTrigger::new(vec![cx.text("How can I contact customer support?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Reach us via email, live chat, or phone. We respond within 24 hours during business days.",
                    )]),
                ),
            ])
            .into_element(cx);
        centered(cx, accordion)
    };

    let basic = {
        let accordion = shadcn::Accordion::single_uncontrolled(Some("item-1"))
            .collapsible(true)
            .refine_layout(max_w_lg.clone())
            .items([
                shadcn::AccordionItem::new(
                    "item-1",
                    shadcn::AccordionTrigger::new(vec![cx.text("How do I reset my password?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Click on 'Forgot Password' on the login page, enter your email address, and we'll send you a link to reset your password. The link will expire in 24 hours.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-2",
                    shadcn::AccordionTrigger::new(vec![cx.text("Can I change my subscription plan?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "Yes, you can upgrade or downgrade your plan at any time from your account settings. Changes will be reflected in your next billing cycle.",
                    )]),
                ),
                shadcn::AccordionItem::new(
                    "item-3",
                    shadcn::AccordionTrigger::new(vec![cx.text("What payment methods do you accept?")]),
                    shadcn::AccordionContent::new(vec![cx.text(
                        "We accept all major credit cards, PayPal, and bank transfers. All payments are processed securely through our payment partners.",
                    )]),
                ),
            ])
            .into_element(cx);
        let body = centered(cx, accordion);
        section(cx, "Basic", body)
    };

    let multiple = {
        let accordion = shadcn::Accordion::multiple_uncontrolled(["notifications"])
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
            .into_element(cx);
        let body = centered(cx, accordion);
        section(cx, "Multiple", body)
    };

    let disabled = {
        let accordion = shadcn::Accordion::single_uncontrolled(None::<Arc<str>>)
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
            .into_element(cx);
        let body = centered(cx, accordion);
        section(cx, "Disabled", body)
    };

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
        let wrapper = cx.container(wrapper_props, move |_cx| vec![accordion]);

        let body = centered(cx, wrapper);
        section(cx, "Borders", body)
    };

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

        let card = shadcn::Card::new(vec![
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
        .into_element(cx);

        let body = centered(cx, card);
        section(cx, "Card", body)
    };

    let rtl = {
        let accordion = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Accordion::single_uncontrolled(Some("item-1"))
                    .collapsible(true)
                    .dir(Some(fret_ui_kit::primitives::direction::LayoutDirection::Rtl))
                    .refine_layout(max_w_lg.clone())
                    .items([
                        shadcn::AccordionItem::new(
                            "item-1",
                            shadcn::AccordionTrigger::new(vec![cx.text("كيف يمكنني إعادة تعيين كلمة المرور؟")]),
                            shadcn::AccordionContent::new(vec![cx.text(
                                "انقر على 'نسيت كلمة المرور' في صفحة تسجيل الدخول، أدخل عنوان بريدك الإلكتروني، وسنرسل لك رابطًا لإعادة تعيين كلمة المرور. سينتهي صلاحية الرابط خلال 24 ساعة.",
                            )]),
                        ),
                        shadcn::AccordionItem::new(
                            "item-2",
                            shadcn::AccordionTrigger::new(vec![cx.text("هل يمكنني تغيير خطة الاشتراك الخاصة بي؟")]),
                            shadcn::AccordionContent::new(vec![cx.text(
                                "نعم، يمكنك ترقية أو تخفيض خطتك في أي وقت من إعدادات حسابك. ستظهر التغييرات في دورة الفوترة التالية.",
                            )]),
                        ),
                        shadcn::AccordionItem::new(
                            "item-3",
                            shadcn::AccordionTrigger::new(vec![cx.text("ما هي طرق الدفع التي تقبلونها؟")]),
                            shadcn::AccordionContent::new(vec![cx.text(
                                "نقبل جميع بطاقات الائتمان الرئيسية و PayPal والتحويلات المصرفية. تتم معالجة جميع المدفوعات بأمان من خلال شركاء الدفع لدينا.",
                            )]),
                        ),
                    ])
                    .into_element(cx)
            },
        );
        let body = centered(cx, accordion);
        section(cx, "RTL", body)
    };

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![basic, multiple, disabled, borders, card, rtl],
    );

    vec![demo, examples]
}
