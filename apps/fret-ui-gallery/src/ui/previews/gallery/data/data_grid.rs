use super::super::super::super::*;
use fret::AppComponentCx;
use fret_ui_shadcn::facade as shadcn;

fn data_grid_cell_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    fret_ui_kit::declarative::text::text_table_cell(cx, text)
}

pub(in crate::ui) fn preview_data_grid(
    cx: &mut AppComponentCx<'_>,
    selected_row: Model<Option<u64>>,
) -> Vec<AnyElement> {
    let selected = cx
        .get_model_copied(&selected_row, Invalidation::Paint)
        .flatten();

    let selected_text: Arc<str> = selected
        .map(|v| Arc::<str>::from(v.to_string()))
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let grid = cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        |cx| {
            let selected = cx
                .get_model_copied(&selected_row, Invalidation::Layout)
                .flatten();

            let grid = shadcn::experimental::DataGridElement::new(
                ["PID", "Name", "State", "CPU%"],
                DATA_GRID_ROWS,
            )
            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
            .into_element(
                cx,
                1,
                1,
                |row| row as u64,
                move |row| {
                    let is_selected = selected == Some(row as u64);
                    let cmd = data_grid_row_command(row).unwrap_or_else(|| {
                        // Fallback for out-of-range row IDs.
                        CommandId::new(format!("{CMD_DATA_GRID_ROW_PREFIX}{row}"))
                    });
                    shadcn::experimental::DataGridRowState {
                        selected: is_selected,
                        enabled: row % 17 != 0,
                        on_click: Some(cmd),
                    }
                },
                |cx, row, col| {
                    let pid = 1000 + row as u64;
                    match col {
                        0 => data_grid_cell_text(cx, pid.to_string()),
                        1 => data_grid_cell_text(cx, format!("Process {row}")),
                        2 => data_grid_cell_text(cx, if row % 3 == 0 { "Running" } else { "Idle" }),
                        _ => data_grid_cell_text(cx, ((row * 7) % 100).to_string()),
                    }
                },
            );

            vec![grid]
        },
    );

    vec![
        doc_layout::paragraph_text(
            cx,
            "Virtualized rows/cols viewport; click a row to select (disabled every 17th row).",
        ),
        doc_layout::control_readout_text(cx, format!("Selected row: {selected_text}")),
        grid,
    ]
}
