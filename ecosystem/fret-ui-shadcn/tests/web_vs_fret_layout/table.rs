use super::*;
use fret_ui_shadcn::facade as shadcn;

fn text_sm_nowrap<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let size = theme
        .metric_by_key("component.text.sm_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.text.sm_line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    ui::text(text)
        .text_size_px(size)
        .line_height_px(line_height)
        .nowrap()
        .into_element(cx)
}

#[test]
fn web_vs_fret_layout_table_demo_row_heights_and_caption_gap() {
    let web = read_web_golden("table-demo");
    let theme = web_theme(&web);

    let web_caption = web_find_by_tag_and_text(&theme.root, "caption", "recent invoices")
        .or_else(|| find_first(&theme.root, &|n| n.tag == "caption"))
        .expect("web caption");
    let web_header_row = find_first(&theme.root, &|n| n.tag == "thead")
        .and_then(|thead| thead.children.iter().find(|n| n.tag == "tr"))
        .expect("web header tr");
    let web_body_row = find_first(&theme.root, &|n| n.tag == "tbody")
        .and_then(|tbody| tbody.children.iter().find(|n| n.tag == "tr"))
        .expect("web body tr");
    let web_footer_row = find_first(&theme.root, &|n| n.tag == "tfoot")
        .and_then(|tfoot| tfoot.children.iter().find(|n| n.tag == "tr"))
        .expect("web footer tr");

    let web_caption_gap = web_caption.rect.y - (web_footer_row.rect.y + web_footer_row.rect.h);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let head_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:table-demo:header-row")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    shadcn::TableRow::new(
                        4,
                        vec![
                            shadcn::TableHead::new("Invoice").into_element(cx),
                            shadcn::TableHead::new("Status").into_element(cx),
                            shadcn::TableHead::new("Method").into_element(cx),
                            shadcn::TableHead::new("Amount").into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        let first_body_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:table-demo:body-row-0")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    shadcn::TableRow::new(
                        4,
                        vec![
                            shadcn::TableCell::new(text_sm_nowrap(cx, "INV001")).into_element(cx),
                            shadcn::TableCell::new(text_sm_nowrap(cx, "Paid")).into_element(cx),
                            shadcn::TableCell::new(text_sm_nowrap(cx, "Credit Card"))
                                .into_element(cx),
                            shadcn::TableCell::new(text_sm_nowrap(cx, "$250.00")).into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        let other_rows = [
            ("INV002", "Pending", "PayPal", "$150.00"),
            ("INV003", "Unpaid", "Bank Transfer", "$350.00"),
            ("INV004", "Paid", "Credit Card", "$450.00"),
            ("INV005", "Paid", "PayPal", "$550.00"),
            ("INV006", "Pending", "Bank Transfer", "$200.00"),
            ("INV007", "Unpaid", "Credit Card", "$300.00"),
        ]
        .into_iter()
        .map(|(invoice, status, method, amount)| {
            shadcn::TableRow::new(
                4,
                vec![
                    shadcn::TableCell::new(text_sm_nowrap(cx, invoice)).into_element(cx),
                    shadcn::TableCell::new(text_sm_nowrap(cx, status)).into_element(cx),
                    shadcn::TableCell::new(text_sm_nowrap(cx, method)).into_element(cx),
                    shadcn::TableCell::new(text_sm_nowrap(cx, amount)).into_element(cx),
                ],
            )
            .into_element(cx)
        })
        .collect::<Vec<_>>();

        let footer_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:table-demo:footer-row")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    shadcn::TableRow::new(
                        4,
                        vec![
                            shadcn::TableCell::new(text_sm_nowrap(cx, "Total"))
                                .col_span(3)
                                .into_element(cx),
                            shadcn::TableCell::new(text_sm_nowrap(cx, "$2,500.00"))
                                .into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        let caption = shadcn::TableCaption::new("A list of your recent invoices.")
            .into_element(cx)
            .test_id("Golden:table-demo:caption");

        vec![
            shadcn::Table::new(vec![
                shadcn::TableHeader::new(vec![head_row]).into_element(cx),
                shadcn::TableBody::new({
                    let mut rows = Vec::new();
                    rows.push(first_body_row);
                    rows.extend(other_rows);
                    rows
                })
                .into_element(cx),
                shadcn::TableFooter::new(vec![footer_row]).into_element(cx),
                caption,
            ])
            .into_element(cx),
        ]
    });

    let header_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:table-demo:header-row"),
    )
    .expect("fret header row");
    let body_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:table-demo:body-row-0"),
    )
    .expect("fret first body row");
    let footer_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:table-demo:footer-row"),
    )
    .expect("fret footer row");

    assert_close_px(
        "table-demo header row x",
        header_row.bounds.origin.x,
        body_row.bounds.origin.x.0,
        1.0,
    );
    assert_close_px(
        "table-demo header row width",
        header_row.bounds.size.width,
        body_row.bounds.size.width.0,
        1.0,
    );
    assert_close_px(
        "table-demo footer row x",
        footer_row.bounds.origin.x,
        body_row.bounds.origin.x.0,
        1.0,
    );
    assert_close_px(
        "table-demo footer row width",
        footer_row.bounds.size.width,
        body_row.bounds.size.width.0,
        1.0,
    );

    assert_close_px(
        "table-demo header row height",
        header_row.bounds.size.height,
        web_header_row.rect.h,
        1.0,
    );
    assert_close_px(
        "table-demo first body row height",
        body_row.bounds.size.height,
        web_body_row.rect.h,
        1.0,
    );
    assert_close_px(
        "table-demo footer row height",
        footer_row.bounds.size.height,
        web_footer_row.rect.h,
        2.0,
    );

    let caption = find_by_test_id(&snap, "Golden:table-demo:caption");
    let fret_caption_gap = caption.bounds.origin.y.0
        - (footer_row.bounds.origin.y.0 + footer_row.bounds.size.height.0);
    assert_close_px(
        "table-demo caption gap",
        Px(fret_caption_gap),
        web_caption_gap,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_data_table_demo_row_height_and_action_button_size() {
    let web = read_web_golden("data-table-demo");
    let theme = web_theme(&web);

    let web_header_row = find_first(&theme.root, &|n| n.tag == "thead")
        .and_then(|thead| thead.children.iter().find(|n| n.tag == "tr"))
        .expect("web header tr");
    let web_body_row = find_first(&theme.root, &|n| n.tag == "tbody")
        .and_then(|tbody| tbody.children.iter().find(|n| n.tag == "tr"))
        .expect("web body tr");

    let web_select_row = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|r| r == "checkbox")
            && n.attrs.get("aria-label").is_some_and(|v| v == "Select row")
    })
    .expect("web select row checkbox");

    let web_open_menu = find_first(&theme.root, &|n| {
        n.tag == "button" && contains_text(n, "Open menu")
    })
    .expect("web open menu button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let header_select_all: Model<bool> = cx.app.models_mut().insert(false);
        let row_select: Model<bool> = cx.app.models_mut().insert(false);

        let header_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:data-table-demo:header-row")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    shadcn::TableRow::new(
                        5,
                        vec![
                            shadcn::TableCell::new(
                                shadcn::Checkbox::new(header_select_all.clone())
                                    .a11y_label("Select all")
                                    .into_element(cx),
                            )
                            .into_element(cx),
                            shadcn::TableHead::new("Status").into_element(cx),
                            shadcn::TableHead::new("Email").into_element(cx),
                            shadcn::TableHead::new("Amount").into_element(cx),
                            shadcn::TableHead::new("").into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        let body_row = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:data-table-demo:row-0")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    shadcn::TableRow::new(
                        5,
                        vec![
                            shadcn::TableCell::new(
                                shadcn::Checkbox::new(row_select.clone())
                                    .a11y_label("Select row")
                                    .into_element(cx),
                            )
                            .into_element(cx),
                            shadcn::TableCell::new(decl_text::text_sm(cx, "success"))
                                .into_element(cx),
                            shadcn::TableCell::new(decl_text::text_sm(cx, "ken99@example.com"))
                                .into_element(cx),
                            shadcn::TableCell::new(decl_text::text_sm(cx, "$316.00"))
                                .into_element(cx),
                            shadcn::TableCell::new(
                                shadcn::Button::new("Open menu")
                                    .variant(shadcn::ButtonVariant::Ghost)
                                    .size(shadcn::ButtonSize::IconSm)
                                    .children(vec![
                                        shadcn::Spinner::new().speed(0.0).into_element(cx),
                                    ])
                                    .into_element(cx),
                            )
                            .into_element(cx),
                        ],
                    )
                    .into_element(cx),
                ]
            },
        );

        vec![
            shadcn::Table::new(vec![
                shadcn::TableHeader::new(vec![header_row]).into_element(cx),
                shadcn::TableBody::new(vec![body_row]).into_element(cx),
            ])
            .into_element(cx),
        ]
    });

    let header_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:data-table-demo:header-row"),
    )
    .expect("fret header row");
    let body_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:data-table-demo:row-0"),
    )
    .expect("fret body row");

    let select_row = find_semantics(&snap, SemanticsRole::Checkbox, Some("Select row"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Checkbox, None))
        .expect("fret select row checkbox");
    let open_menu = find_semantics(&snap, SemanticsRole::Button, Some("Open menu"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret open menu button");

    assert_close_px(
        "data-table-demo header row height",
        header_row.bounds.size.height,
        web_header_row.rect.h,
        1.0,
    );
    assert_close_px(
        "data-table-demo row height",
        body_row.bounds.size.height,
        web_body_row.rect.h,
        2.0,
    );

    assert_close_px(
        "data-table-demo select row checkbox width",
        select_row.bounds.size.width,
        web_select_row.rect.w,
        1.0,
    );
    assert_close_px(
        "data-table-demo select row checkbox height",
        select_row.bounds.size.height,
        web_select_row.rect.h,
        1.0,
    );

    assert_close_px(
        "data-table-demo open menu button width",
        open_menu.bounds.size.width,
        web_open_menu.rect.w,
        1.0,
    );
    assert_close_px(
        "data-table-demo open menu button height",
        open_menu.bounds.size.height,
        web_open_menu.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_data_table_demo_empty_state_cell_spans_table_width() {
    let web = read_web_golden("data-table-demo.empty");
    let theme = web_theme(&web);

    let web_table = find_first(&theme.root, &|n| n.tag == "table").expect("web table");
    let web_empty_td =
        web_find_by_tag_and_text(&theme.root, "td", "No results").expect("web empty state td");

    let expected_rel = WebRect {
        x: web_empty_td.rect.x - web_table.rect.x,
        y: web_empty_td.rect.y - web_table.rect.y,
        w: web_empty_td.rect.w,
        h: web_empty_td.rect.h,
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();

        let empty_td = shadcn::TableCell::new(decl_text::text_sm(cx, "No results."))
            .col_span(5)
            .refine_layout(LayoutRefinement::default().h_px(Px(web_empty_td.rect.h)))
            .into_element(cx);

        let table = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:data-table-demo.empty:table")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    shadcn::Table::new(vec![
                        shadcn::TableHeader::new(vec![
                            shadcn::TableRow::new(
                                5,
                                vec![
                                    shadcn::TableHead::new("").into_element(cx),
                                    shadcn::TableHead::new("Status").into_element(cx),
                                    shadcn::TableHead::new("Email").into_element(cx),
                                    shadcn::TableHead::new("Amount").into_element(cx),
                                    shadcn::TableHead::new("").into_element(cx),
                                ],
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::TableBody::new(vec![
                            shadcn::TableRow::new(5, vec![empty_td])
                                .border_bottom(false)
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                ]
            },
        );

        vec![cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_px(Px(web_table.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![table],
        )]
    });

    let _ = snap;

    // We render only the table subtree in Fret, so the "relative to table" rect becomes an
    // absolute rect in our test harness.
    let expected_abs = WebRect {
        x: expected_rel.x,
        y: expected_rel.y,
        w: expected_rel.w,
        h: expected_rel.h,
    };

    let (td_id, td_bounds) = if let Some(found) =
        find_node_with_bounds_close(&ui, root, expected_abs, 2.0)
    {
        found
    } else {
        let mut nodes = Vec::new();
        collect_subtree_nodes(&ui, root, &mut nodes);

        let mut best: Option<(NodeId, Rect, f32)> = None;
        for id in nodes {
            let Some(bounds) = ui.debug_node_bounds(id) else {
                continue;
            };
            let score = (bounds.origin.x.0 - expected_abs.x).abs()
                + (bounds.origin.y.0 - expected_abs.y).abs()
                + (bounds.size.width.0 - expected_abs.w).abs()
                + (bounds.size.height.0 - expected_abs.h).abs();
            if best.as_ref().is_none_or(|(_, _, s)| score < *s) {
                best = Some((id, bounds, score));
            }
        }

        let (id, b, score) = best.expect("no debug bounds in subtree");
        panic!(
            "fret td bounds not found; bestCandidate id={id:?} bounds={b:?} score={score} expected={expected_abs:?}"
        );
    };
    let _ = td_id;

    assert_rect_close_px("data-table-demo.empty td", td_bounds, expected_abs, 2.0);
}
