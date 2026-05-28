use std::collections::HashMap;

use fret_core::AppWindowId;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::imui::interaction_runtime::hover) struct ImUiSharedHoverDelayState {
    pub(super) delay_short_met: bool,
    pub(super) delay_normal_met: bool,
    pub(super) short_timer: Option<fret_runtime::TimerToken>,
    pub(super) normal_timer: Option<fret_runtime::TimerToken>,
    pub(super) clear_timer: Option<fret_runtime::TimerToken>,
}

impl ImUiSharedHoverDelayState {
    pub(in crate::imui::interaction_runtime::hover) fn delay_flags(self) -> (bool, bool) {
        (self.delay_short_met, self.delay_normal_met)
    }
}

#[derive(Default)]
struct ImUiSharedHoverDelayStore {
    by_window: HashMap<AppWindowId, fret_runtime::Model<ImUiSharedHoverDelayState>>,
}

pub(in crate::imui::interaction_runtime::hover) fn model_for_window<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> fret_runtime::Model<ImUiSharedHoverDelayState> {
    let window = cx.window;
    cx.app
        .with_global_mut_untracked(ImUiSharedHoverDelayStore::default, |st, app| {
            st.by_window
                .entry(window)
                .or_insert_with(|| {
                    app.models_mut()
                        .insert(ImUiSharedHoverDelayState::default())
                })
                .clone()
        })
}
