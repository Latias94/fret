pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn table_row<H: UiHost>(
    id: &'static str,
    role: &'static str,
    checked: Model<bool>,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::table_row(3, move |cx| {
        ui::children![
            cx;
            shadcn::table_cell(
                shadcn::Checkbox::new(checked)
                    .a11y_label(format!("Select {id}"))
                    .test_id(test_id),
            ),
            shadcn::table_cell(ui::text(id)),
            shadcn::table_cell(ui::text(role)),
        ]
    })
    .border_bottom(true)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let table_all = cx.local_model_keyed("table_all", || false);
    let table_row_1 = cx.local_model_keyed("table_row_1", || true);
    let table_row_2 = cx.local_model_keyed("table_row_2", || false);
    let table_row_3 = cx.local_model_keyed("table_row_3", || false);

    shadcn::table(|cx| {
        ui::children![
            cx;
            shadcn::table_header(|cx| {
                ui::children![
                    cx;
                    shadcn::table_row(3, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_cell(
                                shadcn::Checkbox::new(table_all)
                                    .a11y_label("Select all rows")
                                    .test_id("ui-gallery-checkbox-table-all"),
                            ),
                            shadcn::table_head("Member"),
                            shadcn::table_head("Role"),
                        ]
                    })
                    .border_bottom(true),
                ]
            }),
            shadcn::table_body(|_cx| {
                vec![
                    table_row(
                        "Alex Johnson",
                        "Owner",
                        table_row_1,
                        "ui-gallery-checkbox-table-row-1",
                    ),
                    table_row(
                        "Riley Chen",
                        "Editor",
                        table_row_2,
                        "ui-gallery-checkbox-table-row-2",
                    ),
                    table_row(
                        "Morgan Lee",
                        "Viewer",
                        table_row_3,
                        "ui-gallery-checkbox-table-row-3",
                    ),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-table")
}
// endregion: example
