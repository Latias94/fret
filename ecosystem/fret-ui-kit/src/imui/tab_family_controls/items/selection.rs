use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::BuiltTabItem;

pub(super) fn normalize_selected_tab<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: &Model<Option<Arc<str>>>,
    items: &[BuiltTabItem],
) -> Option<Arc<str>> {
    let current = cx
        .read_model(
            selected_model,
            fret_ui::Invalidation::Paint,
            |_app, value| value.clone(),
        )
        .unwrap_or(None);
    let current_is_valid = current.as_ref().is_some_and(|selected_id| {
        items
            .iter()
            .any(|item| item.enabled && item.id.as_ref() == selected_id.as_ref())
    });
    if current_is_valid {
        return current;
    }

    let next = items
        .iter()
        .find(|item| item.enabled && item.default_selected)
        .or_else(|| items.iter().find(|item| item.enabled))
        .map(|item| item.id.clone());
    let _ = cx.app.models_mut().update(selected_model, |value| {
        *value = next.clone();
    });
    next
}
