pub const SOURCE: &str = include_str!("children.rs");

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
                            shadcn::table_cell(ui::text("INV101").font_weight(FontWeight::MEDIUM))
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
                            shadcn::table_cell(ui::text("Paid")),
                            shadcn::table_cell(ui::text("Credit Card")),
                            shadcn::table_cell(ui::text("$120.00")).text_align_end(),
                        ]
                    }),
                    shadcn::table_row(4, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_cell(ui::text("INV102").font_weight(FontWeight::MEDIUM))
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
                            shadcn::table_cell(ui::text("Pending")),
                            shadcn::table_cell(ui::text("Wire Transfer")),
                            shadcn::table_cell(ui::text("$340.00")).text_align_end(),
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
