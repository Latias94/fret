pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::{FontWeight, Px};
use fret_ui_kit::ui::UiElementSinkExt;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let invoice_w = Px(100.0);

    shadcn::Table::build(|cx, out| {
        out.push(
            shadcn::TableCaption::new("A list of your recent invoices.").into_element(cx),
        );
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
                out.push(
                    shadcn::TableRow::build(4, |cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::TableCell::build(ui::text("INV001").font_weight(FontWeight::MEDIUM))
                                .refine_layout(LayoutRefinement::default().w_px(invoice_w)),
                        );
                        out.push_ui(cx, shadcn::TableCell::build(ui::text("Paid")));
                        out.push_ui(cx, shadcn::TableCell::build(ui::text("Credit Card")));
                        out.push_ui(
                            cx,
                            shadcn::TableCell::build(ui::text("$250.00")).text_align_end(),
                        );
                    })
                    .into_element(cx),
                );
            }),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-table-usage")
}
// endregion: example
