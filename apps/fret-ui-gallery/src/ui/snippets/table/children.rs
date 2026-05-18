pub const SOURCE: &str = include_str!("children.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
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
                            shadcn::table_head_children(|cx| {
                                ui::children![
                                    cx;
                                    ui::text("Status "),
                                    shadcn::Badge::new("Live")
                                        .variant(shadcn::BadgeVariant::Outline),
                                ]
                            }),
                            shadcn::table_head("Method"),
                            shadcn::table_head_children(|cx| {
                                ui::children![
                                    cx;
                                    ui::text("Amount "),
                                    ui::text("(USD)"),
                                ]
                            })
                            .text_align_end(),
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
                            shadcn::table_cell(super::table_cell_text_emphasis(cx, "INV101"))
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
                            shadcn::table_cell(super::table_cell_text(cx, "Paid")),
                            shadcn::table_cell(super::table_cell_text(cx, "Credit Card")),
                            shadcn::table_cell(super::table_cell_text(cx, "$120.00")).text_align_end(),
                        ]
                    }),
                    shadcn::table_row(4, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_cell(super::table_cell_text_emphasis(cx, "INV102"))
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
                            shadcn::table_cell(super::table_cell_text(cx, "Pending")),
                            shadcn::table_cell(super::table_cell_text(cx, "Wire Transfer")),
                            shadcn::table_cell(super::table_cell_text(cx, "$340.00")).text_align_end(),
                        ]
                    }),
                ]
            }),
            shadcn::table_caption_children(|cx| {
                ui::children![
                    cx;
                    ui::text("A list of your recent invoices."),
                    ui::text("Use the children helpers when the compact text constructors are too narrow."),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-table-children")
}
// endregion: example
