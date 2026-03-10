pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret_ui_kit::ui::UiElementSinkExt;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    table_all: Option<Model<bool>>,
    table_row_1: Option<Model<bool>>,
    table_row_2: Option<Model<bool>>,
    table_row_3: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (table_all, table_row_1, table_row_2, table_row_3) = cx.with_state(Models::default, |st| {
        (
            st.table_all.clone(),
            st.table_row_1.clone(),
            st.table_row_2.clone(),
            st.table_row_3.clone(),
        )
    });
    let (table_all, table_row_1, table_row_2, table_row_3) =
        match (table_all, table_row_1, table_row_2, table_row_3) {
            (Some(table_all), Some(table_row_1), Some(table_row_2), Some(table_row_3)) => {
                (table_all, table_row_1, table_row_2, table_row_3)
            }
            _ => {
                let table_all = cx.app.models_mut().insert(false);
                let table_row_1 = cx.app.models_mut().insert(true);
                let table_row_2 = cx.app.models_mut().insert(false);
                let table_row_3 = cx.app.models_mut().insert(false);
                cx.with_state(Models::default, |st| {
                    st.table_all = Some(table_all.clone());
                    st.table_row_1 = Some(table_row_1.clone());
                    st.table_row_2 = Some(table_row_2.clone());
                    st.table_row_3 = Some(table_row_3.clone());
                });
                (table_all, table_row_1, table_row_2, table_row_3)
            }
        };

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
