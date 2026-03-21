pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::primitives::checkbox::CheckedState;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

mod act {
    fret::actions!([ToggleAllRows = "ui-gallery.checkbox.table.toggle_all_rows.v1"]);
}

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
    let table_row_1 = cx.local_model_keyed("table_row_1", || true);
    let table_row_2 = cx.local_model_keyed("table_row_2", || false);
    let table_row_3 = cx.local_model_keyed("table_row_3", || false);
    let rows = [
        table_row_1.clone(),
        table_row_2.clone(),
        table_row_3.clone(),
    ];
    let row_values = [
        cx.get_model_copied(&table_row_1, Invalidation::Layout)
            .unwrap_or(false),
        cx.get_model_copied(&table_row_2, Invalidation::Layout)
            .unwrap_or(false),
        cx.get_model_copied(&table_row_3, Invalidation::Layout)
            .unwrap_or(false),
    ];
    let selected_count = row_values.iter().filter(|checked| **checked).count();
    let select_all_state = match selected_count {
        0 => CheckedState::Unchecked,
        n if n == row_values.len() => CheckedState::Checked,
        _ => CheckedState::Indeterminate,
    };

    ui::v_stack(|cx| {
        cx.actions().models::<act::ToggleAllRows>({
            let rows = rows.clone();
            move |models| {
                let should_select_all = rows
                    .iter()
                    .any(|row| models.read(row, |value| !*value).ok().unwrap_or(true));

                rows.iter().fold(false, |updated, row| {
                    models
                        .update(row, |value| *value = should_select_all)
                        .is_ok()
                        || updated
                })
            }
        });

        vec![
            shadcn::table(|cx| {
                ui::children![
                    cx;
                    shadcn::table_header(|cx| {
                        ui::children![
                            cx;
                            shadcn::table_row(3, |cx| {
                                ui::children![
                                    cx;
                                    shadcn::table_head_children(|cx| {
                                        ui::children![
                                            cx;
                                            shadcn::Checkbox::from_checked_state(select_all_state)
                                                .a11y_label("Select all rows")
                                                .action(act::ToggleAllRows)
                                                .test_id("ui-gallery-checkbox-table-all"),
                                        ]
                                    }),
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
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-checkbox-table"),
        ]
    })
    .layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
}
// endregion: example
