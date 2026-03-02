// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn align_end(cx: &mut ElementContext<'_, App>, child: AnyElement) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_end(),
        move |_cx| [child],
    )
}

fn make_invoice_table(
    cx: &mut ElementContext<'_, App>,
    rows: &[(&'static str, &'static str, &'static str, &'static str)],
    include_footer: bool,
    test_id: &'static str,
) -> AnyElement {
    let invoice_w = Px(128.0);
    let status_w = Px(120.0);
    let method_w = Px(180.0);
    let amount_w = Px(132.0);

    let header = shadcn::TableHeader::new(vec![shadcn::TableRow::new(
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
    .into_element(cx)])
    .into_element(cx);

    let body_rows = rows
        .iter()
        .copied()
        .map(|(invoice, status, amount, method)| {
            let invoice_slug = invoice.to_ascii_lowercase();
            let row_test_id = format!("{test_id}-row-{invoice_slug}");
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
            .test_id(row_test_id)
        })
        .collect::<Vec<_>>();

    let body = shadcn::TableBody::new(body_rows).into_element(cx);

    let mut children = vec![header, body];
    if include_footer {
        let footer = shadcn::TableFooter::new(vec![shadcn::TableRow::new(
            4,
            vec![
                shadcn::TableCell::new(cx.text("Total"))
                    .col_span(3)
                    .refine_layout(
                        LayoutRefinement::default().w_px(invoice_w + status_w + method_w),
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
        .into_element(cx)])
        .into_element(cx);
        children.push(footer);
    }

    children.push(shadcn::TableCaption::new("A list of your recent invoices.").into_element(cx));

    shadcn::Table::new(children)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id(test_id)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let invoices: [(&str, &str, &str, &str); 3] = [
        ("INV001", "Paid", "$250.00", "Credit Card"),
        ("INV002", "Pending", "$150.00", "PayPal"),
        ("INV003", "Unpaid", "$350.00", "Bank Transfer"),
    ];

    make_invoice_table(cx, &invoices, true, "ui-gallery-table-footer")
}

// endregion: example

