pub const SOURCE: &str = include_str!("footer.rs");

// region: example
use fret_app::App;
use fret_core::{FontWeight, Px};
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn make_invoice_table(
    cx: &mut ElementContext<'_, App>,
    rows: &[(&'static str, &'static str, &'static str, &'static str)],
    include_footer: bool,
    test_id: &'static str,
) -> AnyElement {
    let invoice_w = Px(100.0);

    let header = shadcn::TableHeader::new(vec![
        shadcn::TableRow::new(
            4,
            vec![
                shadcn::TableHead::new("Invoice")
                    .refine_layout(LayoutRefinement::default().w_px(invoice_w))
                    .into_element(cx),
                shadcn::TableHead::new("Status").into_element(cx),
                shadcn::TableHead::new("Method").into_element(cx),
                shadcn::TableHead::new("Amount")
                    .text_align_end()
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
        .map(|(invoice, status, method, amount)| {
            let invoice_slug = invoice.to_ascii_lowercase();
            let row_test_id = format!("{test_id}-row-{invoice_slug}");
            shadcn::TableRow::new(
                4,
                vec![
                    {
                        let invoice_text = ui::text(cx, invoice)
                            .font_weight(FontWeight::MEDIUM)
                            .into_element(cx);
                        shadcn::TableCell::new(invoice_text)
                            .refine_layout(LayoutRefinement::default().w_px(invoice_w))
                            .into_element(cx)
                    },
                    shadcn::TableCell::new(cx.text(status)).into_element(cx),
                    shadcn::TableCell::new(cx.text(method)).into_element(cx),
                    shadcn::TableCell::new(cx.text(amount))
                        .text_align_end()
                        .into_element(cx),
                ],
            )
            .into_element(cx)
            .test_id(row_test_id)
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
                        .into_element(cx),
                    shadcn::TableCell::new(cx.text("$2,500.00"))
                        .text_align_end()
                        .into_element(cx),
                ],
            )
            .border_bottom(false)
            .into_element(cx),
        ])
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
        ("INV001", "Paid", "Credit Card", "$250.00"),
        ("INV002", "Pending", "PayPal", "$150.00"),
        ("INV003", "Unpaid", "Bank Transfer", "$350.00"),
    ];

    make_invoice_table(cx, &invoices, true, "ui-gallery-table-footer")
}

// endregion: example
