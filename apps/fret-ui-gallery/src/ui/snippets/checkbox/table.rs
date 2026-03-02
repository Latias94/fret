pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    table_all: Model<bool>,
    table_row_1: Model<bool>,
    table_row_2: Model<bool>,
    table_row_3: Model<bool>,
) -> AnyElement {
    let table_row = |cx: &mut ElementContext<'_, H>,
                     id: &'static str,
                     role: &'static str,
                     checked: Model<bool>,
                     test_id: &'static str| {
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(
                    shadcn::Checkbox::new(checked)
                        .a11y_label(format!("Select {id}"))
                        .test_id(test_id)
                        .into_element(cx),
                )
                .into_element(cx),
                shadcn::TableCell::new(cx.text(id)).into_element(cx),
                shadcn::TableCell::new(cx.text(role)).into_element(cx),
            ],
        )
        .border_bottom(true)
        .into_element(cx)
    };

    shadcn::Table::new(vec![
        shadcn::TableHeader::new(vec![
            shadcn::TableRow::new(
                3,
                vec![
                    shadcn::TableCell::new(
                        shadcn::Checkbox::new(table_all)
                            .a11y_label("Select all rows")
                            .test_id("ui-gallery-checkbox-table-all")
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::TableHead::new("Member").into_element(cx),
                    shadcn::TableHead::new("Role").into_element(cx),
                ],
            )
            .border_bottom(true)
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::TableBody::new(vec![
            table_row(
                cx,
                "Alex Johnson",
                "Owner",
                table_row_1,
                "ui-gallery-checkbox-table-row-1",
            ),
            table_row(
                cx,
                "Riley Chen",
                "Editor",
                table_row_2,
                "ui-gallery-checkbox-table-row-2",
            ),
            table_row(
                cx,
                "Morgan Lee",
                "Viewer",
                table_row_3,
                "ui-gallery-checkbox-table-row-3",
            ),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-table")
}
// endregion: example
