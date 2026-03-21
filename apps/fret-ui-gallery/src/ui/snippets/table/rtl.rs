pub const SOURCE: &str = include_str!("rtl.rs");

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
                            shadcn::table_head("الفاتورة")
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
                            shadcn::table_head("الحالة"),
                            shadcn::table_head("الطريقة"),
                            shadcn::table_head("المبلغ").text_align_end(),
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
                                shadcn::table_cell(ui::text("المجموع")).col_span(3),
                                shadcn::table_cell(ui::text("$2,500.00")).text_align_end(),
                            ]
                        })
                        .border_bottom(false),
                    ]
                })
                .into_element(cx),
            );
        }

        children.push(shadcn::table_caption("قائمة بفواتيرك الأخيرة.").into_element(cx));
        children
    })
    .ui()
    .w_full()
    .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl).into_element(cx, |cx| {
        let rows: [(&str, &str, &str, &str); 7] = [
            ("INV001", "مدفوع", "بطاقة ائتمانية", "$250.00"),
            ("INV002", "قيد الانتظار", "PayPal", "$150.00"),
            ("INV003", "غير مدفوع", "تحويل بنكي", "$350.00"),
            ("INV004", "مدفوع", "بطاقة ائتمانية", "$450.00"),
            ("INV005", "مدفوع", "PayPal", "$550.00"),
            ("INV006", "قيد الانتظار", "تحويل بنكي", "$200.00"),
            ("INV007", "غير مدفوع", "بطاقة ائتمانية", "$300.00"),
        ];

        make_invoice_table(&rows, true, "ui-gallery-table-rtl").into_element(cx)
    })
}

// endregion: example
