use super::super::super::super::*;

pub(in crate::ui) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    pages::preview_data_table(cx, state)
}
