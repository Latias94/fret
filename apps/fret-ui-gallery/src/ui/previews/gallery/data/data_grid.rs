use super::super::super::super::*;

pub(in crate::ui) fn preview_data_grid(
    cx: &mut ElementContext<'_, App>,
    selected_row: Model<Option<u64>>,
) -> Vec<AnyElement> {
    let selected = cx
        .get_model_copied(&selected_row, Invalidation::Paint)
        .flatten();

    let selected_text: Arc<str> = selected
        .map(|v| Arc::<str>::from(v.to_string()))
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let grid = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
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
                    0 => cx.text(pid.to_string()),
                    1 => cx.text(format!("Process {row}")),
                    2 => cx.text(if row % 3 == 0 { "Running" } else { "Idle" }),
                    _ => cx.text(((row * 7) % 100).to_string()),
                }
            },
        );

        vec![grid]
    });

    vec![
        cx.text("Virtualized rows/cols viewport; click a row to select (disabled every 17th row)."),
        cx.text(format!("Selected row: {selected_text}")),
        grid,
    ]
}
