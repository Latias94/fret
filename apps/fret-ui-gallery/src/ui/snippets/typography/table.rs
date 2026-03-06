pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Table::new(vec![
        shadcn::TableHeader::new(vec![
            shadcn::TableRow::new(
                2,
                vec![
                    shadcn::TableHead::new("King's Treasury").into_element(cx),
                    shadcn::TableHead::new("People's Happiness").into_element(cx),
                ],
            )
            .border_bottom(true)
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::TableBody::new(vec![
            shadcn::TableRow::new(
                2,
                vec![
                    shadcn::TableCell::build(ui::text("Empty")).into_element(cx),
                    shadcn::TableCell::build(ui::text("Overflowing")).into_element(cx),
                ],
            )
            .into_element(cx),
            shadcn::TableRow::new(
                2,
                vec![
                    shadcn::TableCell::build(ui::text("Modest")).into_element(cx),
                    shadcn::TableCell::build(ui::text("Satisfied")).into_element(cx),
                ],
            )
            .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-typography-table")
}
// endregion: example
