pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{FontWeight, Px};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let invoice_w = Px(100.0);

    shadcn::table(|cx| {
        ui::children![
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
            shadcn::table_body(|cx| {
                ui::children![
                    cx;
                    shadcn::table_row(4, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_cell(ui::text("INV001").font_weight(FontWeight::MEDIUM))
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
                            shadcn::table_cell(ui::text("Paid")),
                            shadcn::table_cell(ui::text("Credit Card")),
                            shadcn::table_cell(ui::text("$250.00")).text_align_end(),
                        ]
                    }),
                ]
            }),
            shadcn::table_caption("A list of your recent invoices."),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-table-usage")
}
// endregion: example
