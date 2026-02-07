use super::super::*;

pub(super) fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();

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
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(720.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let build_alert = |cx: &mut ElementContext<'_, App>,
                       test_id: &'static str,
                       variant: shadcn::AlertVariant,
                       icon_name: &'static str,
                       title: &'static str,
                       description: &'static str| {
        shadcn::Alert::new([
            shadcn::icon::icon(cx, fret_icons::IconId::new_static(icon_name)),
            shadcn::AlertTitle::new(title).into_element(cx),
            shadcn::AlertDescription::new(description).into_element(cx),
        ])
        .variant(variant)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id(test_id))
    };

    let demo_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                build_alert(
                    cx,
                    "ui-gallery-alert-demo-success",
                    shadcn::AlertVariant::Default,
                    "lucide.circle-check",
                    "Payment successful",
                    "Your payment of $29.99 has been processed and a receipt has been emailed.",
                ),
                build_alert(
                    cx,
                    "ui-gallery-alert-demo-info",
                    shadcn::AlertVariant::Default,
                    "lucide.info",
                    "New feature available",
                    "Dark mode support is now available in account settings.",
                ),
            ]
        },
    )
    .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-alert-demo"));
    let demo = section_card(cx, "Demo", demo_content);

    let basic_content = build_alert(
        cx,
        "ui-gallery-alert-basic",
        shadcn::AlertVariant::Default,
        "lucide.circle-check",
        "Account updated successfully",
        "Your profile information has been saved and applied immediately.",
    );
    let basic = section_card(cx, "Basic", basic_content);

    let destructive_content = build_alert(
        cx,
        "ui-gallery-alert-destructive",
        shadcn::AlertVariant::Destructive,
        "lucide.triangle-alert",
        "Payment failed",
        "Please verify card details, billing address, and available funds.",
    );
    let destructive = section_card(cx, "Destructive", destructive_content);

    let action_content = {
        let action = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_end(),
            |cx| {
                vec![
                    shadcn::Button::new("Enable")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .into_element(cx),
                ]
            },
        );

        shadcn::Alert::new([
            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.moon")),
            shadcn::AlertTitle::new("Dark mode is now available").into_element(cx),
            shadcn::AlertDescription::new(
                "Enable it in profile settings to reduce eye strain during long sessions.",
            )
            .into_element(cx),
            action,
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-alert-action"))
    };
    let action = section_card(cx, "Action", action_content);

    let custom_colors_content = shadcn::Alert::new([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.triangle-alert")),
        shadcn::AlertTitle::new("Your subscription expires in 3 days").into_element(cx),
        shadcn::AlertDescription::new(
            "Renew now to avoid service interruption or upgrade to a paid plan.",
        )
        .into_element(cx),
    ])
    .refine_style(
        ChromeRefinement::default()
            .bg(ColorRef::Color(CoreColor {
                r: 1.0,
                g: 0.98,
                b: 0.92,
                a: 1.0,
            }))
            .border_color(ColorRef::Color(CoreColor {
                r: 0.98,
                g: 0.85,
                b: 0.45,
                a: 1.0,
            })),
    )
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-alert-colors"));
    let custom_colors = section_card(cx, "Custom Colors", custom_colors_content);

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3).items_start(),
                |cx| {
                    vec![build_alert(
                        cx,
                        "ui-gallery-alert-rtl",
                        shadcn::AlertVariant::Default,
                        "lucide.info",
                        "RTL alert sample",
                        "This alert validates right-to-left layout and text alignment.",
                    )]
                },
            )
        },
    );
    let rtl = section_card(cx, "RTL", rtl_content);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Alert docs order and groups each scenario for quick lookup.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                basic,
                destructive,
                action,
                custom_colors,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-alert-component"));

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Basic / Destructive",
                    "Alert::new([icon, AlertTitle::new(...), AlertDescription::new(...)])\n    .variant(AlertVariant::Default | Destructive)",
                ),
                code_block(
                    cx,
                    "Action",
                    "Alert::new([... , action_row])\n// current Fret API has no AlertAction slot yet, use inline row as workaround",
                ),
                code_block(
                    cx,
                    "Custom Colors / RTL",
                    "Alert::new([...]).refine_style(ChromeRefinement::default().bg(...).border_color(...))\nwith_direction_provider(LayoutDirection::Rtl, |cx| ...)",
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "Keep alert copy concise and action-oriented; reserve longer guidance for Dialog or Sheet.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use `Destructive` only for high-risk or blocking failures to preserve visual hierarchy.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Current gallery uses an inline action row to approximate shadcn `AlertAction` behavior.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Validate RTL + narrow layout so icon/title/description remain readable in editor sidebars.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-alert",
        component_panel,
        code_panel,
        notes_panel,
    )
}
