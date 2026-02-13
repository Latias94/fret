use super::super::super::*;

pub(in crate::ui) fn preview_table(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct TableModels {
        actions_open_1: Option<Model<bool>>,
        actions_open_2: Option<Model<bool>>,
        actions_open_3: Option<Model<bool>>,
    }

    let state = cx.with_state(TableModels::default, |st| st.clone());
    let (actions_open_1, actions_open_2, actions_open_3) = match (
        state.actions_open_1,
        state.actions_open_2,
        state.actions_open_3,
    ) {
        (Some(open_1), Some(open_2), Some(open_3)) => (open_1, open_2, open_3),
        _ => {
            let open_1 = cx.app.models_mut().insert(false);
            let open_2 = cx.app.models_mut().insert(false);
            let open_3 = cx.app.models_mut().insert(false);
            cx.with_state(TableModels::default, |st| {
                st.actions_open_1 = Some(open_1.clone());
                st.actions_open_2 = Some(open_2.clone());
                st.actions_open_3 = Some(open_3.clone());
            });
            (open_1, open_2, open_3)
        }
    };

    let invoice_w = fret_core::Px(128.0);
    let status_w = fret_core::Px(120.0);
    let method_w = fret_core::Px(180.0);
    let amount_w = fret_core::Px(132.0);

    let invoices: [(&str, &str, &str, &str); 7] = [
        ("INV001", "Paid", "$250.00", "Credit Card"),
        ("INV002", "Pending", "$150.00", "PayPal"),
        ("INV003", "Unpaid", "$350.00", "Bank Transfer"),
        ("INV004", "Paid", "$450.00", "Credit Card"),
        ("INV005", "Paid", "$550.00", "PayPal"),
        ("INV006", "Pending", "$200.00", "Bank Transfer"),
        ("INV007", "Unpaid", "$300.00", "Credit Card"),
    ];

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

    let align_end = |cx: &mut ElementContext<'_, App>, child: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_end(),
            move |_cx| [child],
        )
    };

    let make_invoice_table =
        |cx: &mut ElementContext<'_, App>,
         rows: &[(&'static str, &'static str, &'static str, &'static str)],
         include_footer: bool,
         test_id: &'static str| {
            let header = shadcn::TableHeader::new(vec![
                shadcn::TableRow::new(
                    4,
                    vec![
                        shadcn::TableHead::new("Invoice")
                            .refine_layout(LayoutRefinement::default().w_px(invoice_w))
                            .into_element(cx),
                        shadcn::TableHead::new("Status")
                            .refine_layout(LayoutRefinement::default().w_px(status_w))
                            .into_element(cx),
                        shadcn::TableHead::new("Method")
                            .refine_layout(LayoutRefinement::default().w_px(method_w))
                            .into_element(cx),
                        shadcn::TableHead::new("Amount")
                            .refine_layout(LayoutRefinement::default().w_px(amount_w))
                            .into_element(cx),
                    ],
                )
                .border_bottom(true)
                .into_element(cx),
            ])
            .into_element(cx);

            let body_rows = rows
                .iter()
                .copied()
                .map(|(invoice, status, amount, method)| {
                    shadcn::TableRow::new(
                        4,
                        vec![
                            shadcn::TableCell::new(cx.text(invoice))
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w))
                                .into_element(cx),
                            shadcn::TableCell::new(cx.text(status))
                                .refine_layout(LayoutRefinement::default().w_px(status_w))
                                .into_element(cx),
                            shadcn::TableCell::new(cx.text(method))
                                .refine_layout(LayoutRefinement::default().w_px(method_w))
                                .into_element(cx),
                            {
                                let amount_text = cx.text(amount);
                                shadcn::TableCell::new(align_end(cx, amount_text))
                                    .refine_layout(LayoutRefinement::default().w_px(amount_w))
                                    .into_element(cx)
                            },
                        ],
                    )
                    .into_element(cx)
                })
                .collect::<Vec<_>>();

            let body = shadcn::TableBody::new(body_rows).into_element(cx);

            let mut children = vec![header, body];
            if include_footer {
                let footer = shadcn::TableFooter::new(vec![
                    shadcn::TableRow::new(
                        4,
                        vec![
                            shadcn::TableCell::new(cx.text("Total"))
                                .col_span(3)
                                .refine_layout(
                                    LayoutRefinement::default()
                                        .w_px(invoice_w + status_w + method_w),
                                )
                                .into_element(cx),
                            {
                                let total_amount = cx.text("$2,500.00");
                                shadcn::TableCell::new(align_end(cx, total_amount))
                                    .refine_layout(LayoutRefinement::default().w_px(amount_w))
                                    .into_element(cx)
                            },
                        ],
                    )
                    .border_bottom(false)
                    .into_element(cx),
                ])
                .into_element(cx);
                children.push(footer);
            }

            children.push(
                shadcn::TableCaption::new("A list of your recent invoices.").into_element(cx),
            );

            shadcn::Table::new(children)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id(test_id)
        };

    let demo = {
        let table = make_invoice_table(cx, &invoices, true, "ui-gallery-table-demo");
        let table_shell = shell(cx, table);
        let body = centered(cx, table_shell);
        section(cx, "Demo", body)
    };

    let footer = {
        let table = make_invoice_table(cx, &invoices[..3], true, "ui-gallery-table-footer");
        let table_shell = shell(cx, table);
        let body = centered(cx, table_shell);
        section(cx, "Footer", body)
    };

    let actions = {
        let action_row = |cx: &mut ElementContext<'_, App>,
                          product: &'static str,
                          price: &'static str,
                          open_model: Model<bool>,
                          key: &'static str| {
            let trigger_id = format!("ui-gallery-table-actions-trigger-{key}");
            let dropdown = shadcn::DropdownMenu::new(open_model.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("?")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .toggle_model(open_model.clone())
                        .test_id(trigger_id.clone())
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Edit")),
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Duplicate")),
                        shadcn::DropdownMenuEntry::Separator,
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Delete").variant(
                                shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                            ),
                        ),
                    ]
                },
            );

            shadcn::TableRow::new(
                3,
                vec![
                    shadcn::TableCell::new(cx.text(product)).into_element(cx),
                    shadcn::TableCell::new(cx.text(price)).into_element(cx),
                    {
                        let action_cell = align_end(cx, dropdown);
                        shadcn::TableCell::new(action_cell).into_element(cx)
                    },
                ],
            )
            .into_element(cx)
        };

        let table = shadcn::Table::new(vec![
            shadcn::TableHeader::new(vec![
                shadcn::TableRow::new(
                    3,
                    vec![
                        shadcn::TableHead::new("Product")
                            .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
                            .into_element(cx),
                        shadcn::TableHead::new("Price")
                            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
                            .into_element(cx),
                        shadcn::TableHead::new("Actions")
                            .refine_layout(LayoutRefinement::default().w_px(Px(120.0)))
                            .into_element(cx),
                    ],
                )
                .border_bottom(true)
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::TableBody::new(vec![
                action_row(cx, "Gaming Mouse", "$129.99", actions_open_1, "row-1"),
                action_row(cx, "Mechanical Keyboard", "$89.99", actions_open_2, "row-2"),
                action_row(cx, "4K Monitor", "$299.99", actions_open_3, "row-3"),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-table-actions");

        let table_shell = shell(cx, table);
        let body = centered(cx, table_shell);
        section(cx, "Actions", body)
    };

    let rtl = {
        let rtl_table = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let rows: [(&str, &str, &str, &str); 3] = [
                    ("INV001", "Paid", "$250.00", "Credit Card"),
                    ("INV002", "Pending", "$150.00", "PayPal"),
                    ("INV003", "Unpaid", "$350.00", "Bank Transfer"),
                ];
                make_invoice_table(cx, &rows, true, "ui-gallery-table-rtl")
            },
        );

        let table_shell = shell(cx, rtl_table);
        let body = centered(cx, table_shell);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("A responsive table component."),
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![demo, footer, actions, rtl],
        ),
    ]
}
