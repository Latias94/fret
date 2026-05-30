use fret_ui::element::{ContainerProps, PressableState};
use fret_ui::{ElementContext, UiHost};

mod children;
mod props;

pub(super) use children::combo_trigger_children;
pub(super) use props::{ComboTriggerPropsInput, combo_trigger_props};

#[cfg(test)]
pub(in crate::imui::combo_controls) use props::combo_trigger_a11y_label;

pub(super) fn combo_trigger_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    state: PressableState,
) -> (
    super::super::super::control_chrome::ImUiControlPalette,
    ContainerProps,
) {
    super::super::super::control_chrome::field_chrome(cx, enabled, state)
}
