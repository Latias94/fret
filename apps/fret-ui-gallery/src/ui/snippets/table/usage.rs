pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::{FontWeight, Px};
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let invoice_w = Px(100.0);

    let header = shadcn::TableHeader::new([shadcn::TableRow::new(
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
    .into_element(cx)])
    .into_element(cx);

    let invoice_text = ui::text("INV001")
        .font_weight(FontWeight::MEDIUM)
        .into_element(cx);
    let body = shadcn::TableBody::new([shadcn::TableRow::new(
        4,
        vec![
            shadcn::TableCell::new(invoice_text)
                .refine_layout(LayoutRefinement::default().w_px(invoice_w))
                .into_element(cx),
            shadcn::TableCell::new(cx.text("Paid")).into_element(cx),
            shadcn::TableCell::new(cx.text("Credit Card")).into_element(cx),
            shadcn::TableCell::new(cx.text("$250.00"))
                .text_align_end()
                .into_element(cx),
        ],
    )
    .into_element(cx)])
    .into_element(cx);

    shadcn::Table::new([
        shadcn::TableCaption::new("A list of your recent invoices.").into_element(cx),
        header,
        body,
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-table-usage")
}
// endregion: example
