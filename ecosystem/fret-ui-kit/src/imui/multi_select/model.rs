use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::ImUiMultiSelectState;

pub(super) fn multi_select_use_model<H: UiHost, K: Clone + 'static>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<ImUiMultiSelectState<K>>>,
    default_value: impl FnOnce() -> ImUiMultiSelectState<K>,
) -> crate::primitives::controllable_state::ControllableModel<ImUiMultiSelectState<K>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}
