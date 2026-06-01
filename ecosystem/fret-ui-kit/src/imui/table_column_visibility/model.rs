use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::ImUiTableColumnVisibilityState;

pub(super) fn table_column_visibility_use_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<ImUiTableColumnVisibilityState>>,
    default_value: impl FnOnce() -> ImUiTableColumnVisibilityState,
) -> crate::primitives::controllable_state::ControllableModel<ImUiTableColumnVisibilityState> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}
