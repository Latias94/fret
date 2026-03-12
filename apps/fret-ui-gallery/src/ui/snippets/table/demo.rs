pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::UiCx;
use fret_core::{FontWeight, Px};
use fret_ui_kit::{IntoUiElement, ui::UiElementSinkExt};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn make_invoice_table(
    rows: &[(&'static str, &'static str, &'static str, &'static str)],
    include_footer: bool,
    test_id: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    // shadcn docs: only the first column is fixed-width (`w-[100px]`).
    let invoice_w = Px(100.0);
    let rows = rows.to_vec();

    let body_row = move |cx: &mut UiCx<'_>,
                         invoice: &'static str,
                         status: &'static str,
                         method: &'static str,
                         amount: &'static str| {
        let invoice_slug = invoice.to_ascii_lowercase();
        let row_test_id = format!("{test_id}-row-{invoice_slug}");
        shadcn::TableRow::build(4, move |cx, out| {
            out.push_ui(
                cx,
                shadcn::TableCell::build(ui::text(invoice).font_weight(FontWeight::MEDIUM))
                    .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
            );
            out.push_ui(cx, shadcn::TableCell::build(ui::text(status)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(method)));
            out.push_ui(
                cx,
                shadcn::TableCell::build(ui::text(amount)).text_align_end(),
            );
        })
        .into_element(cx)
        .test_id(row_test_id)
    };

    shadcn::Table::build(move |cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(4, |cx, out| {
                        out.push(
                            shadcn::TableHead::new("Invoice")
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w))
                                .into_element(cx),
                        );
                        out.push(shadcn::TableHead::new("Status").into_element(cx));
                        out.push(shadcn::TableHead::new("Method").into_element(cx));
                        out.push(
                            shadcn::TableHead::new("Amount")
                                .text_align_end()
                                .into_element(cx),
                        );
                    })
                    .border_bottom(true)
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                for (invoice, status, method, amount) in rows.iter().copied() {
                    out.push(body_row(cx, invoice, status, method, amount));
                }
            }),
        );
        if include_footer {
            out.push_ui(
                cx,
                shadcn::TableFooter::build(|cx, out| {
                    out.push(
                        shadcn::TableRow::build(4, |cx, out| {
                            out.push(
                                shadcn::TableCell::build(ui::text("Total"))
                                    .col_span(3)
                                    .into_element(cx),
                            );
                            out.push(
                                shadcn::TableCell::build(ui::text("$2,500.00"))
                                    .text_align_end()
                                    .into_element(cx),
                            );
                        })
                        .border_bottom(false)
                        .into_element(cx),
                    );
                }),
            );
        }
        out.push(shadcn::TableCaption::new("A list of your recent invoices.").into_element(cx));
    })
    .ui()
    .w_full()
    .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    // Tuple order matches shadcn docs: (invoice, status, method, amount).
    let invoices: [(&str, &str, &str, &str); 7] = [
        ("INV001", "Paid", "Credit Card", "$250.00"),
        ("INV002", "Pending", "PayPal", "$150.00"),
        ("INV003", "Unpaid", "Bank Transfer", "$350.00"),
        ("INV004", "Paid", "Credit Card", "$450.00"),
        ("INV005", "Paid", "PayPal", "$550.00"),
        ("INV006", "Pending", "Bank Transfer", "$200.00"),
        ("INV007", "Unpaid", "Credit Card", "$300.00"),
    ];

    make_invoice_table(&invoices, true, "ui-gallery-table-demo").into_element(cx)
}

// endregion: example
