mod apply;
mod selection;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use crate::core::GroupId;
use crate::io::NodeGraphViewState;

pub(super) fn bring_selected_groups_to_front_in_view_state(
    view_state: &mut NodeGraphViewState,
    selected_groups: &[GroupId],
) {
    let selected_in_order =
        selection::selected_groups_in_draw_order(&view_state.group_draw_order, selected_groups);
    apply::bring_selected_groups_to_front_in_view_state(
        view_state,
        selected_groups,
        selected_in_order,
    );
}

pub(super) fn send_selected_groups_to_back_in_view_state(
    view_state: &mut NodeGraphViewState,
    selected_groups: &[GroupId],
) {
    let selected_in_order =
        selection::selected_groups_in_draw_order(&view_state.group_draw_order, selected_groups);
    apply::send_selected_groups_to_back_in_view_state(
        view_state,
        selected_groups,
        selected_in_order,
    );
}
