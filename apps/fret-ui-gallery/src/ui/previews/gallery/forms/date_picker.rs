use super::super::super::super::*;

pub(in crate::ui) fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    pages::preview_date_picker(cx, open, month, selected)
}
