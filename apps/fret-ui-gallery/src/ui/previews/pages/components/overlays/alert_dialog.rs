use super::super::super::super::super::*;

pub(in crate::ui) fn preview_alert_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_alert_dialog(cx, open)
}
