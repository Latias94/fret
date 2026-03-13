pub const SOURCE: &str = include_str!("footer.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_core::{FontWeight, Px};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn body_row(
    invoice_w: Px,
    invoice: &'static str,
    status: &'static str,
    method: &'static str,
    amount: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let invoice_slug = invoice.to_ascii_lowercase();
    let row_test_id = Arc::<str>::from(format!("{test_id}-row-{invoice_slug}"));

    shadcn::table_row(4, move |cx| {
        ui::children![
            cx;
            shadcn::table_cell(ui::text(invoice).font_weight(FontWeight::MEDIUM))
                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
            shadcn::table_cell(ui::text(status)),
            shadcn::table_cell(ui::text(method)),
            shadcn::table_cell(ui::text(amount)).text_align_end(),
        ]
    })
    .test_id(row_test_id)
}

fn make_invoice_table(
    rows: &[(&'static str, &'static str, &'static str, &'static str)],
    include_footer: bool,
    test_id: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let invoice_w = Px(100.0);
    let rows = rows.to_vec();

    shadcn::table(move |cx| {
        let mut children = ui::children![
            cx;
            shadcn::table_header(|cx| {
                ui::children![
                    cx;
                    shadcn::table_row(4, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_head("Invoice")
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
                            shadcn::table_head("Status"),
                            shadcn::table_head("Method"),
                            shadcn::table_head("Amount").text_align_end(),
                        ]
                    })
                    .border_bottom(true),
                ]
            }),
            shadcn::table_body(move |_cx| {
                rows.iter().copied().map(move |(invoice, status, method, amount)| {
                    body_row(invoice_w, invoice, status, method, amount, test_id)
                })
                .collect::<Vec<_>>()
            }),
        ];

        if include_footer {
            children.push(
                shadcn::table_footer(|cx| {
                    ui::children![
                        cx;
                        shadcn::table_row(4, |cx| {
                            ui::children![
                                cx;
                                shadcn::table_cell(ui::text("Total")).col_span(3),
                                shadcn::table_cell(ui::text("$2,500.00")).text_align_end(),
                            ]
                        })
                        .border_bottom(false),
                    ]
                })
                .into_element(cx),
            );
        }

        children.push(shadcn::table_caption("A list of your recent invoices.").into_element(cx));
        children
    })
    .ui()
    .w_full()
    .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let invoices: [(&str, &str, &str, &str); 3] = [
        ("INV001", "Paid", "Credit Card", "$250.00"),
        ("INV002", "Pending", "PayPal", "$150.00"),
        ("INV003", "Unpaid", "Bank Transfer", "$350.00"),
    ];

    make_invoice_table(&invoices, true, "ui-gallery-table-footer").into_element(cx)
}

// endregion: example
