pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret_ui_kit::ui::UiElementSinkExt;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let table_all = cx.local_model_keyed("table_all", || false);
    let table_row_1 = cx.local_model_keyed("table_row_1", || true);
    let table_row_2 = cx.local_model_keyed("table_row_2", || false);
    let table_row_3 = cx.local_model_keyed("table_row_3", || false);

    let table_row = |cx: &mut ElementContext<'_, H>,
                     id: &'static str,
                     role: &'static str,
                     checked: Model<bool>,
                     test_id: &'static str| {
        shadcn::TableRow::build(3, move |cx, out| {
            out.push(
                shadcn::TableCell::new(
                    shadcn::Checkbox::new(checked)
                        .a11y_label(format!("Select {id}"))
                        .test_id(test_id)
                        .into_element(cx),
                )
                .into_element(cx),
            );
            out.push_ui(cx, shadcn::TableCell::build(ui::text(id)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(role)));
        })
        .border_bottom(true)
        .into_element(cx)
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(3, |cx, out| {
                        out.push(
                            shadcn::TableCell::new(
                                shadcn::Checkbox::new(table_all)
                                    .a11y_label("Select all rows")
                                    .test_id("ui-gallery-checkbox-table-all")
                                    .into_element(cx),
                            )
                            .into_element(cx),
                        );
                        out.push(shadcn::TableHead::new("Member").into_element(cx));
                        out.push(shadcn::TableHead::new("Role").into_element(cx));
                    })
                    .border_bottom(true)
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push(table_row(
                    cx,
                    "Alex Johnson",
                    "Owner",
                    table_row_1,
                    "ui-gallery-checkbox-table-row-1",
                ));
                out.push(table_row(
                    cx,
                    "Riley Chen",
                    "Editor",
                    table_row_2,
                    "ui-gallery-checkbox-table-row-2",
                ));
                out.push(table_row(
                    cx,
                    "Morgan Lee",
                    "Viewer",
                    table_row_3,
                    "ui-gallery-checkbox-table-row-3",
                ));
            }),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-table")
}
// endregion: example
