use fret_ui::{ElementContext, GlobalElementId, UiHost};

pub(in crate::imui) fn model_value_changed_for<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    current: T,
) -> bool
where
    T: Clone + PartialEq + 'static,
{
    cx.state_for(
        id,
        || current.clone(),
        |previous| {
            let changed = previous != &current;
            if changed {
                *previous = current.clone();
            }
            changed
        },
    )
}
