use fret_ui::{ElementContext, GlobalElementId, UiHost};

pub(in crate::imui) fn hover_blocked_by_active_item_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    active_item_model: &fret_runtime::Model<super::super::ImUiActiveItemState>,
) -> bool {
    let active = cx
        .read_model(
            active_item_model,
            fret_ui::Invalidation::Paint,
            |_app, st| st.active,
        )
        .ok()
        .flatten();
    active.is_some() && active != Some(id)
}
